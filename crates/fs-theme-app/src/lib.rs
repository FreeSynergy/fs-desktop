pub mod app;
pub mod themes_view;
pub mod colors_view;
pub mod cursor_view;
pub mod chrome_view;

pub use app::ThemeManagerApp;

pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}
