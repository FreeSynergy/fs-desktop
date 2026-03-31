#![deny(clippy::all, clippy::pedantic, warnings)]
// FreeSynergy Settings — iced-based standalone launcher.

#[cfg(feature = "iced")]
fn main() -> fs_gui_engine_iced::iced::Result {
    use fs_settings::app::{Message, SettingsApp};
    fs_gui_engine_iced::IcedEngine::run_app::<SettingsApp, Message, _, _>(
        "FreeSynergy Settings",
        SettingsApp::update,
        SettingsApp::view,
    )
}

#[cfg(not(feature = "iced"))]
fn main() {
    eprintln!("No GUI feature enabled. Build with --features iced");
}
