/// Multiwindow support — open any fs-* app in its own native OS window.
///
/// Uses `spawn_window` from `fs_components::launch` to spawn independent windows.
/// Each window has its own Dioxus tree but shares no state with the main shell.
/// All dioxus::desktop API calls are isolated in `fs_components::launch`.
///
/// # Usage
/// ```rust,ignore
/// let handle = use_multiwindow();
/// handle.open_settings();
/// ```
#[cfg(feature = "desktop")]
use fs_components::{spawn_window, DesktopConfig};

/// Handle returned by `use_multiwindow()`. Call methods to open app windows.
#[derive(Clone)]
pub struct MultiwindowHandle;

impl MultiwindowHandle {
    /// Open fs-managers in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_managers(&self) {
        spawn_window(
            DesktopConfig::new().with_title("FreeSynergy \u{2014} Managers").with_size(900.0, 640.0),
            fs_managers::ManagersApp,
        );
    }

    /// Open fs-settings in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_settings(&self) {
        spawn_window(
            DesktopConfig::new().with_title("FreeSynergy \u{2014} Settings").with_size(800.0, 640.0),
            fs_settings::SettingsApp,
        );
    }

    /// Open fs-profile in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_profile(&self) {
        spawn_window(
            DesktopConfig::new().with_title("FreeSynergy \u{2014} Profile").with_size(700.0, 600.0),
            fs_profile::ProfileApp,
        );
    }

    /// Open fs-store in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_store(&self) {
        spawn_window(
            DesktopConfig::new().with_title("FreeSynergy \u{2014} App Store").with_size(1000.0, 700.0),
            fs_store_app::StoreApp,
        );
    }

    /// Open fs-builder in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_builder(&self) {
        spawn_window(
            DesktopConfig::new().with_title("FreeSynergy \u{2014} Builder").with_size(1000.0, 700.0),
            fs_builder::BuilderApp,
        );
    }

    // Non-desktop stubs: no-ops so code compiles for web target too.
    #[cfg(not(feature = "desktop"))]
    pub fn open_managers(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_settings(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_profile(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_store(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_builder(&self) {}
}

/// Hook that returns a `MultiwindowHandle`.
pub fn use_multiwindow() -> MultiwindowHandle {
    MultiwindowHandle
}
