#![deny(clippy::all, clippy::pedantic, warnings)]
pub mod ai_view;
pub mod app_shell;
pub mod builtin_apps;
pub mod context_menu;
pub mod db;
pub mod desktop;
pub mod header;
pub mod help_view;
pub mod icons;
pub mod launcher;
pub mod multiwindow;
pub mod notification;
pub mod sidebar;
pub mod spinner;
pub mod split_view;
pub mod system_info;
pub mod taskbar;
pub mod theme_loader;
pub mod wallpaper;
pub mod web_desktop;
pub mod widgets;
pub mod window;
pub mod window_frame;

// ── i18n plugin for shell strings (shell.*, profile.*) ───────────────────────

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

struct ShellI18nPlugin;

impl fs_i18n::SnippetPlugin for ShellI18nPlugin {
    fn name(&self) -> &'static str {
        "fs-gui-workspace"
    }
    fn snippets(&self) -> &[(&str, &str)] {
        I18N_SNIPPETS
    }
}

// ── App-level i18n init ───────────────────────────────────────────────────────

/// Initialize global i18n at app startup.
///
/// Call **once from `main()`** — before `launch_desktop` — so that all translation
/// keys are resolved before any Dioxus component renders.  Calling from inside a
/// component (`use_context_provider`, `use_hook`, …) has no downside for simple apps
/// but creates a window in which early renders can see raw keys.
///
/// # What this does
/// 1. Loads Mozilla Fluent built-in snippets (actions/nouns/status/… for EN + DE).
/// 2. Loads every app crate's translations via its [`fs_i18n::SnippetPlugin`] impl.
/// 3. Overlays a user-installed language pack from
///    `~/.local/share/fsn/i18n/{lang}/ui.toml` when the active language is not EN.
pub fn init_i18n() {
    let lang = fs_settings::load_active_language();

    let plugins: &[&dyn fs_i18n::SnippetPlugin] = &[
        &fs_store_app::I18nPlugin,
        &fs_settings::I18nPlugin,
        &fs_builder::I18nPlugin,
        &fs_browser::I18nPlugin,
        &fs_lenses::I18nPlugin,
        &fs_managers::I18nPlugin,
        &fs_container_app::I18nPlugin,
        &fs_bots::I18nPlugin,
        &fs_theme_app::I18nPlugin,
        &fs_ai::I18nPlugin,
        &ShellI18nPlugin,
    ];

    if let Err(e) = fs_i18n::init_with_plugins(&lang, plugins) {
        // OnceLock already set during hot-reload — fine. Any other error is a real bug.
        if !e.to_string().contains("already initialized") {
            tracing::error!("i18n init failed: {e}");
        }
    }

    // Overlay user-installed language pack from disk.
    if lang != "en" {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let pack = std::path::PathBuf::from(home)
            .join(".local/share/fsn/i18n")
            .join(&lang)
            .join("ui.toml");
        if let Ok(content) = std::fs::read_to_string(&pack) {
            let _ = fs_i18n::add_toml_lang(&lang, &content);
        }
    }
}

pub use ai_view::AiApp;
pub use app_shell::{AppMode, AppShell, LayoutA, LayoutB, LayoutC, ScreenWrapper};
pub use context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
pub use desktop::Desktop;
pub use header::{Breadcrumb, ShellHeader};
pub use help_view::{HelpApp, HelpSidebarPanel};
pub use launcher::{AppLauncher, LauncherState};
pub use multiwindow::{use_multiwindow, MultiwindowHandle};
pub use notification::{
    Notification, NotificationHistory, NotificationKind, NotificationManager, NotificationStack,
};
pub use sidebar::{default_pinned_items, default_sidebar_sections, ManagerBundle, SidebarEntry};
pub use split_view::{SplitState, SplitView};
pub use taskbar::{LangSwitcher, Taskbar};
pub use web_desktop::{WebDesktop, WebTaskbarState};
pub use window::{
    FsWindow, OpenWindow, Window, WindowButton, WindowContent, WindowHost, WindowId, WindowManager,
    WindowRenderFn, WindowSidebarItem, WindowSize,
};

// Re-export desktop launch abstraction so fs-app can use fs_shell::launch_desktop.
#[cfg(feature = "desktop")]
pub use fs_components::{launch_desktop, spawn_window, DesktopConfig};
pub use spinner::{LoadingOverlay, LoadingSpinner, SpinnerSize};
pub use system_info::{Architecture, Platform, RunMode, SystemInfo, SYSTEM_INFO};
pub use widgets::{
    load_widget_layout, render_widget, save_widget_layout, ClockWidget, PlaceholderWidget,
    QuickNotesWidget, SystemInfoWidget, WidgetKind, WidgetSlot,
};
pub use window_frame::WindowFrame;
