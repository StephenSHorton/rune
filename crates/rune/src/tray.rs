//! A system tray icon with a flat menu: StatusNotifierItem and `com.canonical.dbusmenu` over
//! zbus on Linux, `Shell_NotifyIcon` on its own message-only window on Windows. Each tray runs
//! on its own thread, so it needs no event loop of the app's and answers while the app's window
//! is closed. A primary click sends [`TrayEvent::Activate`]; a secondary click opens the menu,
//! whose picks send [`TrayEvent::Menu`]. Dropping the [`Tray`] removes the icon.
//!
//! Linux shows the icon only where a tray host runs (KDE, waybar, most bars; GNOME needs an
//! extension). Without one, [`Tray::new`] still succeeds and the icon appears once a host
//! starts. The Linux backend needs the default `dbus` feature. macOS isn't supported yet.
#[cfg(any(all(target_os = "linux", feature = "dbus"), target_os = "windows"))]
use std::sync::Arc;

#[cfg(all(target_os = "linux", feature = "dbus"))]
#[path = "tray/sni.rs"]
mod native;
#[cfg(target_os = "windows")]
#[path = "tray/win32.rs"]
mod native;

/// The icon as straight (not premultiplied) RGBA rows, `width × height × 4` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayIcon { pub(crate) width: u32, pub(crate) height: u32, pub(crate) rgba: Vec<u8> }
impl TrayIcon {
    pub fn rgba(width: u32, height: u32, pixels: Vec<u8>) -> Self { Self { width, height, rgba: pixels } }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrayItem {
    /// `id` comes back in [`TrayEvent::Menu`] when it's picked.
    Action { id: String, label: String, enabled: bool },
    /// An item with a check mark beside it while `checked` (the platform's own mark, not a
    /// glyph in the label). Picking it sends [`TrayEvent::Menu`] with `id` and changes nothing
    /// by itself: the app owns the state, flips it, and passes the new menu to
    /// [`Tray::set_menu`].
    Check { id: String, label: String, checked: bool, enabled: bool },
    Separator,
}
impl TrayItem {
    pub fn action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Action { id: id.into(), label: label.into(), enabled: true }
    }
    /// A check item, enabled, checked or not.
    ///
    /// ```
    /// use rune::TrayItem;
    /// let launch_at_login = true;
    /// let item = TrayItem::check("login", "Open at Login", launch_at_login);
    /// assert_eq!(item.is_checked(), Some(true));
    /// ```
    pub fn check(id: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self::Check { id: id.into(), label: label.into(), checked, enabled: true }
    }
    pub fn enabled(self, enabled: bool) -> Self {
        match self {
            Self::Action { id, label, .. } => Self::Action { id, label, enabled },
            Self::Check { id, label, checked, .. } => Self::Check { id, label, checked, enabled },
            separator => separator,
        }
    }
    /// Whether a check item is checked; `None` for other items.
    pub fn is_checked(&self) -> Option<bool> {
        match self { Self::Check { checked, .. } => Some(*checked), _ => None }
    }
    /// The label, for items that show one.
    #[cfg_attr(not(any(all(target_os = "linux", feature = "dbus"), target_os = "windows")), allow(dead_code))]
    pub(crate) fn label(&self) -> Option<&str> {
        match self { Self::Action { label, .. } | Self::Check { label, .. } => Some(label), Self::Separator => None }
    }
    /// Whether the item can be picked.
    #[cfg_attr(not(any(all(target_os = "linux", feature = "dbus"), target_os = "windows")), allow(dead_code))]
    pub(crate) fn is_enabled(&self) -> bool {
        match self { Self::Action { enabled, .. } | Self::Check { enabled, .. } => *enabled, Self::Separator => false }
    }
    /// The id a pick of this item sends, when it can be picked.
    #[cfg_attr(not(any(all(target_os = "linux", feature = "dbus"), target_os = "windows")), allow(dead_code))]
    pub(crate) fn picked(&self) -> Option<&str> {
        match self {
            Self::Action { id, enabled: true, .. } | Self::Check { id, enabled: true, .. } => Some(id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrayEvent {
    /// The icon's primary click. `token` is the XDG activation token a Wayland tray host hands
    /// over with it, for showing a window with focus.
    Activate { token: Option<String> },
    /// A menu item's `id`.
    Menu(String),
}

#[derive(Clone, Debug)]
#[cfg_attr(not(all(target_os = "linux", feature = "dbus")), allow(dead_code))]
pub struct TraySpec {
    /// A stable application id (Linux `Id`).
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) tooltip: String,
    pub(crate) icon: TrayIcon,
    pub(crate) menu: Vec<TrayItem>,
}
impl TraySpec {
    pub fn new(id: impl Into<String>, title: impl Into<String>, icon: TrayIcon) -> Self {
        Self { id: id.into(), title: title.into(), tooltip: String::new(), icon, menu: Vec::new() }
    }
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self { self.tooltip = tooltip.into(); self }
    pub fn menu(mut self, menu: Vec<TrayItem>) -> Self { self.menu = menu; self }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrayError { Unsupported, Backend(String) }
impl std::fmt::Display for TrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Unsupported => f.write_str("no system tray on this platform"), Self::Backend(e) => f.write_str(e) }
    }
}
impl std::error::Error for TrayError {}

/// Runs on the tray's thread; keep it short and hand work to the app.
#[cfg(any(all(target_os = "linux", feature = "dbus"), target_os = "windows"))]
pub(crate) type Handler = Arc<dyn Fn(TrayEvent) + Send + Sync>;

pub struct Tray {
    #[cfg(any(all(target_os = "linux", feature = "dbus"), target_os = "windows"))]
    native: native::Native,
}

impl Tray {
    pub fn new(spec: TraySpec, on_event: impl Fn(TrayEvent) + Send + Sync + 'static) -> Result<Self, TrayError> {
        if spec.icon.rgba.len() != spec.icon.width as usize * spec.icon.height as usize * 4 {
            return Err(TrayError::Backend("tray icon size doesn't match its pixels".into()));
        }
        #[cfg(any(all(target_os = "linux", feature = "dbus"), target_os = "windows"))]
        { native::Native::new(spec, Arc::new(on_event)).map(|native| Self { native }) }
        #[cfg(not(any(all(target_os = "linux", feature = "dbus"), target_os = "windows")))]
        { let _ = (spec, on_event); Err(TrayError::Unsupported) }
    }
    pub fn set_menu(&self, menu: Vec<TrayItem>) {
        #[cfg(any(all(target_os = "linux", feature = "dbus"), target_os = "windows"))]
        self.native.set_menu(menu);
        #[cfg(not(any(all(target_os = "linux", feature = "dbus"), target_os = "windows")))]
        let _ = menu;
    }
    pub fn set_tooltip(&self, tooltip: impl Into<String>) {
        #[cfg(any(all(target_os = "linux", feature = "dbus"), target_os = "windows"))]
        self.native.set_tooltip(tooltip.into());
        #[cfg(not(any(all(target_os = "linux", feature = "dbus"), target_os = "windows")))]
        let _ = tooltip;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_leaves_separators_alone() {
        assert_eq!(TrayItem::Separator.enabled(false), TrayItem::Separator);
        assert_eq!(TrayItem::action("q", "Quit").enabled(false), TrayItem::Action { id: "q".into(), label: "Quit".into(), enabled: false });
    }

    #[test]
    fn a_check_item_keeps_its_state_through_enabled_and_picks_only_when_enabled() {
        let on = TrayItem::check("login", "Open at Login", true);
        assert_eq!(on.is_checked(), Some(true));
        assert_eq!(on.label(), Some("Open at Login"));
        assert_eq!(on.picked(), Some("login"));
        let off = TrayItem::check("login", "Open at Login", false).enabled(false);
        assert_eq!(off, TrayItem::Check { id: "login".into(), label: "Open at Login".into(), checked: false, enabled: false });
        assert_eq!(off.picked(), None);
        assert!(!off.is_enabled());
        assert_eq!(TrayItem::action("q", "Quit").is_checked(), None);
        assert_eq!(TrayItem::Separator.picked(), None);
    }

    #[test]
    fn a_short_icon_is_refused() {
        let spec = TraySpec::new("t", "T", TrayIcon::rgba(2, 2, vec![0; 3]));
        assert!(matches!(Tray::new(spec, |_| {}), Err(TrayError::Backend(_))));
    }
}
