use std::borrow::Cow;

use iced::{
    Color, Element, Length, Pixels, Rectangle, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        clipboard::Kind as ClipboardKind,
        layout::{Limits, Node},
        mouse::{Click, click},
        renderer,
        text::{self, paragraph},
        widget::tree::{self, Tree},
    },
    alignment::{self, Horizontal},
    event::Event,
    keyboard,
    mouse::{self, Cursor, Interaction},
    widget::text::{LineHeight, Shaping, Wrapping},
};
use iced_graphics::text::Paragraph as ConcreteP;
use unicode_segmentation::UnicodeSegmentation as _;

/// Text that can be highlighted, selected with a mouse, and copied to the clipboard.
pub struct SelectableText<'a> {
    content: Cow<'a, str>,
    size: Option<f32>,
    width: Length,
    align_x: text::Alignment,
    highlight_patterns: Vec<(Cow<'a, str>, Box<dyn Fn(&Theme) -> Color + 'a>)>,
    /// Pre-computed highlight spans: (line_idx, byte_from, byte_to, pattern_idx).
    computed_highlights: Vec<(usize, usize, usize, usize)>,
}

struct State {
    paragraph: paragraph::Plain<ConcreteP>,
    /// Anchor and focus cursors as (logical_line_index, byte_offset_within_line).
    selection: Option<((usize, usize), (usize, usize))>,
    last_click: Option<Click>,
    is_dragging: bool,
}

/// Create selectable text with the given content.
pub fn selectable_text<'a>(content: impl Into<Cow<'a, str>>) -> SelectableText<'a> {
    SelectableText {
        content: content.into(),
        size: None,
        width: Length::Shrink,
        align_x: text::Alignment::Default,
        highlight_patterns: Vec::new(),
        computed_highlights: Vec::new(),
    }
}

impl<'a> SelectableText<'a> {
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.align_x = alignment.into();
        self
    }

    pub fn center(self) -> Self {
        self.align_x(Horizontal::Center)
    }

    /// Highlight the given string with the given color.
    /// Silently does nothing if the input is empty.
    pub fn highlight_str(
        mut self,
        pattern: impl Into<Cow<'a, str>>,
        color: impl Fn(&Theme) -> Color + 'a,
    ) -> Self {
        let pattern = pattern.into();
        if pattern.is_empty() {
            return self;
        }
        let pattern_idx = self.highlight_patterns.len();
        for (line_idx, line) in self.content.split('\n').enumerate() {
            let mut search_start = 0;
            while let Some(rel) = line[search_start..].find(pattern.as_ref()) {
                let from = search_start + rel;
                let to = from + pattern.len();
                self.computed_highlights
                    .push((line_idx, from, to, pattern_idx));
                search_start = to;
            }
        }
        self.highlight_patterns.push((pattern, Box::new(color)));
        self
    }
}

/// Returns (x, width) per visual line within `buffer_line` for the selected byte range [from, to).
/// Directly mirrors iced_graphics::text::editor::highlight_line.
fn highlight_line(
    buffer_line: &cosmic_text::BufferLine,
    from: usize,
    to: usize,
) -> Vec<(f32, f32)> {
    let layout = buffer_line
        .layout_opt()
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    layout
        .iter()
        .map(|visual_line| {
            let line_start = visual_line.glyphs.first().map(|g| g.start).unwrap_or(0);
            let line_end = visual_line.glyphs.last().map(|g| g.end).unwrap_or(0);

            let range = line_start.max(from)..line_end.min(to);

            if range.is_empty() {
                (0.0, 0.0)
            } else if range.start == line_start && range.end == line_end {
                (0.0, visual_line.w)
            } else {
                let first_glyph = visual_line
                    .glyphs
                    .iter()
                    .position(|g| range.start <= g.start)
                    .unwrap_or(0);

                let mut glyphs = visual_line.glyphs.iter();
                let x: f32 = glyphs.by_ref().take(first_glyph).map(|g| g.w).sum();
                let width: f32 = glyphs
                    .take_while(|g| range.end > g.start)
                    .map(|g| g.w)
                    .sum();

                (x, width)
            }
        })
        .collect()
}

/// Returns the total number of visual lines in `buffer` before logical line `line_idx`.
fn visual_lines_before(buffer: &cosmic_text::Buffer, line_idx: usize) -> usize {
    buffer.lines[..line_idx]
        .iter()
        .map(|l| l.layout_opt().map(|v| v.len()).unwrap_or(1).max(1))
        .sum()
}

