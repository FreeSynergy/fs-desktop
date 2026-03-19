pub mod app;
pub mod browser;
pub mod missing_icon;
pub mod node_package;
pub mod installed_list;
pub mod package_card;
pub mod install_wizard;
pub mod package_detail;
pub mod store_settings;

pub use app::StoreApp;
pub use install_wizard::{do_install, InstallPopup, InstallResult};

/// Register app-specific i18n strings for fsd-store (`store.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fsn_i18n::add_toml_lang("en", EN);
    let _ = fsn_i18n::add_toml_lang("de", DE);
}
