use iced::{Subscription, Theme, keyboard, keyboard::key, window::icon::from_file_data};
use image::ImageFormat;
use rfd::AsyncFileDialog;
use std::time::Instant;

use crate::app::search::*;
use crate::app::*;
use crate::parse::{
    Benchmark, Format, ckl::load_ckl, detection::detect_stig_format, xccdf::load_v1_1,
};
use crate::ui::{APP_ICON, THEME_COFFEE, THEME_DARK, THEME_HIGH_CONTRAST, THEME_LIGHT};

const MAIN_FADE_START: f32 = 0.0;
const MAIN_FADE_DURATION_SECS: f32 = 0.15;

const POPUP_FADE_START: f32 = 0.0;
const POPUP_FADE_DURATION_SECS: f32 = 0.15;

impl App {
    pub fn new() -> (Self, Task<Message>) {
        crate::app::migrate::run();

        let settings = AppSettings::load().unwrap_or(AppSettings::default());
        let last_opened = TimeLastOpened::load().unwrap_or(TimeLastOpened::new());

        let mut tasks = vec![window::oldest().map(Message::InitWindow)];

        if settings.notify_if_update {
            tasks.push(Task::done(Message::FetchLatestVersion));
        }

        (
            Self {
                benchmark: Benchmark::empty(),
                background_benchmarks: Vec::new(),
                pins: HashMap::new(),
                displayed: None,
                filter_input: String::new(),
                popup: Popup::None,
                err_notif: None,
                display_update_available: false,
                window_id: None,
                settings: settings,
                last_opened,
                home_menu_hash: 0,
                stig_list_hash: 0,
                display_type: settings.default_display_type,
                filter_string: String::new(),

                main_col_opacity: 1.0,
                main_col_last_tick: None,
                popup_opacity: 1.0,
                popup_last_tick: None,
            },
            Task::batch(tasks),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)));

        if self.main_col_last_tick.is_some() || self.popup_last_tick.is_some() {
            let tick = window::frames().map(Message::Tick);
            Subscription::batch([keyboard, tick])
        } else {
            keyboard
        }
    }

    pub fn theme(&self) -> Theme {
        match self.settings.theme {
            AppTheme::Dark => THEME_DARK.clone(),
            AppTheme::Light => THEME_LIGHT.clone(),
            AppTheme::HighContrast => THEME_HIGH_CONTRAST.clone(),
            AppTheme::Coffee => THEME_COFFEE.clone(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InitWindow(id) => {
                let id = id.expect("Not able to retrieve window id.");
                self.window_id = Some(id);

                // Toggle window decorations and set the app icon.
                Task::batch(vec![
                    window::toggle_decorations(id),
                    window::set_resizable(id, true),
                    window::set_icon(
                        id,
                        from_file_data(APP_ICON, Some(ImageFormat::Png))
                            .expect("Could not load app icon!"),
                    ),
                ])
            }
            Message::WindowClose => iced::exit(),
            Message::WindowMin => {
                if let Some(id) = self.window_id {
                    window::minimize(id, true)
                } else {
                    Task::done(Message::SendErrNotif("Cant get window id to minimize."))
                }
            }
            Message::WindowFullscreenToggle => {
                if let Some(id) = self.window_id {
                    window::toggle_maximize(id)
                } else {
                    Task::done(Message::SendErrNotif(
                        "Cant get window id to toggle fullscreen.",
                    ))
                }
            }
            Message::WindowMove => {
                if let Some(id) = self.window_id {
                    window::drag(id)
                } else {
                    Task::done(Message::SendErrNotif("Cant get window id to move."))
                }
            }
            Message::WindowDragResize(dir) => {
                if let Some(id) = self.window_id {
                    window::drag_resize(id, dir)
                } else {
                    Task::done(Message::SendErrNotif("Cant get window id to resize."))
                }
            }

            Message::SwitchTheme(theme) => {
                self.settings.theme = theme;

                Task::done(Message::SaveSettings)
            }

            Message::FetchLatestVersion => Task::future(async {
                use crate::app::latest_release::is_latest_version;

                match is_latest_version() {
                    Some(true) => Message::DoNothing,
                    Some(false) => Message::SwitchDisplayUpdateAvailable(true),
                    None => Message::DoNothing, // Silently fail.
                }
            }),

            Message::SwitchDisplayUpdateAvailable(toggle) => {
                self.display_update_available = toggle;

                Task::none()
            }

            Message::OpenFile => Task::future(async move {
                let home_dir = dirs::home_dir();

                let home_dir = match home_dir {
                    Some(dir) => dir,
                    None => return Message::SendErrNotif("Home directory could not be found."),
                };

                let file_handle = AsyncFileDialog::new()
                    .add_filter("STIG", &["toml", "xml", "zip", "ckl", "cklb"])
                    .set_directory(home_dir)
                    .set_title("Xylok View - Select File")
                    .pick_file()
                    .await;

                // Do nothing if the user closed their file explorer before selecting a file.
                let file_handle = match file_handle {
                    Some(handle) => handle,
                    None => return Message::DoNothing,
                };

                let format = detect_stig_format(file_handle.path());

                match format {
                    Some(Format::Xylok(xylok_toml)) => {
                        let benchmark = xylok_toml.convert();

                        if let Some(benchmark) = benchmark {
                            Message::SwitchBenchmark(benchmark)
                        } else {
                            Message::SendErrNotif(
                                "Xylok toml could not be converted into a Benchmark.",
                            )
                        }
                    }

                    Some(Format::XccdfV1_1(file_str)) => {
                        let benchmark = load_v1_1(&file_str);

                        if let Some(benchmark) = benchmark {
                            Message::SwitchBenchmark(benchmark)
                        } else {
                            Message::SendErrNotif("Xml could not be converted into a Benchmark.")
                        }
                    }

                    Some(Format::XccdfV1_2) => {
                        Message::SendErrNotif("SCAP's are not a supported file type.")
                    }

                    Some(Format::CKL(file_str)) => {
                        let benchmarks = load_ckl(&file_str);

                        if benchmarks.is_empty() {
                            Message::SendErrNotif("CKL could not be converted into a Benchmark.")
                        } else {
                            Message::SwitchBenchmarks(benchmarks)
                        }
                    }

                    Some(Format::CKLB(cklb)) => {
                        let benchmarks = cklb.convert();

                        if benchmarks.is_empty() {
                            Message::SendErrNotif("CKLB could not be converted into a Benchmark.")
                        } else {
                            Message::SwitchBenchmarks(benchmarks)
                        }
                    }

                    None => Message::SendErrNotif("Selected file is an unsupported type."),
                }
            }),

            Message::Switch(id) => {
                // If the rule already displayed is being switched to, do nothing.
                if let Some(rule) = &self.displayed {
                    if rule.group_id == id {
                        return Task::none();
                    }
                }

                if let Some(rule) = self.benchmark.rules.get(&id) {
                    Task::done(Message::Display(rule.to_owned()))
                } else {
                    Task::done(Message::DoNothing)
                }
            }
            Message::SwitchBenchmark(benchmark) => {
                if let Some((name, _rule)) = benchmark.rules.first_key_value() {
                    let name = name.to_owned();

                    self.benchmark = benchmark;

                    // Reset pin values.
                    self.pins = HashMap::new();
                    // Reset background Benchmarks.
                    self.background_benchmarks = Vec::new();

                    // Remember when this was opened.
                    self.last_opened.insert(self.benchmark.id.clone());

                    // Benchmark has been switched to, so change tell the home menu to update,
                    // reflecting that this benchmark has been opened recently.
                    self.home_menu_hash += 1;

                    self.stig_list_hash += 1;

                    let tasks = vec![
                        Task::done(Message::Switch(name)),
                        Task::done(Message::SwitchPopup(Popup::Save)),
                    ];

                    Task::batch(tasks)
                } else {
                    // Do nothing when an attempting to switch an empty benchmark.
                    Task::none()
                }
            }
            Message::SwitchBenchmarks(mut benchmarks) => {
                if benchmarks.is_empty() {
                    return Task::none();
                }

                let first = benchmarks.remove(0);
                let mut tasks = vec![Task::done(Message::SwitchBenchmark(first))];

                for benchmark in benchmarks {
                    tasks.push(Task::done(Message::PushBackgroundBenchmark(benchmark)));
                }

                Task::batch(tasks)
            }
            Message::PushBackgroundBenchmark(benchmark) => {
                self.background_benchmarks.push(benchmark);

                Task::none()
            }
            Message::SwitchToBackground => {
                if self.background_benchmarks.is_empty() {
                    return Task::none();
                }

                // Get the benchmark that has been setting in the background for
                // the longest.
                let new_benchmark = self.background_benchmarks.remove(0);

                let old_benchmark = std::mem::replace(&mut self.benchmark, new_benchmark);
                self.background_benchmarks.push(old_benchmark);

                // Reset pin values when switching to this new benchmark.
                self.pins = HashMap::new();

                // Remember when this was opened.
                self.last_opened.insert(self.benchmark.id.clone());

                // Benchmark has been switched to, so change tell the home menu to update,
                // reflecting that this benchmark has been opened recently.
                self.home_menu_hash += 1;

                self.stig_list_hash += 1;

                Task::none()
            }

            Message::SetPins(pins) => {
                self.pins = pins;

                self.stig_list_hash += 1;

                // When the pins are set, check if the displayed rule has a filter applied.
                // If not, switch to the first one that does.

                // Get the displayed STIG, if its already pinned, dont switch which STIG is viewed.
                if let Some(rule) = &self.displayed {
                    let pin_status = self.pins.get(&rule.group_id);

                    match pin_status.unwrap_or(&Pinned::Not) {
                        Pinned::ByFilter => return Task::done(Message::DoNothing),
                        Pinned::ByFilterAndUser => return Task::done(Message::DoNothing),
                        _ => (), // Continue if not above options.
                    }
                }

                for (name, _rule) in self.benchmark.rules.iter() {
                    match self.pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::ByFilter => return Task::done(Message::Switch(name.to_owned())),
                        Pinned::ByFilterAndUser => {
                            return Task::done(Message::Switch(name.to_owned()));
                        }
                        _ => (),
                    }
                }

                Task::none()
            }
            Message::SwitchNext => {
                if let Some(displayed) = &self.displayed {
                    use std::ops::Bound::{Excluded, Unbounded};

                    let next = self
                        .benchmark
                        .rules
                        .range::<String, _>((Excluded(displayed.group_id.clone()), Unbounded))
                        .next()
                        .or_else(|| self.benchmark.rules.first_key_value());

                    if let Some((key, _)) = next {
                        return Task::done(Message::Switch(key.clone()));
                    }

                    Task::done(Message::DoNothing)
                } else {
                    Task::none()
                }
            }
            Message::Display(rule) => {
                self.displayed = Some(rule);

                // Only animate if configured to.
                if self.settings.animate {
                    self.main_col_opacity = MAIN_FADE_START;
                    self.main_col_last_tick = Some(Instant::now());
                }

                Task::none()
            }

            Message::SwitchPopup(popup) => {
                match (&self.popup, &popup) {
                    (Popup::Filter, Popup::Filter) => self.popup = Popup::None,
                    (Popup::Settings, Popup::Settings) => self.popup = Popup::None,
                    _ => {
                        if self.settings.animate && popup != Popup::None {
                            self.popup_opacity = POPUP_FADE_START;
                            self.popup_last_tick = Some(Instant::now());
                        }
                        self.popup = popup;
                    }
                }

                Task::none()
            }

            Message::SendErrNotif(err_str) => {
                if let None = self.err_notif {
                    self.err_notif = Some(err_str);
                }

                Task::none()
            }
            Message::ClearErrNotif => {
                self.err_notif = None;

                Task::none()
            }

            Message::Pin(id) => {
                let pin_status = self.pins.get(&id);

                match pin_status.unwrap_or(&Pinned::Not) {
                    Pinned::Not => {
                        let _ = self.pins.insert(id, Pinned::ByUser);
                    }
                    Pinned::ByUser => {
                        let _ = self.pins.insert(id, Pinned::Not);
                    }

                    Pinned::ByFilter => {
                        let _ = self.pins.insert(id, Pinned::ByFilterAndUser);
                    }
                    Pinned::ByFilterAndUser => {
                        let _ = self.pins.insert(id, Pinned::ByFilter);
                    }
                }

                self.stig_list_hash += 1;

                Task::none()
            }

            Message::FocusWidget(widget_id) => iced::widget::operation::focus(widget_id),

            Message::TypeCmd(filter_input) => {
                self.filter_input = filter_input;

                Task::none()
            }
            Message::ProcessCmd(command_str) => {
                let command = parse_command(&command_str);

                match command {
                    Some(command) => {
                        // If the command is a phrase, highlight that phrase.
                        // Otherwise, highlight nothing.
                        match command {
                            Command::Phrase(ref phrase) => {
                                self.filter_string = phrase.clone();
                            }
                            Command::Reset => self.filter_string = "".into(),
                        }

                        let new_pins = run_search_cmd(
                            command,
                            &self.benchmark,
                            std::mem::take(&mut self.pins),
                        );

                        match new_pins {
                            Some(new_pins) => Task::done(Message::SetPins(new_pins)),
                            None => {
                                Task::done(Message::SendErrNotif("Error when running the command."))
                            }
                        }
                    }
                    None => Task::none(),
                }
            }

            Message::KeyPressed(event) => match &event {
                keyboard::Event::KeyPressed {
                    key: key::Key::Character(key_smolstr),
                    modifiers,
                    ..
                } => match key_smolstr.as_str() {
                    "q" if modifiers.control() => return Task::done(Message::WindowClose),
                    "i" if modifiers.control() => return Task::done(Message::OpenFile),
                    "f" if modifiers.control() => {
                        return Task::done(Message::SwitchPopup(Popup::Filter));
                    }
                    _ => Task::none(),
                },

                keyboard::Event::KeyPressed {
                    key: key::Key::Named(key_name),
                    modifiers,
                    ..
                } => match key_name {
                    key::Named::Tab if modifiers.control() => Task::done(Message::SwitchNext),
                    _ => Task::none(),
                },

                _ => Task::none(),
            },

            Message::SaveSettings => {
                let err = &self.settings.save();

                match err {
                    Ok(_) => Task::none(),
                    Err(AppSettingsErr::CantSave(err_str)) => {
                        Task::done(Message::SendErrNotif(err_str))
                    }
                }
            }
            Message::SaveBenchmark => {
                let all = std::iter::once(&self.benchmark).chain(self.background_benchmarks.iter());

                for benchmark in all {
                    if let None = benchmark.save() {
                        return Task::done(Message::SendErrNotif("Couldn't save benchmark."));
                    }
                }

                // After saving, turn off the save menu.
                Task::done(Message::SwitchPopup(Popup::None))
            }

            Message::LoadCachedBenchmark(path) => match Benchmark::load(&path) {
                Some(benchmark) => {
                    if let Some((name, _rule)) = benchmark.rules.first_key_value() {
                        let name = name.to_owned();

                        self.benchmark = benchmark;

                        // Reset pin values.
                        self.pins = HashMap::new();
                        // Reset background Benchmarks.
                        self.background_benchmarks = Vec::new();

                        // Remember when this was opened.
                        self.last_opened.insert(self.benchmark.id.clone());

                        // Benchmark has been switched to, so change tell the home menu to update,
                        // reflecting that this benchmark has been opened recently.
                        self.home_menu_hash += 1;

                        self.stig_list_hash += 1;

                        Task::done(Message::Switch(name))
                    } else {
                        // Do nothing when an attempting to switch an empty benchmark.
                        Task::none()
                    }
                }
                None => Task::done(Message::SendErrNotif(
                    "Couldn't load cached benchmark. File version may be unsupported.",
                )),
            },
            Message::DeleteCachedBenchmark(path) => {
                let err = std::fs::remove_file(path);

                self.home_menu_hash += 1;

                if err.is_err() {
                    Task::done(Message::SendErrNotif("Couldn't delete cached benchmark."))
                } else {
                    // Benchmark has been deleted, so change tell the home menu to update,
                    // reflecting that this benchmark has been removed.

                    Task::none()
                }
            }

            Message::SwitchDisplayType(display_type) => {
                self.display_type = display_type;

                self.stig_list_hash += 1;

                Task::none()
            }
            // Instead of just switching display types, save it as the default for next time.
            Message::SaveDisplayType(display_type) => {
                self.display_type = display_type;
                self.settings.default_display_type = display_type;

                self.stig_list_hash += 1;

                Task::done(Message::SaveSettings)
            }

            Message::SaveAnimate(animate) => {
                self.settings.animate = animate;

                Task::done(Message::SaveSettings)
            }

            Message::SaveUpdateNotify(notify) => {
                self.settings.notify_if_update = notify;

                Task::done(Message::SaveSettings)
            }

            Message::ReturnHome => {
                self.benchmark = Benchmark::empty();
                self.background_benchmarks = Vec::new();
                self.displayed = None;

                Task::none()
            }

            Message::Tick(now) => {
                if let Some(last) = self.main_col_last_tick {
                    let delta_t = now.duration_since(last).as_secs_f32();

                    self.main_col_opacity =
                        (self.main_col_opacity + delta_t / MAIN_FADE_DURATION_SECS).min(1.0);

                    if self.main_col_opacity >= 1.0 {
                        self.main_col_last_tick = None;
                    } else {
                        self.main_col_last_tick = Some(now);
                    }
                }

                if let Some(last) = self.popup_last_tick {
                    let delta_t = now.duration_since(last).as_secs_f32();

                    self.popup_opacity =
                        (self.popup_opacity + delta_t / POPUP_FADE_DURATION_SECS).min(1.0);

                    if self.popup_opacity >= 1.0 {
                        self.popup_last_tick = None;
                    } else {
                        self.popup_last_tick = Some(now);
                    }
                }

                Task::none()
            }

            Message::DoNothing => Task::none(),

            Message::OpenURL(url) => {
                let _ = open::that(url);

                Task::none()
            }
        }
    }

    pub fn load_cache() -> Vec<std::path::PathBuf> {
        let Some(mut cache_dir) = dirs::cache_dir() else {
            return Vec::new();
        };

        cache_dir.push("xylok-view/");

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                if name.ends_with(".msgpack.zstd") && name != ".msgpack.zstd" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }
}
