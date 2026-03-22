pub mod ai_view;
pub mod icons;
pub mod builtin_apps;
pub mod theme_loader;
pub mod app_shell;
pub mod context_menu;
pub mod db;
pub mod spinner;
pub mod widgets;
pub mod desktop;
pub mod header;
pub mod help_view;
pub mod launcher;
pub mod multiwindow;
pub mod notification;
pub mod sidebar;
pub mod split_view;
pub mod taskbar;
pub mod wallpaper;
pub mod system_info;
pub mod web_desktop;
pub mod window;
pub mod window_frame;

pub use ai_view::AiApp;
pub use app_shell::{AppMode, AppShell, LayoutA, LayoutB, LayoutC, ScreenWrapper};
pub use desktop::Desktop;
pub use header::{Breadcrumb, ShellHeader};
pub use help_view::{HelpApp, HelpSidebarPanel};
pub use launcher::{AppLauncher, LauncherState};
pub use multiwindow::{MultiwindowHandle, use_multiwindow};
pub use context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
pub use notification::{Notification, NotificationHistory, NotificationKind, NotificationManager, NotificationStack};
pub use sidebar::{ShellSidebar, SidebarSection, SidebarNavItem};
pub use split_view::{SplitState, SplitView};
pub use taskbar::Taskbar;
pub use web_desktop::{WebDesktop, WebTaskbarState};
pub use window::{Window, WindowButton, WindowContent, WindowId, WindowManager, WindowSize};

// Re-export desktop launch abstraction so fs-app can use fs_shell::launch_desktop.
#[cfg(feature = "desktop")]
pub use fs_components::{launch_desktop, spawn_window, DesktopConfig};
pub use window_frame::WindowFrame;
pub use spinner::{LoadingOverlay, LoadingSpinner, SpinnerSize};
pub use system_info::{Architecture, Platform, RunMode, SystemInfo, SYSTEM_INFO};
pub use widgets::{
    ClockWidget, SystemInfoWidget, QuickNotesWidget, PlaceholderWidget,
    WidgetKind, WidgetSlot, render_widget, load_widget_layout, save_widget_layout,
};
