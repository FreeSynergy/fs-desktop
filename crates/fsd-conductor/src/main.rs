fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fsd_conductor::ConductorApp);
}
