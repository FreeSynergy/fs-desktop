pub mod app;
pub mod language_panel;
pub mod icons_panel;
pub mod cursor_panel;

pub use app::ManagersApp;
pub use language_panel::LanguageManagerPanel;
pub use icons_panel::IconsManagerPanel;
pub use cursor_panel::CursorManagerPanel;

/// Register i18n strings (no-op stub — managers split into separate crates).
pub fn register_i18n() {}
