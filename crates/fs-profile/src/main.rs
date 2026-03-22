fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_profile::ProfileApp);
}