/// Like `buffer.hit(x, y)` but falls back to a y-based line lookup when `hit()` returns
/// `None`. This handles blank lines, which have no glyphs and thus can't be hit directly.
/// Returns `(logical_line_index, byte_offset)`.
fn hit_or_nearest(buffer: &cosmic_text::Buffer, x: f32, y: f32) -> Option<(usize, usize)> {
    if let Some(c) = buffer.hit(x, y) {
        return Some((c.line, c.index));
    }

    if buffer.lines.is_empty() {
        return None;
    }

    let line_height = buffer.metrics().line_height;
    if line_height <= 0.0 {
        return None;
    }

    let target_visual = (y / line_height).max(0.0) as usize;
    let mut visual_start = 0usize;

    for (i, line) in buffer.lines.iter().enumerate() {
        // Blank lines may have 0 layout entries; treat them as occupying 1 visual line.
        let visual_count = line.layout_opt().map(|v| v.len()).unwrap_or(1).max(1);
        if target_visual < visual_start + visual_count || i + 1 == buffer.lines.len() {
            return Some((i, line.text().len()));
        }
        visual_start += visual_count;
    }

    let last = buffer.lines.len() - 1;
    Some((last, buffer.lines[last].text().len()))
}

/// Draws highlight quads for a byte range within a single buffer line. Returns the number of
/// visual sub-lines consumed, so callers tracking a running visual offset can advance it.
fn draw_highlight_span<R>(
    renderer: &mut R,
    bounds: Rectangle,
    buffer_line: &cosmic_text::BufferLine,
    from: usize,
    to: usize,
    visual_line_start: usize,
    line_height: f32,
    color: Color,
) -> usize
where
    R: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = iced::Font>,
{
    let spans = highlight_line(buffer_line, from, to);
    let count = buffer_line
        .layout_opt()
        .map(|v| v.len())
        .unwrap_or(1)
        .max(1);
    for (sub, (x, width)) in spans.into_iter().enumerate() {
        if width > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + x,
                        y: bounds.y + (visual_line_start + sub) as f32 * line_height,
                        width,
                        height: line_height,
                    },
                    ..renderer::Quad::default()
                },
                color,
            );
        }
    }
    count
}

