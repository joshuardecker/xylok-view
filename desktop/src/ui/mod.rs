mod assets;
mod styles;
mod themes;

// Re-exports.
pub use assets::*;
pub use themes::*;

use iced::{
    Alignment::End,
    Center, Element, Fill, FillPortion, Shrink,
    widget::{
        Id, button, column, container, lazy, mouse_area, opaque, pick_list, row, rule, scrollable,
        sensor, space, stack, svg, text, text_input, toggler, tooltip,
    },
};
use stig_view_core::CKLStatus;

use crate::app::*;
use crate::ui::styles::*;
use crate::widgets::{markdown, selectable_text};

/// The default seperation between elements.
/// I use magic values around because they look better.
const SEPERATION: f32 = 8.0;

impl App {
    /// Get the view of the application.
    pub fn view(&self) -> Element<'_, Message> {
        let window_decorations = self.window_decorations();
        let content = row![
            self.stig_list(),
            space().width(SEPERATION * 2.0),
            self.displayed_stig()
        ]
        .into();

        let padded_content = self.padding(window_decorations, content);

        let popup = match self.popup {
            Popup::Filter => self.filter_menu(),
            Popup::Settings => self.settings_menu(),
            Popup::Save => self.save_menu(),
            Popup::None => space().into(),
        };

        let err_notification = match &self.err_notif {
            Some(err_str) => self.display_error(&err_str),
            None => space().into(),
        };

