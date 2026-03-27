#![deny(clippy::all, clippy::pedantic, warnings)]
fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(|| {
        use dioxus::prelude::*;
        rsx! { fs_settings::SettingsApp {} }
    });
}
