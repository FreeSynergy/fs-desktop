/// Store — root component: package browser, install, and updates.
use dioxus::prelude::*;

use crate::browser::PackageBrowser;

#[derive(Clone, PartialEq, Debug)]
pub enum StoreTab {
    Browse,
    Installed,
    Updates,
}

/// Root Store component.
#[component]
pub fn StoreApp() -> Element {
    let active_tab = use_signal(|| StoreTab::Browse);
    let search = use_signal(String::new);

    rsx! {
        div {
            class: "fsd-store",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fsn-color-bg-base);",

            // Header with search
            div {
                style: "padding: 16px; background: var(--fsn-color-bg-surface); border-bottom: 1px solid var(--fsn-color-border-default);",
                h2 { style: "margin: 0 0 12px 0; font-size: 20px;", "Store" }
                input {
                    r#type: "search",
                    placeholder: "Search packages…",
                    style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 14px;",
                    oninput: move |e| *search.write() = e.value(),
                }
            }

            // Tab bar
            div {
                style: "display: flex; border-bottom: 1px solid var(--fsn-color-border-default);",
                StoreTabBtn { label: "Browse",    tab: StoreTab::Browse,     active: active_tab }
                StoreTabBtn { label: "Installed", tab: StoreTab::Installed,  active: active_tab }
                StoreTabBtn { label: "Updates",   tab: StoreTab::Updates,    active: active_tab }
            }

            // Content
            div {
                style: "flex: 1; overflow: auto; padding: 16px;",
                match *active_tab.read() {
                    StoreTab::Browse    => rsx! { PackageBrowser { search: search.read().clone() } },
                    StoreTab::Installed => rsx! { div { "Installed packages — coming soon" } },
                    StoreTab::Updates   => rsx! { div { "Available updates — coming soon" } },
                }
            }
        }
    }
}

#[component]
fn StoreTabBtn(label: &'static str, tab: StoreTab, mut active: Signal<StoreTab>) -> Element {
    let is_active = *active.read() == tab;
    rsx! {
        button {
            style: "padding: 10px 20px; border: none; cursor: pointer; font-size: 14px; background: {if is_active { \"var(--fsn-color-bg-base)\" } else { \"transparent\" }}; border-bottom: 2px solid {if is_active { \"var(--fsn-color-primary)\" } else { \"transparent\" }};",
            onclick: move |_| *active.write() = tab.clone(),
            "{label}"
        }
    }
}
