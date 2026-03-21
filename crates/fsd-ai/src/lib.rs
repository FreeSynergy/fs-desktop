pub mod app;

pub use app::AiManagerApp;

pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fsn_i18n::add_toml_lang("en", EN);
    let _ = fsn_i18n::add_toml_lang("de", DE);
}
