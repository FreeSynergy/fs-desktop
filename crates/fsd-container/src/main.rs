fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fsd_container::Container);
}