impl<'a, Message, R> Widget<Message, Theme, R> for SelectableText<'a>
where
    R: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = iced::Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            paragraph: paragraph::Plain::default(),
            selection: None,
            last_click: None,
            is_dragging: false,
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &R, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<State>();

        let font_size = self
            .size
            .map(Pixels)
            .unwrap_or_else(|| renderer.default_size());

        let changed = state.paragraph.update(text::Text {
            content: &self.content,
            bounds: limits.max(),
            size: font_size,
            line_height: LineHeight::default(),
            font: renderer.default_font(),
            align_x: self.align_x,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::default(),
        });

        if changed {
            state.selection = None;
            state.last_click = None;
        }

        let measured_bounds = state.paragraph.min_bounds();
        Node::new(limits.resolve(self.width, Length::Shrink, measured_bounds))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut R,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if !self.computed_highlights.is_empty() {
            let buffer = state.paragraph.raw().buffer();
            let line_height = buffer.metrics().line_height;

            for &(line_idx, from, to, pattern_idx) in &self.computed_highlights {
                let buffer_line = &buffer.lines[line_idx];
                let visual_offset = visual_lines_before(buffer, line_idx);
                let color = self.highlight_patterns[pattern_idx].1(theme);
                draw_highlight_span(
                    renderer,
                    bounds,
                    buffer_line,
                    from,
                    to,
                    visual_offset,
                    line_height,
                    color,
                );
            }
        }

        if let Some(((anchor_line, anchor_idx), (focus_line, focus_idx))) = state.selection {
            let (start_line, start_idx, end_line, end_idx) =
                if (anchor_line, anchor_idx) <= (focus_line, focus_idx) {
                    (anchor_line, anchor_idx, focus_line, focus_idx)
                } else {
                    (focus_line, focus_idx, anchor_line, anchor_idx)
                };

            if (start_line, start_idx) < (end_line, end_idx) {
                let buffer = state.paragraph.raw().buffer();
                let line_height = buffer.metrics().line_height;
                let selection_color = theme.extended_palette().primary.weak.color;
                let selected_logical_lines = end_line - start_line + 1;

                let mut visual_offset = visual_lines_before(buffer, start_line);

                for (i, buffer_line) in buffer
                    .lines
                    .iter()
                    .skip(start_line)
                    .take(selected_logical_lines)
                    .enumerate()
                {
                    let from = if i == 0 { start_idx } else { 0 };
                    let to = if i == selected_logical_lines - 1 {
                        end_idx
                    } else {
                        buffer_line.text().len()
                    };

                    visual_offset += draw_highlight_span(
                        renderer,
                        bounds,
                        buffer_line,
                        from,
                        to,
                        visual_offset,
                        line_height,
                        selection_color,
                    );
                }
            }
        }

        let paragraph_anchor = bounds.anchor(
            state.paragraph.min_bounds(),
            state.paragraph.align_x(),
            state.paragraph.align_y(),
        );

        renderer.fill_paragraph(
            state.paragraph.raw(),
            paragraph_anchor,
            style.text_color,
            *viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(mouse_pos) = cursor.position_in(layout.bounds()) {
                        let click = Click::new(mouse_pos, mouse::Button::Left, state.last_click);

                        if let Some(c) =
                            state.paragraph.raw().buffer().hit(mouse_pos.x, mouse_pos.y)
                        {
                            let buffer = state.paragraph.raw().buffer();
                            let line_text = buffer.lines[c.line].text();

                            state.selection = match click.kind() {
                                click::Kind::Single => {
                                    state.is_dragging = true;
                                    Some(((c.line, c.index), (c.line, c.index)))
                                }
                                click::Kind::Double => {
                                    let start = line_text
                                        .unicode_word_indices()
                                        .rev()
                                        .map(|(i, _)| i)
                                        .find(|&i| i < c.index)
                                        .unwrap_or(0);

                                    let end = line_text
                                        .unicode_word_indices()
                                        .map(|(i, word)| i + word.len())
                                        .find(|&i| i > c.index)
                                        .unwrap_or(line_text.len());

                                    Some(((c.line, start), (c.line, end)))
                                }
                                click::Kind::Triple => {
                                    Some(((c.line, 0), (c.line, line_text.len())))
                                }
                            };
                        } else {
                            state.selection = None;
                        }

                        state.last_click = Some(click);
                    } else {
                        state.selection = None;
                    }
                    shell.request_redraw();
                }

                mouse::Event::CursorMoved { .. } => {
                    if state.is_dragging
                        && state.last_click.map(|c| c.kind()) == Some(click::Kind::Single)
                    {
                        if let Some(mouse_pos) = cursor.position_in(layout.bounds()) {
                            if let Some(((anchor_line, anchor_idx), _)) = state.selection {
                                let focus = hit_or_nearest(
                                    state.paragraph.raw().buffer(),
                                    mouse_pos.x,
                                    mouse_pos.y,
                                );

                                if let Some(focus_pos) = focus {
                                    state.selection = Some(((anchor_line, anchor_idx), focus_pos));
                                    shell.request_redraw();
                                }
                            }
                        }
                    }
                }

                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.is_dragging = false;
                }

                _ => {}
            },

            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if c.as_str() == "c" && modifiers.command() => {
                if let Some(((anchor_line, anchor_idx), (focus_line, focus_idx))) = state.selection
                {
                    let (start_line, start_idx, end_line, end_idx) =
                        if (anchor_line, anchor_idx) <= (focus_line, focus_idx) {
                            (anchor_line, anchor_idx, focus_line, focus_idx)
                        } else {
                            (focus_line, focus_idx, anchor_line, anchor_idx)
                        };

                    if (start_line, start_idx) < (end_line, end_idx) {
                        let buffer = state.paragraph.raw().buffer();
                        let mut selected_text = String::new();
                        let selected_logical_lines = end_line - start_line + 1;

                        for (i, buffer_line) in buffer
                            .lines
                            .iter()
                            .skip(start_line)
                            .take(selected_logical_lines)
                            .enumerate()
                        {
                            if i > 0 {
                                selected_text.push('\n');
                            }

                            let text = buffer_line.text();
                            let from = if i == 0 { start_idx } else { 0 };
                            let to = if i == selected_logical_lines - 1 {
                                end_idx
                            } else {
                                text.len()
                            };

                            selected_text.push_str(&text[from.min(text.len())..to.min(text.len())]);
                        }

                        if !selected_text.is_empty() {
                            clipboard.write(ClipboardKind::Standard, selected_text);
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &R,
    ) -> Interaction {
        if cursor.is_over(layout.bounds()) {
            Interaction::Text
        } else {
            Interaction::default()
        }
    }
}

impl<'a, Message, R> From<SelectableText<'a>> for Element<'a, Message, Theme, R>
where
    R: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = iced::Font> + 'a,
    Message: 'a,
{
    fn from(widget: SelectableText<'a>) -> Self {
        Element::new(widget)
    }
}
