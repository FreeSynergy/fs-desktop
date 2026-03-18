/// Managers stub — these managers have been split into separate programs.
use dioxus::prelude::*;

/// Stub component shown when the old "Managers" app is opened.
/// The actual functionality lives in Container Manager, Theme Manager, and Bot Manager.
#[component]
pub fn ManagersApp() -> Element {
    rsx! {
        div {
            style: "padding: 40px; text-align: center; color: var(--fsn-color-text-muted, #64748b);",
            p { "The managers have been split into separate programs." }
            p { "Use Container Manager, Theme Manager, and Bot Manager from the sidebar." }
        }
    }
}
