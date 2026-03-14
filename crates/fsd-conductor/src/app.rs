/// Conductor — main app component for container/service/bot management.
use dioxus::prelude::*;

use crate::service_list::ServiceList;
use crate::log_viewer::LogViewer;

/// Active tab in the Conductor.
#[derive(Clone, PartialEq, Debug)]
pub enum ConductorTab {
    Services,
    Bots,
    Resources,
    Logs,
}

/// Root component of the Conductor app.
#[component]
pub fn ConductorApp() -> Element {
    let active_tab = use_signal(|| ConductorTab::Services);
    let selected_service: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            class: "fsd-conductor",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fsn-color-bg-base);",

            // Tab bar
            div {
                class: "fsd-conductor__tabs",
                style: "display: flex; border-bottom: 1px solid var(--fsn-color-border-default); background: var(--fsn-color-bg-surface);",

                ConductorTabBtn { label: "Services", tab: ConductorTab::Services, active: active_tab }
                ConductorTabBtn { label: "Bots",     tab: ConductorTab::Bots,     active: active_tab }
                ConductorTabBtn { label: "Resources",tab: ConductorTab::Resources, active: active_tab }
                ConductorTabBtn { label: "Logs",     tab: ConductorTab::Logs,     active: active_tab }
            }

            // Content area
            div {
                class: "fsd-conductor__content",
                style: "flex: 1; overflow: auto; padding: 16px;",

                match *active_tab.read() {
                    ConductorTab::Services  => rsx! { ServiceList { selected: selected_service } },
                    ConductorTab::Bots      => rsx! { div { "Bot management — coming soon" } },
                    ConductorTab::Resources => rsx! { div { "Resource configuration — coming soon" } },
                    ConductorTab::Logs      => rsx! {
                        LogViewer {
                            service: selected_service.read().clone().unwrap_or_default()
                        }
                    },
                }
            }
        }
    }
}

/// A single tab button in the Conductor tab bar.
#[component]
fn ConductorTabBtn(
    label: &'static str,
    tab: ConductorTab,
    mut active: Signal<ConductorTab>,
) -> Element {
    let is_active = *active.read() == tab;
    rsx! {
        button {
            style: "padding: 10px 20px; border: none; cursor: pointer; font-size: 14px; background: {if is_active { \"var(--fsn-color-bg-base)\" } else { \"transparent\" }}; border-bottom: 2px solid {if is_active { \"var(--fsn-color-primary)\" } else { \"transparent\" }};",
            onclick: move |_| *active.write() = tab.clone(),
            "{label}"
        }
    }
}
