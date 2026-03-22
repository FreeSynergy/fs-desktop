fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_settings::SettingsApp);
}
