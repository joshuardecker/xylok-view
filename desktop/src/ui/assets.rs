use iced::widget::svg::Handle;
use std::sync::LazyLock;

pub static DOWN_TICK: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/down-tick.svg")));
pub static CROSS: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/cross.svg")));
pub static SQUARE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/square.svg")));
pub static BOOKMARK: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/bookmark.svg")));
pub static FILLED_BOOKMARK: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../../assets/images/bookmark-filled.svg"))
});
pub static SETTINGS: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/settings.svg")));
pub static SWITCH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/switch.svg")));
pub static FILE_ICON: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/file.svg")));
pub static CHECKED_CIRCLE: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../../assets/images/check-circle.svg"))
});
pub static CROSS_CIRCLE: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../../assets/images/cross-circle.svg"))
});
pub static MINUS_CIRCLE: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../../assets/images/minus-circle.svg"))
});
pub static HOME: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/home.svg")));
pub static REFRESH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/refresh.svg")));
pub static TRASH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../../assets/images/trash.svg")));

/// Just the bytes of the app icon png file.
pub static APP_ICON: &[u8] = include_bytes!("../../../assets/logo/logo-1024.png");
