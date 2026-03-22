pub mod app;
pub mod container_panel;
pub mod cursor_panel;
pub mod icons_panel;
pub mod language_panel;
pub mod manager_view;
pub mod picker_panel;
pub mod theme_panel;
pub mod view_model;

pub use app::ManagersApp;
pub use container_panel::ContainerManagerPanel;
pub use cursor_panel::CursorManagerPanel;
pub use icons_panel::IconsManagerPanel;
pub use language_panel::LanguageManagerPanel;
pub use manager_view::ManagerView;
pub use picker_panel::{PickerItem, PickerPanel};
pub use theme_panel::ThemeManagerPanel;
pub use view_model::PackageViewModel;

/// Register i18n strings (no-op stub — managers split into separate crates).
pub fn register_i18n() {}
