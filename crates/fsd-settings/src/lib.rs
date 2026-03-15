pub mod app;
pub mod appearance;
pub mod service_roles;
pub mod language;
pub mod accounts;
pub mod desktop_settings;

pub use app::SettingsApp;
pub use service_roles::{ServiceRoles, ServiceRole, KNOWN_ROLES};
pub use desktop_settings::{DesktopConfig, DisplayMode, SidebarConfig, SidebarPosition, TaskbarPosition};

/// Returns the path to a named config file in `~/.config/fsn/`.
///
/// Example: `config_path("desktop.toml")` → `/home/user/.config/fsn/desktop.toml`
pub fn config_path(filename: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".config").join("fsn").join(filename)
}
