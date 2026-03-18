pub mod app;
pub mod bot_management;
pub mod dep_graph;
pub mod instance_config;
pub mod log_viewer;
pub mod resource_editor;
pub mod resource_view;
pub mod service_detail;
pub mod service_list;

pub use app::ConductorApp;

/// Register app-specific i18n strings for fsd-conductor (`conductor.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fsn_i18n::add_toml_lang("en", EN);
    let _ = fsn_i18n::add_toml_lang("de", DE);
}
