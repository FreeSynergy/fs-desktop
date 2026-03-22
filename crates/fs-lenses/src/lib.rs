pub mod app;
pub mod model;
pub mod query;

pub use app::LensesApp;

/// Register lenses i18n strings (`lenses.*` keys).
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}
