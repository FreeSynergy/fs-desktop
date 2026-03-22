pub mod app;
pub mod browser;
pub mod missing_icon;
pub mod node_package;
pub mod installed_list;
pub mod package_card;
pub mod install_wizard;
pub mod package_detail;
pub mod state;
pub mod store_settings;

pub use app::StoreApp;
pub use install_wizard::{do_install, InstallPopup, InstallResult};
pub use state::{notify_install_changed, INSTALL_COUNTER};

/// Register app-specific i18n strings for fs-store (`store.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}
