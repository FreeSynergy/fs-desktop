pub mod app;
pub mod build_view;
pub mod instance_config;
pub mod log_viewer;
pub mod service_detail;
pub mod service_list;

pub use app::Container;

/// Register app-specific i18n strings for fs-container-app (`container.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}
