pub mod app;
pub mod appearance;
pub mod service_roles;
pub mod language;
pub mod translation_editor;
pub mod accounts;
pub mod desktop_settings;
pub mod shortcuts;

/// Register app-specific i18n strings for fs-settings (`settings.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}

pub use app::SettingsApp;
pub use language::{load_active_language, LangContext, LanguageSettings};
pub use service_roles::{ServiceRoles, ServiceRole, KNOWN_ROLES};
pub use desktop_settings::{DesktopConfig, DisplayMode, SidebarConfig, SidebarPosition, TaskbarPosition};
pub use shortcuts::{ActionDef, ShortcutsConfig, register_actions, resolve_shortcut};

/// Returns the path to a named config file in `~/.config/fsn/`.
///
/// Example: `config_path("desktop.toml")` → `/home/user/.config/fsn/desktop.toml`
pub fn config_path(filename: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".config").join("fsn").join(filename)
}
