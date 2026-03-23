fn main() {
    // Initialize i18n before Dioxus starts — guarantees all translation keys
    // are resolved before any component renders for the first time.
    fs_gui_workspace::init_i18n();

    #[cfg(feature = "desktop")]
    fs_gui_workspace::launch_desktop(
        fs_gui_workspace::DesktopConfig::new().with_all_navigation(),
        fs_gui_workspace::Desktop,
    );
}
