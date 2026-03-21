// AI Manager view — delegates to fsd-ai crate.
use dioxus::prelude::*;

/// Root component for the AI Manager.
#[component]
pub fn AiApp() -> Element {
    rsx! { fsd_ai::AiManagerApp {} }
}
