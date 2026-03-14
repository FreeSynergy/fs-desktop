pub mod app;
pub mod appearance;
pub mod service_roles;
pub mod language;
pub mod accounts;
pub mod desktop_settings;

pub use app::SettingsApp;
pub use service_roles::{ServiceRoles, ServiceRole, KNOWN_ROLES};
