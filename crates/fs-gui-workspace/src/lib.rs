#![deny(clippy::all, clippy::pedantic, warnings)]
pub mod ai_view;
pub mod app_lifecycle;
pub mod app_shell;
pub mod builtin_apps;
pub mod capability_observer;
pub mod context_menu;
pub mod corner_menus;
pub mod db;
pub mod header;
pub mod help_sidebar;
pub mod help_view;
pub mod icons;
pub mod launcher;
pub mod multiwindow;
pub mod notification;
pub mod search_component;
pub mod shell;
pub mod shell_layout;
pub mod sidebar;
pub mod sidebar_state;
pub mod spinner;
pub mod split_view;
pub mod system_info;
pub mod taskbar;
pub mod theme_loader;
pub mod view;
pub mod wallpaper;
pub mod web_desktop;
pub mod widgets;
pub mod window;
pub mod window_frame;

// ── App-level i18n init ───────────────────────────────────────────────────────

/// Initialize global i18n at app startup.
///
/// Call **once from `main()`** — before any iced rendering — so that all
/// translation keys are resolved before the first frame is painted.
///
/// All shell/profile strings are bundled in fs-i18n via `desktop.ftl` (FTL format).
pub fn init_i18n() {
    let lang = fs_settings::load_active_language();

    if let Err(e) = fs_i18n::init_with_builtins(&lang) {
        if !e.to_string().contains("already initialized") {
            tracing::error!("i18n init failed: {e}");
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

pub use app_lifecycle::{AppLifecycleBus, AppLifecycleEvent, AppLifecycleObserver};
pub use app_shell::AppMode;
pub use capability_observer::CapabilityObserver;
pub use context_menu::{ContextMenuItem, ContextMenuState};
pub use corner_menus::{AiMenu, HelpMenu, SettingsMenu, TasksMenu};
pub use header::{Breadcrumb, HeaderState};
pub use help_sidebar::{
    ActiveWindowObserver, AiHelpSource, CapabilityCheck, HelpContent, HelpSidebarState, HelpSource,
    LocalHelpTopicSource, NoHelpSource,
};
pub use help_view::{HelpApp, HelpSidebarPanel};
pub use launcher::LauncherState;
pub use multiwindow::{use_multiwindow, MultiwindowHandle};
pub use notification::{Notification, NotificationHistory, NotificationKind, NotificationManager};
pub use shell::{DesktopMessage, DesktopShell};
pub use shell_layout::{ShellLayout, ShellSection, SlotEntry};
pub use sidebar::{
    default_pinned_items, default_sidebar_sections, ManagerBundle, SidebarEntry, SidebarItem,
    SidebarSection,
};
pub use sidebar_state::{
    MouseProximityObserver, SidebarMode, SidebarSide, SidebarState, SidebarTransition,
};
pub use spinner::{LoadingOverlay, LoadingSpinner, SpinnerSize};
pub use split_view::SplitState;
pub use system_info::{Architecture, Platform, RunMode, SystemInfo};
pub use taskbar::AppEntry;
pub use web_desktop::WebTaskbarState;
pub use widgets::{load_widget_layout, save_widget_layout, WidgetKind, WidgetSlot};
pub use window::{
    AppId, FsWindow, OpenWindow, Window, WindowButton, WindowContent, WindowHost, WindowId,
    WindowManager, WindowRenderFn, WindowSidebarItem, WindowSize,
};
pub use window_frame::{MinimizedWindowIcon, WindowFrame};
