// AI Manager view — delegates to fs-ai crate.
use dioxus::prelude::*;

/// Root component for the AI Manager.
#[component]
pub fn AiApp() -> Element {
    rsx! { fs_ai::AiManagerApp {} }
}
