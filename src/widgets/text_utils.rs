use cosmic_text::{Buffer, BufferLine, LayoutRun};
use iced::advanced::mouse::{Click, click};
use iced::Point;
use unicode_segmentation::UnicodeSegmentation as _;

/// Like `buffer.hit(x, y)` but falls back to a y-based line lookup when `hit()` returns
/// `None`. This handles blank lines, which have no glyphs and thus can't be hit directly.
/// Returns `(logical_line_index, byte_offset)`.
pub fn hit_or_nearest(buffer: &Buffer, x: f32, y: f32) -> Option<(usize, usize)> {
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
        let visual_count = line.layout_opt().map(|v| v.len()).unwrap_or(1).max(1);
        if target_visual < visual_start + visual_count || i + 1 == buffer.lines.len() {
            return Some((i, line.text().len()));
        }
        visual_start += visual_count;
    }

    let last = buffer.lines.len() - 1;
    Some((last, buffer.lines[last].text().len()))
}

/// Returns `(x, width)` of the highlighted byte range `[from, to)` within a slice of glyphs.
pub(crate) fn highlight_glyphs(
    glyphs: &[cosmic_text::LayoutGlyph],
    from: usize,
    to: usize,
) -> (f32, f32) {
    if glyphs.is_empty() {
        return (0.0, 0.0);
    }
    let line_start = glyphs.first().map(|g| g.start).unwrap_or(0);
    let line_end = glyphs.last().map(|g| g.end).unwrap_or(0);
    let range = line_start.max(from)..line_end.min(to);
    if range.is_empty() {
        return (0.0, 0.0);
    }
    let first = glyphs
        .iter()
        .position(|g| range.start <= g.start)
        .unwrap_or(0);
    let mut it = glyphs.iter();
    let x: f32 = it.by_ref().take(first).map(|g| g.w).sum();
    let width: f32 = it.take_while(|g| range.end > g.start).map(|g| g.w).sum();
    (x, width)
}

/// Returns `(x, width)` for each visual sub-line within a `BufferLine`.
pub fn highlight_line(buffer_line: &BufferLine, from: usize, to: usize) -> Vec<(f32, f32)> {
    let layout = buffer_line
        .layout_opt()
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    layout
        .iter()
        .map(|visual_line| highlight_glyphs(&visual_line.glyphs, from, to))
        .collect()
}

/// Returns `(x, width)` of the highlighted byte range within a `LayoutRun`.
pub fn highlight_run(run: &LayoutRun<'_>, from: usize, to: usize) -> (f32, f32) {
    highlight_glyphs(run.glyphs, from, to)
}

/// Normalize a selection range so that start <= end.
pub fn normalize_selection(
    anchor_line: usize,
    anchor_idx: usize,
    focus_line: usize,
    focus_idx: usize,
) -> ((usize, usize), (usize, usize)) {
    if (anchor_line, anchor_idx) <= (focus_line, focus_idx) {
        ((anchor_line, anchor_idx), (focus_line, focus_idx))
    } else {
        ((focus_line, focus_idx), (anchor_line, anchor_idx))
    }
}

/// Extract the selected text from a buffer given a selection range.
pub fn extract_selection_text(
    buffer: &Buffer,
    anchor_line: usize,
    anchor_idx: usize,
    focus_line: usize,
    focus_idx: usize,
) -> Option<String> {
    let ((start_line, start_idx), (end_line, end_idx)) =
        normalize_selection(anchor_line, anchor_idx, focus_line, focus_idx);

    if (start_line, start_idx) >= (end_line, end_idx) {
        return None;
    }

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

    if selected_text.is_empty() {
        None
    } else {
        Some(selected_text)
    }
}

/// Compute a new selection based on a mouse click inside a text buffer.
///
/// Returns `None` if the click does not hit any text (e.g., empty padding).
pub fn selection_from_click(
    buffer: &Buffer,
    click: Click,
    mouse_pos: Point,
) -> Option<((usize, usize), (usize, usize))> {
    let c = buffer.hit(mouse_pos.x, mouse_pos.y)?;
    let line_text = buffer.lines[c.line].text();

    Some(match click.kind() {
        click::Kind::Single => ((c.line, c.index), (c.line, c.index)),
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
            ((c.line, start), (c.line, end))
        }
        click::Kind::Triple => ((c.line, 0), (c.line, line_text.len())),
    })
}

/// Compute a new selection while dragging.
///
/// Returns `None` if the drag position does not resolve to a text position.
pub fn selection_from_drag(
    buffer: &Buffer,
    anchor: (usize, usize),
    mouse_pos: Point,
) -> Option<((usize, usize), (usize, usize))> {
    let focus = hit_or_nearest(buffer, mouse_pos.x, mouse_pos.y)?;
    Some((anchor, focus))
}
