pub mod app;
pub mod bridge_builder;
pub mod container_builder;
pub mod i18n_editor;
pub mod ollama;
pub mod resource_browser;

pub use app::BuilderApp;

/// Register app-specific i18n strings for fsd-builder (`builder.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fsn_i18n::add_toml_lang("en", EN);
    let _ = fsn_i18n::add_toml_lang("de", DE);
}