        stack![padded_content, popup, err_notification].into()
    }

    /// A generic function that pads the content with window decorations
    /// and resize regions the user can click and drag to resize the window.
    /// A generic function that pads the content with window decorations
    /// and resize regions the user can click and drag to resize the window.
    fn padding<'a>(
        &self,
        window_decorations: Element<'a, Message>,
        content: Element<'a, Message>,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        use iced::window::Direction::{
            East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West,
        };

        // There are a few mouse areas here.
        // Without window decorations, we need to handle windoe drag and click resizing ourselves.
        // So we surround the gui on every edge with a mouse area to detect window resizing.

        container(column![
            // Top section above window decorations.
            row![
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION)
                            .height(SEPERATION)
                    )
                    .on_press(Message::WindowDragResize(NorthWest))
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(Fill)
                            .height(SEPERATION)
                    )
                    .on_press(Message::WindowDragResize(North))
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION)
                            .height(SEPERATION)
                    )
                    .on_press(Message::WindowDragResize(NorthEast))
                ),
            ],
            window_decorations,
            space().height(SEPERATION),
            // The main area of the application.
            // Surrounded on left and right by drag click resize areas.
            row![
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(Fill)
                    )
                    .on_press(Message::WindowDragResize(West))
                ),
                content,
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(Fill)
                    )
                    .on_press(Message::WindowDragResize(East))
                ),
            ],
            // Bottom section below the main content.
            row![
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(SEPERATION * 2.0)
                    )
                    .on_press(Message::WindowDragResize(SouthWest))
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(Fill)
                            .height(SEPERATION * 2.0)
                    )
                    .on_press(Message::WindowDragResize(South))
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(SEPERATION * 2.0)
                    )
                    .on_press(Message::WindowDragResize(SouthEast))
                ),
            ],
        ])
        .into()
    }

    /// Gets a column of all loaded STIGs, allowing the user to choose which one
    /// to display. Acts like a file tree.
    fn stig_list(&self) -> Element<'_, Message> {
        if self.benchmark.rules.is_empty() {
            return container(space::vertical())
                .style(background_container)
                .width(300)
                .into();
        }

        lazy(self.stig_list_hash, |_| {
            // A few buttons that allow the user to switch what value is displayed on the buttons.
            // Separate to the scrollable, should always be present.
            let header = column![
                row![
                    button(text("Group ID").size(12).center())
                        .on_press(Message::SwitchDisplayType(DisplayType::GroupId))
                        .style(rounded_primary_button)
                        .width(FillPortion(1)),
                    space().width(SEPERATION),
                    button(text("Rule ID").size(12).center())
                        .on_press(Message::SwitchDisplayType(DisplayType::RuleId))
                        .style(rounded_primary_button)
                        .width(FillPortion(1)),
                    space().width(SEPERATION),
                    button(text("STIG ID").size(12).center())
                        .on_press(Message::SwitchDisplayType(DisplayType::STIGId))
                        .style(rounded_primary_button)
                        .width(FillPortion(1)),
                ],
                space().height(SEPERATION)
            ]
            .align_x(Center);

            let mut not_pin_col = column![];
            let mut user_pin_col = column![];
            let mut filter_pin_col = column![];
            let mut filter_user_pin_col = column![];

            // Counters used to keep track of the total number of compliant,
            // noncompliant, and manual review recommendations.
            let mut compliant_counter = 0;
            let mut manual_counter = 0;
            let mut noncompliant_counter = 0;

            // The amount of filtered STIGs.
            // Columns do not have a len() function, so I keep track here.
            // If this is greater than 0, a seperating rule will be placed between
            // filtered and non filtered STIGs.
            let mut total_filtered = 0;

            for (name, rule) in self.benchmark.rules.iter() {
                match &rule.ckl_status {
                    Some(CKLStatus::NotAFinding) => compliant_counter += 1,
                    Some(CKLStatus::Open) => noncompliant_counter += 1,
                    Some(CKLStatus::NotApplicable) => compliant_counter += 1,
                    Some(CKLStatus::NotReviewed) => manual_counter += 1,
                    None => (),
                }

                let pin_type = self.pins.get(name).unwrap_or(&Pinned::Not);

                let button = self.stig_button(
                    pin_type.to_owned(),
                    name.to_owned(),
                    rule.ckl_status.clone(),
                    rule.rule_id.clone(),
                    rule.stig_id.clone(),
                );

                match pin_type {
                    Pinned::Not => not_pin_col = not_pin_col.push(button).push(space().height(8)),
                    Pinned::ByUser => {
                        user_pin_col = user_pin_col.push(button).push(space().height(8))
                    }
                    Pinned::ByFilter => {
                        // Puts a nice strip of color on the left side of the button.
                        let button_with_accent: Element<'_, Message> = row![
                            container(space::horizontal())
                                .width(SEPERATION * 0.5)
                                .height(Fill)
                                .style(filter_accent),
                            button
                        ]
                        .into();

                        filter_pin_col = filter_pin_col
                            .push(button_with_accent)
                            .push(space().height(SEPERATION));

                        total_filtered += 1;
                    }
                    Pinned::ByFilterAndUser => {
                        // Puts a nice strip of color on the left side of the button.
                        let button_with_accent: Element<'_, Message> = row![
                            container(space::horizontal())
                                .width(SEPERATION * 0.5)
                                .height(Fill)
                                .style(filter_accent),
                            button
                        ]
                        .into();

                        filter_user_pin_col = filter_user_pin_col
                            .push(button_with_accent)
                            .push(space().height(SEPERATION));

                        total_filtered += 1
                    }
                }
            }

            // Counters visually displays how many of each ckl status is present.
            // If a non ckl was loaded, this will not be displayed.
            let counters: Element<'_, Message> =
                if (compliant_counter + manual_counter + noncompliant_counter) != 0 {
                    column![
                        rule::horizontal(2),
                        space().height(SEPERATION),
                        row![
                            tooltip(
                                svg(SQUARE.clone()).width(12).height(12).style(good_svg),
                                container("Total Compliant.")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Bottom
                            ),
                            space().width(SEPERATION * 0.5),
                            text(compliant_counter.to_string()),
                            space().width(SEPERATION * 2.0),
                            tooltip(
                                svg(SQUARE.clone()).width(12).height(12).style(bad_svg),
                                container("Total Non-Compliant.")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Bottom
                            ),
                            space().width(SEPERATION * 0.5),
                            text(noncompliant_counter.to_string()),
                            space().width(SEPERATION * 2.0),
                            tooltip(
                                svg(SQUARE.clone()).width(12).height(12).style(warning_svg),
                                container("Total Manual Review.")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Bottom
                            ),
                            space().width(SEPERATION * 0.5),
                            text(manual_counter.to_string()),
                        ]
                        .align_y(Center),
                        space().height(SEPERATION),
                    ]
                    .align_x(Center)
                    .into()
                } else {
                    space().into()
                };

            // Place a horizontal rule if there are any STIGs that have been filtered.
            let horizontal_rule: Element<'_, Message> = if total_filtered != 0 {
                column![rule::horizontal(2), space().height(SEPERATION)].into()
            } else {
                space().into()
            };

            container(column![
                header,
                counters,
                scrollable(column![
                    filter_user_pin_col,
                    filter_pin_col,
                    horizontal_rule,
                    user_pin_col,
                    not_pin_col,
                    space::vertical(), // Ensures this container is proper size.
                ])
                .spacing(SEPERATION),
            ])
            .width(300)
            .style(background_container)
            .padding(8)
        })
        .into()
    }

    /// Get a button the user can click to swich displayed STIGs.
    fn stig_button(
        &self,
        pin_type: Pinned,
        name: String,
        ckl_status: Option<CKLStatus>,
        rule_id: String,
        stig_id: Option<String>,
    ) -> Element<'static, Message> {
        // A visual indicator of the cki status of a STIG.
        let cki_status: Element<'_, Message> = match &ckl_status {
            Some(CKLStatus::NotAFinding) => row![
                tooltip(
                    svg(CHECKED_CIRCLE.clone())
                        .width(16)
                        .height(16)
                        .style(good_svg),
                    container("Compliant.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),
            Some(CKLStatus::Open) => row![
                tooltip(
                    svg(CROSS_CIRCLE.clone())
                        .width(16)
                        .height(16)
                        .style(bad_svg),
                    container("Non-Compliant.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),
            Some(CKLStatus::NotApplicable) => row![
                tooltip(
                    svg(CHECKED_CIRCLE.clone())
                        .width(16)
                        .height(16)
                        .style(good_svg),
                    container("Not Applicable.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),
            Some(CKLStatus::NotReviewed) => row![
                tooltip(
                    svg(MINUS_CIRCLE.clone())
                        .width(16)
                        .height(16)
                        .style(warning_svg),
                    container("Not Reviewed.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),

            // If no status, dont add any visual element.
            None => space().into(),
        };

        // Button theme depends on whether a filter has pinned it.
        // Make the button more obvious when its contents matches a filter.
        let theme = match pin_type {
            Pinned::Not => rounded_boring_button,
            Pinned::ByUser => rounded_boring_button,
            Pinned::ByFilter => rounded_boring_button_right,
            Pinned::ByFilterAndUser => rounded_boring_button_right,
        };

        // Get the button text depending on what information the user has chosen to display
        // for button text.
        let button_text = match self.display_type {
            DisplayType::GroupId => name.clone(),
            DisplayType::RuleId => rule_id,
            // If there is no STIG Id, fall back to Group Id since its always known.
            DisplayType::STIGId => stig_id.unwrap_or(name.clone()),
        };

        let bookmark_symbol = match pin_type {
            Pinned::Not => BOOKMARK.clone(),
            Pinned::ByUser => FILLED_BOOKMARK.clone(),
            Pinned::ByFilter => BOOKMARK.clone(),
            Pinned::ByFilterAndUser => FILLED_BOOKMARK.clone(),
        };

        button(
            column![
                row![
                    cki_status,
                    text(button_text).center(),
                    space::horizontal(),
                    button(svg(bookmark_symbol).width(32).height(32).style(colored_svg))
                        .padding(1)
                        .style(no_button)
                        .on_press(Message::Pin(name.clone()))
                ]
                .align_y(Center)
                .height(Fill),
            ]
            .align_x(Center)
            .width(Fill),
        )
        .height(SEPERATION * 8.0)
        .padding(8)
        .width(Fill)
        .style(theme)
        .on_press(Message::Switch(name))
        .into()
    }

    /// Content of the currently selected STIG.
    fn displayed_stig(&self) -> Element<'_, Message> {
        // Get the displayed STIG.
        // If there is none, display a special screen.
        let stig_rule = match &self.displayed {
            Some(rule) => rule,
            None => return self.display_empty(),
        };

        let lazy_widget = lazy((&stig_rule.group_id, &self.filter_string), |_| {
            let content = column![
            row![
                column![
                    text("Group ID").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.group_id.clone()).highlight_str(
                        self.filter_string.clone(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION),
                    text("Severity").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.severity.as_str()).highlight_str(
                        self.filter_string.clone(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                ]
                .align_x(Center)
                .width(FillPortion(1)),
                space().width(SEPERATION),
                rule::vertical(2),
                space().width(SEPERATION),
                column![
                    text("Rule ID").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.rule_id.clone()).highlight_str(
                        self.filter_string.clone(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION),
                ]
                .align_x(Center)
                .width(FillPortion(1)),
                space().width(SEPERATION),
                rule::vertical(2),
                space().width(SEPERATION),
                column![
                    text("STIG ID").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.stig_id.clone().unwrap_or("None".into())).highlight_str(
                        self.filter_string.clone(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION),
                    text("Documentable").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.documentable_str()).highlight_str(
                        self.filter_string.clone(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                ]
                .align_x(Center)
                .width(FillPortion(1)),
            ],
            space().height(SEPERATION),
            row![
                space().width(SEPERATION),
                Element::from(
                    markdown::view_selectable(
                        markdown::parse(&format!(
                            "# Introduction\n{}\n# Description\n{}\n# Check\n{}\n# Fix\n{}\n# CCIs\n{}\n# False Positives\n{}\n# False Negatives\n{}",
                            stig_rule.title.clone(),
                            stig_rule.vuln_discussion.clone(),
                            stig_rule.check_text.clone(),
                            stig_rule.fix_text.clone(),
                            stig_rule
                                .cci_refs
                                .clone()
                                .map(|strings| strings.join("\n"))
                                .unwrap_or_default(),
                            stig_rule.false_positives.clone().unwrap_or("".into()),
                            stig_rule.false_negatives.clone().unwrap_or("".into()),
                        )),
                        markdown::Settings::from(self.theme()),
                    )
                    .highlight_str(&self.filter_string, |theme| {
                        theme.extended_palette().primary.weak.color
                    })
                )
                .map(|_| Message::DoNothing)
            ],
        ];

            // Wrap it in a scrollable.
            let content = scrollable(content).spacing(SEPERATION);

            let content = container(content)
                .center(Fill)
                .padding(8)
                .style(background_container);

            content
        });

        // Stack the content with a container that fades in and out.
        // This acts as animation, showing the user the STIG has changed when
        // a new STIG is selected.
        stack![
            lazy_widget,
            container(space())
                .width(Fill)
                .height(Fill)
                .style(fade_overlay(1.0 - self.main_col_opacity))
        ]
        .into()
    }

    /// This gets displayed when no STIG is selected:
    /// A button prompting the user to choose a benchmark to load into the viewer.
    fn display_empty(&self) -> Element<'_, Message> {
        use std::path::PathBuf;

        let lazy_view = lazy(self.home_menu_hash, |_| {
            // Load any benchmarks the user opted to save in the past.
            let cache = App::load_cache();

            // Change the displayed string based on if the cache loaded any items.
            let displayed_string = if cache.is_empty() {
                "Open a File to Get Started"
            } else {
                "Recently Saved Files"
            };

            let mut main_col = column![];

            // If the cache is empty, add an obvious button for the user to click that opens a new benchmark.
            if cache.is_empty() {
                main_col = main_col.push(
                    button(text("Open").center())
                        .width(SEPERATION * 10.0)
                        .height(SEPERATION * 5.0)
                        .style(rounded_boring_button)
                        .on_press(Message::OpenFile),
                )
            }

            // A vector that contains the unix time when this cache entry was last opened,
            // its path, and its nicely formatted name.
            let mut times_last_loaded: Vec<(u64, PathBuf, String)> = Vec::new();

            for path in cache {
                match path.file_name().and_then(|os_str| os_str.to_str()) {
                    Some(str) => {
                        // If this file for whatever reason isnt the type we are looking for.
                        if !str.ends_with(".msgpack.zstd") {
                            continue;
                        }

                        let str = str.trim_end_matches(".msgpack.zstd");

                        // Get the last time this benchmark was accessed.
                        let time_last = self.last_opened.get_time_used(str);

                        // Trim the file extension off, and make the name a little prettier.
                        let name: String = str
                            .chars()
                            .flat_map(|c| match c {
                                '_' | '-' => ' '.to_lowercase(),
                                c => c.to_lowercase(),
                            })
                            .collect();

                        // Save the time last accessed, path, and formatted name.
                        times_last_loaded.push((time_last, path, name));
                    }

                    None => continue,
                };
            }

            // Sort most recent to oldest.
            times_last_loaded.sort_by(|a, b| b.0.cmp(&a.0));

            for time_loaded in times_last_loaded {
                main_col = main_col.push(
                    button(
                        row![
                            svg(FILE_ICON.clone())
                                .style(boring_svg)
                                .width(20)
                                .height(20),
                            space().width(SEPERATION),
                            text(time_loaded.2).center(),
                            space::horizontal(),
                            button(svg(TRASH.clone()).style(colored_svg).width(20).height(20))
                                .style(no_button)
                                .on_press(Message::DeleteCachedBenchmark(time_loaded.1.clone())),
                        ]
                        .align_y(Center),
                    )
                    .width(Fill)
                    .style(rounded_boring_button)
                    .on_press(Message::LoadCachedBenchmark(time_loaded.1)),
                );

                // Space out each file entry nicely.
                main_col = main_col.push(space().height(SEPERATION));
            }

            container(
                column![
                    text(displayed_string).size(24).center(),
                    space().height(SEPERATION * 3.0),
                    scrollable(main_col).spacing(SEPERATION)
                ]
                .align_x(Center)
                .width(400),
            )
            .padding(30)
            .center(Fill)
            .style(background_container)
        });

        // Wrap lazy in a container so that it fills the whole width.
        // For some reason it shrinks the content inside of it.
        container(lazy_view).width(Fill).height(Fill).into()
    }

    /// Display of the filter menu, gets stacked on top of the main application view.
    fn filter_menu(&self) -> Element<'_, Message> {
        let id = Id::new("filter_text_input");

        // The filter popup itself.
        let popup: Element<'_, Message> = container(
            sensor(opaque(stack![
                container(
                    row![
                        button(svg(REFRESH.clone()).style(colored_svg).width(25).height(25))
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::ProcessCmd("reset".to_string())),
                        space::horizontal(),
                        text_input(
                            "Type keywords here, then press enter...",
                            &self.filter_input
                        )
                        .on_input(Message::TypeCmd)
                        .on_submit(Message::ProcessCmd(self.filter_input.clone()))
                        .id(id.clone())
                        .width(320),
                        space::horizontal(),
                        button(svg(CROSS.clone()).style(colored_svg).width(16).height(16))
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::SwitchPopup(Popup::None)),
                    ]
                    .align_y(Center),
                )
                .width(400)
                .height(Shrink)
                .padding(8)
                .style(cmd_container),
                container(space())
                    .width(Fill)
                    .height(Fill)
                    .style(fade_overlay(1.0 - self.popup_opacity)),
            ]))
            .on_show(move |_| Message::FocusWidget(id.clone())),
        )
        .width(Fill)
        .height(Fill)
        .align_x(Center)
        .align_y(End)
        .into();

        // Add some space below it, that way it is not hugging the bottom of the window.
        // Looks nicer this way.
        column![popup, space().height(SEPERATION * 4.0)].into()
    }

    /// Display of the settings menu, gets stacked on top of the main application view.
    fn settings_menu(&self) -> Element<'_, Message> {
        let themes = [
            AppTheme::Dark,
            AppTheme::Light,
            AppTheme::HighContrast,
            AppTheme::Coffee,
        ];
        let display_types = [
            DisplayType::GroupId,
            DisplayType::RuleId,
            DisplayType::STIGId,
        ];

        container(opaque(stack![
            container(
                column![
                    row![
                        space::horizontal(),
                        text("Settings Menu"),
                        space::horizontal(),
                        button(svg(CROSS.clone()).style(colored_svg).width(16).height(16))
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::SwitchPopup(Popup::None)),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION * 4.0),
                    row![
                        text("Theme"),
                        space::horizontal(),
                        pick_list(themes, Some(self.settings.theme), Message::SwitchTheme),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![
                        text("Default Display Type"),
                        space::horizontal(),
                        pick_list(
                            display_types,
                            Some(self.settings.default_display_type),
                            Message::SaveDisplayType
                        ),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![
                        text("Animations"),
                        space::horizontal(),
                        toggler(self.settings.animate)
                            .on_toggle(Message::SaveAnimate)
                            .style(toggler_theme),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![
                        text("Notify About Updates"),
                        space::horizontal(),
                        toggler(self.settings.notify_if_update)
                            .on_toggle(Message::SaveUpdateNotify)
                            .style(toggler_theme),
                    ]
                    .align_y(Center),
                ]
                .align_x(Center),
            )
            .width(375)
            .height(200)
            .padding(8)
            .style(cmd_container),
            container(space())
                .width(Fill)
                .height(Fill)
                .style(fade_overlay(1.0 - self.popup_opacity)),
        ]))
        .center(Fill)
        .into()
    }

    /// Display of an error that occured, gets stacked on top of the main application view.
    fn display_error<'a>(&self, err_str: &'a str) -> Element<'a, Message> {
        container(opaque(
            container(
                column![
                    row![
                        space::horizontal(),
                        text("Error Occurred"),
                        space::horizontal(),
                        button(svg(CROSS.clone()).style(boring_svg).width(16).height(16))
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::ClearErrNotif),
                    ]
                    .align_y(Center),
                    space::vertical(),
                    row![
                        text(err_str)
                            .size(12)
                            .height(Fill)
                            .wrapping(text::Wrapping::WordOrGlyph)
                    ]
                    .align_y(Center),
                    space::vertical(),
                ]
                .align_x(Center),
            )
            .width(250)
            .height(125)
            .padding(8)
            .style(err_container),
        ))
        .align_right(Fill)
        .align_bottom(Fill)
        .padding(SEPERATION * 4.0)
        .into()
    }

    /// Display to the user that an update is available.
    fn display_update_available<'a>(&self) -> Element<'a, Message> {
        if !self.display_update_available {
            return space().into();
        }

        container(
            row![
                space().width(SEPERATION * 0.5),
                button(svg(CROSS.clone()).style(boring_svg).width(10).height(10))
                    .padding(0)
                    .width(Shrink)
                    .height(Shrink)
                    .style(no_button)
                    .on_press(Message::SwitchDisplayUpdateAvailable(false)),
                space().width(SEPERATION),
                button(text("Update Available").size(11).center())
                    .style(no_button)
                    .padding(0)
                    .on_press(Message::OpenURL(
                        "https://github.com/joshuardecker/stig-view/releases"
                    )),
                space().width(SEPERATION * 0.5),
            ]
            .align_y(Center),
        )
        .padding(4)
        .style(update_available_container)
        .width(Shrink)
        .height(22)
        .into()
    }

    /// A menu prompting the user to save the benchmark to the cache.
    fn save_menu(&self) -> Element<'_, Message> {
        container(opaque(stack![
            container(
                column![
                    row![
                        space::horizontal(),
                        text("Save Benchmark for Later?"),
                        space::horizontal(),
                        button(svg(CROSS.clone()).style(colored_svg).width(16).height(16))
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::SwitchPopup(Popup::None)),
                    ]
                    .align_y(Center),
                    space::vertical(),
                    row![
                        space::horizontal(),
                        button(text("Cancel").size(14).center())
                            .style(rounded_danger_button)
                            .width(65)
                            .height(30)
                            .on_press(Message::SwitchPopup(Popup::None)),
                        space().width(SEPERATION * 8.0),
                        button(text("Confirm").size(14).center())
                            .style(rounded_success_button)
                            .width(70)
                            .height(30)
                            .on_press(Message::SaveBenchmark),
                        space::horizontal()
                    ]
                    .align_y(Center),
                ]
                .align_x(Center),
            )
            .width(270)
            .height(120)
            .padding(8)
            .style(cmd_container),
            container(space())
                .width(Fill)
                .height(Fill)
                .style(fade_overlay(1.0 - self.popup_opacity)),
        ]))
        .center(Fill)
        .into()
    }

    /// Return the window decorations container.
    fn window_decorations(&self) -> Element<'_, Message> {
        lazy((&self.benchmark.id, self.display_update_available), |_| {
            // A complicated way of getting mouse_area to work.
            // Captures mouse input in the window decorations so the window can be dragged.
            container(
                mouse_area(
                    container(
                        row![
                            space().width(15),
                            tooltip(
                                button(
                                    svg(SETTINGS.clone())
                                        .style(colored_svg)
                                        .width(18)
                                        .height(18)
                                )
                                .padding(1)
                                .width(Shrink)
                                .height(Shrink)
                                .style(no_button)
                                .on_press(Message::SwitchPopup(Popup::Settings)),
                                container("Customize Settings.")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Right
                            )
                            .delay(iced::time::Duration::from_millis(600)),
                            space().width(11),
                            tooltip(
                                button(svg(HOME.clone()).style(colored_svg).width(18).height(18))
                                    .padding(1)
                                    .width(Shrink)
                                    .height(Shrink)
                                    .style(no_button)
                                    .on_press(Message::ReturnHome),
                                container("Return to the Start Screen.")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Right
                            )
                            .delay(iced::time::Duration::from_millis(600)),
                            space().width(8),
                            tooltip(
                                button(text("File").center().size(15))
                                    .padding(4)
                                    .width(Shrink)
                                    .height(Shrink)
                                    .style(rounded_dark_button)
                                    .on_press(Message::OpenFile),
                                container("Open a New File (Ctrl + I)")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Right
                            )
                            .delay(iced::time::Duration::from_millis(600)),
                            space().width(4),
                            tooltip(
                                button(text("Filter").center().size(15))
                                    .padding(4)
                                    .width(Shrink)
                                    .height(Shrink)
                                    .style(rounded_dark_button)
                                    .on_press(Message::SwitchPopup(Popup::Filter)),
                                container("Sort Content Based on Keywords (Ctrl + F)")
                                    .style(background_container)
                                    .padding(4),
                                tooltip::Position::Right
                            )
                            .delay(iced::time::Duration::from_millis(600)),
                            space::horizontal(),
                            // Make the window title look nice.
                            text(
                                self.benchmark
                                    .id
                                    .chars()
                                    .flat_map(|c| match c {
                                        '_' | '-' => ' '.to_lowercase(),
                                        c => c.to_lowercase(),
                                    })
                                    .collect::<String>()
                            )
                            .size(16),
                            {
                                let switch_element: Element<Message> =
                                    if !self.background_benchmarks.is_empty() {
                                        row![
                                            space().width(15),
                                            tooltip(
                                                button(
                                                    svg(SWITCH.clone())
                                                        .style(colored_svg)
                                                        .width(18)
                                                        .height(18)
                                                )
                                                .padding(1)
                                                .width(Shrink)
                                                .height(Shrink)
                                                .style(no_button)
                                                .on_press(Message::SwitchToBackground),
                                                container("Switch Benchmark")
                                                    .style(background_container)
                                                    .padding(4),
                                                tooltip::Position::Right
                                            )
                                            .delay(iced::time::Duration::from_millis(600)),
                                        ]
                                        .into()
                                    } else {
                                        space().width(0).into()
                                    };
                                switch_element
                            },
                            space::horizontal(),
                            self.display_update_available(),
                            space().width(SEPERATION * 4.0),
                            button(
                                svg(DOWN_TICK.clone())
                                    .style(colored_svg)
                                    .width(24)
                                    .height(24)
                            )
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::WindowMin),
                            space().width(15),
                            button(svg(SQUARE.clone()).style(colored_svg).width(16).height(16))
                                .padding(1)
                                .width(Shrink)
                                .height(Shrink)
                                .style(no_button)
                                .on_press(Message::WindowFullscreenToggle),
                            space().width(18),
                            button(svg(CROSS.clone()).style(colored_svg).width(16).height(16))
                                .padding(1)
                                .width(Shrink)
                                .height(Shrink)
                                .style(no_button)
                                .on_press(Message::WindowClose),
                            space().width(15)
                        ]
                        .align_y(Center),
                    )
                    .height(26)
                    .padding(1)
                    .align_x(End)
                    .align_y(Center)
                    .width(Fill),
                )
                .on_press(Message::WindowMove),
            )
        })
        .into()
    }
}
