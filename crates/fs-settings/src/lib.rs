#![deny(clippy::all, clippy::pedantic, warnings)]
pub mod accounts;
pub mod app;
pub mod appearance;
pub mod browser_settings;
pub mod desktop_settings;
pub mod language;
pub mod package_settings;
pub mod service_roles;
pub mod shortcuts;
pub mod translation_editor;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-settings (`settings.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &'static str {
        "fs-settings"
    }
    fn snippets(&self) -> &[(&str, &str)] {
        I18N_SNIPPETS
    }
}

pub use app::{SettingsApp, SettingsAppProps};
pub use browser_settings::BrowserSettings;
pub use desktop_settings::{
    DesktopConfig, DisplayMode, SidebarConfig, SidebarPosition, TaskbarPosition,
};
#[allow(deprecated)]
pub use language::{load_active_language, LangContext, LanguageSettings};
pub use package_settings::{PackageSettingsEntry, PackageSettingsView};
pub use service_roles::{ServiceRole, ServiceRoles, KNOWN_ROLES};
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
