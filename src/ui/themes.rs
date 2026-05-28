use iced::{
    Theme, color,
    theme::{Custom, Palette},
};
use std::sync::{Arc, LazyLock};

pub static THEME_DARK: LazyLock<Theme> = LazyLock::new(|| {
    Theme::Custom(Arc::new(Custom::new(
        "Custom Dark".to_string(),
        Palette {
            background: color!(0x1B1C1C),
            text: color!(0xE6E6E6),
            primary: color!(0xA2A2D0),
            success: color!(0x22A67A),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        },
    )))
});

pub static THEME_LIGHT: LazyLock<Theme> = LazyLock::new(|| {
    Theme::Custom(Arc::new(Custom::new(
        "Custom Light".to_string(),
        Palette {
            background: color!(0xF4F4F6),
            text: color!(0x1E1A2E),
            primary: color!(0x5A5A8E),
            success: color!(0x0E9E6A),
            warning: color!(0xE07B00),
            danger: color!(0xC0393A),
        },
    )))
});

pub static THEME_HIGH_CONTRAST: LazyLock<Theme> = LazyLock::new(|| {
    Theme::Custom(Arc::new(Custom::new(
        "High Contrast".to_string(),
        Palette {
            background: color!(0x181818),
            text: color!(0xFFFFFF),
            primary: color!(0xFFD700),
            success: color!(0x00FF7F),
            warning: color!(0xFF8C00),
            danger: color!(0xFF3333),
        },
    )))
});

pub static THEME_COFFEE: LazyLock<Theme> = LazyLock::new(|| {
    Theme::Custom(Arc::new(Custom::new(
        "Coffee".to_string(),
        Palette {
            background: color!(0x1A1714),
            text: color!(0xC8BAA8),
            primary: color!(0x9E7840),
            success: color!(0x5A7A4E),
            warning: color!(0xC49A18),
            danger: color!(0xAC4444),
        },
    )))
});
