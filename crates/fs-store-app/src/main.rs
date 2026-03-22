fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_store::StoreApp);
}
