// Makes a terminal not appear when the windows app is launched.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

/// Contains all of the app logic, which is the apps state and functionality.
mod app;
/// Contains all logic for parsing files into interpretable logic.
mod parse;
/// Contains all of the ui code, such as styles and ui elements.
/// Contains some logic, but only ui logic.
mod ui;
/// Custom iced widgets.
mod widgets;

use crate::app::App;

#[cfg(not(target_os = "linux"))]
fn main() -> iced::Result {
    use iced::Font;

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .font(include_bytes!("../assets/fonts/font.ttf"))
        .default_font(Font::with_name("CMU Sans Serif"))
        .run()
}

#[cfg(target_os = "linux")]
fn main() -> iced::Result {
    use iced::{
        Font,
        window::settings::{PlatformSpecific, Settings},
    };

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .font(include_bytes!("../assets/fonts/font.ttf"))
        .default_font(Font::with_name("CMU Sans Serif"))
        .window(Settings {
            platform_specific: PlatformSpecific {
                application_id: String::from("io.github.joshuardecker.stig-view"),
                override_redirect: false,
            },
            ..Settings::default()
        })
        .run()
}
