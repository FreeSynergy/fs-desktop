pub mod app;
pub mod bridge_builder;
pub mod container_builder;
pub mod i18n_editor;
pub mod ollama;
pub mod resource_browser;

pub use app::BuilderApp;

/// Register app-specific i18n strings for fs-builder (`builder.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}
