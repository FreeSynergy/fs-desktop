#![deny(clippy::all, clippy::pedantic, warnings)]
pub mod accounts;
pub mod app;
pub mod appearance;
pub mod browser_settings;
pub mod desktop_settings;
pub mod language;
pub mod layout_settings;
pub mod package_settings;
pub mod service_roles;
pub mod settings_config_component;
pub mod shortcuts;
pub mod translation_editor;

pub use app::SettingsApp;
pub use browser_settings::BrowserSettings;
pub use desktop_settings::{
    AnimationConfig, ClickConfig, ClickStyle, DesktopConfig, DisplayMode, DoubleClickAction,
    FocusPolicy, IconConfig, PanelArrangement, ResizeEdgeSize, SidebarConfig, SidebarPosition,
    TaskbarPosition, TitleBarStyle, WindowConfig, WorkspaceConfig,
};
pub use language::{load_active_language, LanguageSettings};
pub use package_settings::{PackageSettingsEntry, PackageSettingsView};
pub use service_roles::{ServiceRole, ServiceRoles, KNOWN_ROLES};
pub use settings_config_component::{SettingsConfigComponent, SettingsSection};
pub use shortcuts::{register_actions, resolve_shortcut, ActionDef, ShortcutsConfig};

/// Returns the path to a named config file in `~/.config/fsn/`.
#[must_use]
///
/// Example: `config_path("desktop.toml")` → `/home/user/.config/fsn/desktop.toml`
pub fn config_path(filename: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("fsn")
        .join(filename)
}
