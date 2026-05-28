/// Contains the app internal code.
mod app;
/// Detect whether the user is running the latest release.
mod latest_release;
/// One-time migration from legacy "stig-view" paths.
mod migrate;
/// Contains search logic.
mod search;
/// Contains settings logic, like saving to the disk.
mod settings;
/// Contains the logic for remembering when benchmarks were last opened, and saving this to the disk.
mod time_opened;

use iced::{Task, keyboard, widget::Id, window, window::Direction};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};

use crate::app::{
    settings::{AppSettings, AppSettingsErr},
    time_opened::TimeLastOpened,
};
use crate::parse::{Benchmark, Rule};

/// The overarching state of the application.
#[derive(Debug, Clone)]
pub struct App {
    /// Currently displayed benchmark.
    pub benchmark: Benchmark,
    /// Benchmarks that live in the background, but are not currently displayed.
    pub background_benchmarks: Vec<Benchmark>,
    /// What rules are pinned, and why the are pinned.
    pub pins: HashMap<String, Pinned>,
    /// The currently displayed rule.
    pub displayed: Option<Rule>,
    /// The text input for the user to type filters into.
    pub filter_input: String,
    /// The current popup being displayed.
    pub popup: Popup,
    /// Error notification text to be displayed.
    pub err_notif: Option<&'static str>,
    /// If true, display to the user there is an update available.
    pub display_update_available: bool,
    /// The internal id of the window.
    pub window_id: Option<window::Id>,
    /// Settings applied to the app.
    pub settings: AppSettings,
    /// When benchmarks were last opened by the user.
    pub last_opened: TimeLastOpened,
    /// A counter that changes whenever the home menu ui should be refreshed.
    pub home_menu_hash: u64,
    /// A counter that changes whenever the rules list ui should be refreshed.
    pub stig_list_hash: u64,
    /// What data should be displayed in the rules list.
    pub display_type: DisplayType,
    /// The keyword / phrase the user is searching for.
    pub filter_string: String,

    /// The opacity of the main element, the data of the current rule.
    pub main_col_opacity: f32,
    /// How long its been since the last time the opacity of the main element has changed.
    pub main_col_last_tick: Option<Instant>,
    /// The opacity of any popup.
    pub popup_opacity: f32,
    /// How long its been since the last time the opacity of the popup element has changed.
    pub popup_last_tick: Option<Instant>,
}

/// Popups that can appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popup {
    Filter,
    Settings,
    Save,
    None,
}

/// Every way to change the state.
#[derive(Debug, Clone)]
pub enum Message {
    InitWindow(Option<window::Id>),
    WindowClose,
    WindowMin,
    WindowFullscreenToggle,
    WindowMove,
    WindowDragResize(Direction),

    SwitchTheme(AppTheme),

    FetchLatestVersion,
    SwitchDisplayUpdateAvailable(bool),

    OpenFile,

    Switch(String),
    SwitchBenchmark(Benchmark),
    SwitchBenchmarks(Vec<Benchmark>),
    PushBackgroundBenchmark(Benchmark),
    // Switch the current Benchmark to one loaded in the background.
    // Puts the current Benchmark into the background.
    SwitchToBackground,

    SetPins(HashMap<String, Pinned>),
    SwitchNext,
    Display(Rule),

    SwitchPopup(Popup),

    SendErrNotif(&'static str),
    ClearErrNotif,

    Pin(String),

    FocusWidget(Id),

    TypeCmd(String),
    ProcessCmd(String),

    KeyPressed(keyboard::Event),

    SaveSettings,
    SaveBenchmark,
    LoadCachedBenchmark(std::path::PathBuf),
    DeleteCachedBenchmark(std::path::PathBuf),

    SwitchDisplayType(DisplayType),
    SaveDisplayType(DisplayType),

    SaveAnimate(bool),
    SaveUpdateNotify(bool),

    ReturnHome,

    Tick(Instant),

    DoNothing,

    OpenURL(&'static str),
}

/// The color theme of the app.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    HighContrast,
    Coffee,
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
            AppTheme::HighContrast => "High Contrast",
            AppTheme::Coffee => "Coffee",
        })
    }
}

/// Whether the stig has been pinned in the list for any reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
    ByFilterAndUser,
}

/// What name should be displayed on the buttons that switch the displayed STIG.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Deserialize, Serialize)]
pub enum DisplayType {
    GroupId,
    RuleId,
    STIGId,
}

impl std::fmt::Display for DisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DisplayType::GroupId => "Group ID",
            DisplayType::RuleId => "Rule ID",
            DisplayType::STIGId => "STIG ID",
        })
    }
}
