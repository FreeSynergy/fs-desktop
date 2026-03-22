fn main() {
    #[cfg(feature = "desktop")]
    fs_shell::launch_desktop(
        fs_shell::DesktopConfig::new().with_all_navigation(),
        fs_shell::Desktop,
    );
}
