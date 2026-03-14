fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fsd_studio::StudioApp);
}
