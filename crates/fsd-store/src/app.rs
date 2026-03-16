/// Store — root component: package browser, installed list, and updates.
use dioxus::prelude::*;
use fsn_components::TabBtn;
use fsn_store::StoreClient;

use crate::browser::PackageBrowser;
use crate::installed_list::InstalledList;
use crate::node_package::NodePackage;
use crate::package_card::PackageEntry;
use crate::package_detail::PackageDetail;

#[derive(Clone, PartialEq, Debug)]
pub enum StoreTab {
    Browse,
    Installed,
    Updates,
}

/// Root Store component.
#[component]
pub fn StoreApp() -> Element {
    let mut active_tab = use_signal(|| StoreTab::Browse);
    let mut search = use_signal(String::new);
    let mut detail: Signal<Option<PackageEntry>> = use_signal(|| None);

    // Catalog versions for update detection — (id, version) pairs
    let catalog_versions: Signal<Vec<(String, String)>> = use_signal(Vec::new);
    {
        let catalog_versions = catalog_versions.clone();
        use_future(move || {
            let mut catalog_versions = catalog_versions.clone();
            async move {
                if let Ok(catalog) = StoreClient::node_store()
                    .fetch_catalog::<NodePackage>("Node", false)
                    .await
                {
                    catalog_versions.set(
                        catalog.packages.into_iter()
                            .map(|p| (p.id, p.version))
                            .collect(),
                    );
                }
            }
        });
    }

    // Show detail panel when a package is selected
    if let Some(pkg) = detail.read().clone() {
        return rsx! {
            PackageDetail {
                package: pkg,
                on_back: move |_| detail.set(None),
            }
        };
    }

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
                TabBtn { label: "Browse",    is_active: *active_tab.read() == StoreTab::Browse,    on_click: move |_| active_tab.set(StoreTab::Browse) }
                TabBtn { label: "Installed", is_active: *active_tab.read() == StoreTab::Installed, on_click: move |_| active_tab.set(StoreTab::Installed) }
                TabBtn { label: "Updates",   is_active: *active_tab.read() == StoreTab::Updates,   on_click: move |_| active_tab.set(StoreTab::Updates) }
            }

            // Content
            div {
                style: "flex: 1; overflow: auto; padding: 16px;",
                match *active_tab.read() {
                    StoreTab::Browse => rsx! {
                        PackageBrowser {
                            search: search.read().clone(),
                            on_select: move |pkg| detail.set(Some(pkg)),
                        }
                    },
                    StoreTab::Installed => rsx! {
                        InstalledList {
                            catalog_versions: catalog_versions.read().clone(),
                        }
                    },
                    StoreTab::Updates   => rsx! {
                        UpdatesList {
                            catalog_versions: catalog_versions.read().clone(),
                        }
                    },
                }
            }
        }
    }
}

// ── UpdatesList ───────────────────────────────────────────────────────────────

/// Shows available catalog updates.
///
/// Installed version detection via container labels requires Podman socket (removed).
/// Run `fsn deploy` to apply updates for all deployed services.
#[component]
fn UpdatesList(catalog_versions: Vec<(String, String)>) -> Element {
    rsx! {
        div {
            style: "text-align: center; color: var(--fsn-text-muted); padding: 48px;",
            p { style: "font-size: 24px; margin-bottom: 12px;", "↑" }
            p { style: "margin-bottom: 8px;", "Update detection requires deployment metadata." }
            p { style: "font-size: 13px;",
                "Run "
                code { style: "background: var(--fsn-bg-elevated); padding: 2px 6px; border-radius: 4px;",
                    "fsn deploy"
                }
                " to check and apply updates for all deployed services."
            }
            if !catalog_versions.is_empty() {
                p { style: "margin-top: 16px; font-size: 13px;",
                    "{catalog_versions.len()} package(s) available in the catalog."
                }
            }
        }
    }
}

