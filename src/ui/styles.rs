use iced::{
    Border, Shadow, Theme, border,
    border::Radius,
    color,
    widget::{button, container, svg, toggler},
};

const BORDER_RAD: f32 = 8.0;

/// A rounded button in the primary color.
pub fn rounded_primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.base.color.into()),
            text_color: palette.primary.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.primary.strong.color.into()),
            text_color: palette.primary.strong.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A rounded button in the success color.
pub fn rounded_success_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.success.weak.color.into()),
            text_color: palette.danger.weak.text, // Makes this match the danger button.
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.success.base.color.into()),
            text_color: palette.danger.base.text, // Makes this match the danger button.
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A rounded button in the danger color.
pub fn rounded_danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.danger.weak.color.into()),
            text_color: palette.danger.weak.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.danger.base.color.into()),
            text_color: palette.danger.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A rounded button in a less obvious background color.
pub fn rounded_boring_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.weak.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: palette.background.strong.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A button thats invisible unless hovered over, then its dark theme.
/// Also makes its text color the primary color.
pub fn rounded_dark_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weakest.color.into()),
            text_color: palette.background.weakest.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(color!(0, 0, 0, 0.0).into()),
            text_color: palette.primary.base.color,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A button that is not visible.
pub fn no_button(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        background: Some(color!(0, 0, 0, 0.0).into()),
        text_color: palette.background.base.text,
        border: Border {
            ..border::rounded(BORDER_RAD)
        },
        shadow: Shadow {
            ..Shadow::default()
        },
        snap: false,
    }
}

/// A button with only the right corners rounded, for use next to the filter accent strip.
pub fn rounded_boring_button_right(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.weak.text,
            border: Border {
                radius: Radius {
                    top_left: 0.0,
                    top_right: BORDER_RAD,
                    bottom_right: BORDER_RAD,
                    bottom_left: 0.0,
                },

                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: palette.background.strong.text,
            border: Border {
                radius: Radius {
                    top_left: 0.0,
                    top_right: BORDER_RAD,
                    bottom_right: BORDER_RAD,
                    bottom_left: 0.0,
                },
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

/// A thin accent strip indicating a filter match.
pub fn filter_accent(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.primary.base.color.into()),
        border: Border {
            radius: Radius {
                top_left: 0.0,
                top_right: BORDER_RAD,
                bottom_right: BORDER_RAD,
                bottom_left: 0.0,
            },
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// A svg with the primary color.
pub fn colored_svg(theme: &Theme, status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    match status {
        svg::Status::Hovered => svg::Style {
            color: Some(palette.background.base.text),
        },
        _ => svg::Style {
            color: Some(palette.primary.base.color),
        },
    }
}

/// A svg with the background color.
pub fn boring_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.background.base.text),
    }
}

/// A svg with the success color.
pub fn good_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.success.base.color),
    }
}

/// A svg with the danger color.
pub fn bad_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.danger.base.color),
    }
}

/// A svg with the warning color.
pub fn warning_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.warning.base.color),
    }
}

/// A rounded container to place elements into, lives in the backgound.
pub fn background_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.weakest.text),
        background: Some(palette.background.weakest.color.into()),
        border: Border {
            color: palette.background.weakest.color,
            width: 2.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            ..Shadow::default()
        },
        snap: false,
    }
}

/// The container style the cmd prompt has.
pub fn cmd_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.strong.text),
        background: Some(palette.background.strong.color.into()),
        border: Border {
            color: palette.background.weak.color,
            width: 0.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            color: palette.background.base.color,
            offset: iced::Vector::ZERO,
            blur_radius: 8.0,
        },
        snap: false,
    }
}

/// The container style the err notification has.
pub fn err_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.danger.base.text),
        background: Some(palette.danger.base.color.into()),
        border: Border {
            color: palette.danger.base.color,
            width: 0.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            color: palette.background.base.color,
            offset: iced::Vector::ZERO,
            blur_radius: 8.0,
        },
        snap: false,
    }
}

/// The style the display update available container uses.
/// Looks nice in the window decorations.
pub fn update_available_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(palette.background.base.color.into()),
        border: Border {
            color: palette.secondary.base.color,
            width: 2.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            ..Shadow::default()
        },
        snap: false,
    }
}

/// An overlay container that fades content in. Pass `1.0 - main_col_opacity` as the alpha.
/// Used in a stack on top of content to simulate fade-in since iced 0.14 has no general opacity widget.
pub fn fade_overlay(alpha: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme: &Theme| {
        let palette = theme.extended_palette();
        let mut bg = palette.background.weakest.color;
        bg.a = alpha;
        container::Style {
            background: Some(bg.into()),
            ..container::Style::default()
        }
    }
}

/// Default theme for togglers does not look good in the settings menu.
pub fn toggler_theme(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let palette = theme.extended_palette();

    match status {
        toggler::Status::Active { is_toggled: true } => toggler::Style {
            background: palette.background.weakest.color.into(),
            background_border_width: 2.0,
            background_border_color: palette.background.strong.color,
            foreground: palette.primary.base.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: palette.primary.base.color,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
        },
        toggler::Status::Active { is_toggled: false } => toggler::Style {
            background: palette.background.weakest.color.into(),
            background_border_width: 2.0,
            background_border_color: palette.background.strong.color,
            foreground: palette.primary.base.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: palette.primary.base.color,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
        },
        toggler::Status::Hovered { is_toggled: true } => toggler::Style {
            background: palette.background.weakest.color.into(),
            background_border_width: 2.0,
            background_border_color: palette.background.strong.color,
            foreground: palette.primary.weak.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: palette.primary.weak.color,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
        },
        toggler::Status::Hovered { is_toggled: false } => toggler::Style {
            background: palette.background.weakest.color.into(),
            background_border_width: 2.0,
            background_border_color: palette.background.strong.color,
            foreground: palette.primary.weak.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: palette.primary.weak.color,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
        },
        toggler::Status::Disabled { is_toggled: true } => toggler::Style {
            background: palette.background.weakest.color.into(),
            background_border_width: 2.0,
            background_border_color: palette.background.weak.color,
            foreground: palette.background.strong.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: palette.background.strong.color,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
        },
        toggler::Status::Disabled { is_toggled: false } => toggler::Style {
            background: palette.background.weakest.color.into(),
            background_border_width: 2.0,
            background_border_color: palette.background.weak.color,
            foreground: palette.background.strong.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: palette.background.strong.color,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
        },
    }
}
