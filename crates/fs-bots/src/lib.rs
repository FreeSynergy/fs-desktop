pub mod app;
pub mod model;
pub mod accounts_view;
pub mod broadcast_view;
pub mod gatekeeper_view;
pub mod groups_view;

pub use app::BotManagerApp;

/// Register app-specific i18n strings for fs-bots (`bots.*` keys).
/// Called once at desktop startup before any component renders.
pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}
