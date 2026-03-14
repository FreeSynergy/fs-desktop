/// Multiwindow support — open any fsd-* app in its own native OS window.
///
/// Uses Dioxus 0.6 desktop `window().new_window()` to spawn independent windows.
/// Each window has its own Dioxus tree but shares no state with the main shell.
///
/// # Usage
/// ```rust,ignore
/// // In a Dioxus component (desktop feature only):
/// let handle = use_multiwindow();
/// handle.open_conductor();
/// ```
#[cfg(feature = "desktop")]
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

/// Handle returned by `use_multiwindow()`. Call methods to open app windows.
#[derive(Clone)]
pub struct MultiwindowHandle;

impl MultiwindowHandle {
    #[cfg(feature = "desktop")]
    fn open(&self, title: &str, width: f64, height: f64, component: fn() -> Element) {
        let cfg = Config::new().with_window(
            WindowBuilder::new()
                .with_title(title)
                .with_inner_size(LogicalSize::new(width, height))
                .with_resizable(true),
        );
        dioxus::desktop::window().new_window(VirtualDom::new(component), cfg);
    }

    /// Open fsd-conductor in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_conductor(&self) {
        self.open("FreeSynergy — Container Manager", 1000.0, 700.0, fsd_conductor::ConductorApp);
    }

    /// Open fsd-settings in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_settings(&self) {
        self.open("FreeSynergy — Settings", 800.0, 640.0, fsd_settings::SettingsApp);
    }

    /// Open fsd-profile in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_profile(&self) {
        self.open("FreeSynergy — Profile", 700.0, 600.0, fsd_profile::ProfileApp);
    }

    /// Open fsd-store in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_store(&self) {
        self.open("FreeSynergy — App Store", 1000.0, 700.0, fsd_store::StoreApp);
    }

    /// Open fsd-studio in its own window.
    #[cfg(feature = "desktop")]
    pub fn open_studio(&self) {
        self.open("FreeSynergy — Studio", 1000.0, 700.0, fsd_studio::StudioApp);
    }

    // Non-desktop stub: no-ops so code compiles for web target too.
    #[cfg(not(feature = "desktop"))]
    pub fn open_conductor(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_settings(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_profile(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_store(&self) {}
    #[cfg(not(feature = "desktop"))]
    pub fn open_studio(&self) {}
}

/// Hook that returns a `MultiwindowHandle`.
pub fn use_multiwindow() -> MultiwindowHandle {
    MultiwindowHandle
}
