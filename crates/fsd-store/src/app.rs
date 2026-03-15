/// Store — root component: package browser, installed list, and updates.
use dioxus::prelude::*;
use fsn_components::TabBtn;
use fsn_store::StoreClient;

use crate::browser::PackageBrowser;
use crate::installed_list::InstalledList;
use crate::node_package::NodePackage;

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
                    StoreTab::Browse    => rsx! { PackageBrowser { search: search.read().clone() } },
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

/// Shows containers where the catalog version is newer than the installed one.
#[component]
fn UpdatesList(catalog_versions: Vec<(String, String)>) -> Element {
    use fsn_container::{ContainerInfo, PodmanClient};

    let mut containers: Signal<Vec<ContainerInfo>> = use_signal(Vec::new);
    use_future(move || async move {
        loop {
            if let Ok(client) = PodmanClient::new() {
                if let Ok(list) = client.list(true).await {
                    containers.set(list);
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    // Find updateable containers
    let updateable: Vec<(String, String, String)> = containers.read().iter()
        .filter_map(|c| {
            let module_id = c.labels.get("fsn.module.id")?;
            let installed_ver = c.labels.get("fsn.module.version")?;
            let (_, catalog_ver) = catalog_versions.iter().find(|(id, _)| id == module_id)?;
            if catalog_ver != installed_ver {
                Some((c.name.clone(), installed_ver.clone(), catalog_ver.clone()))
            } else {
                None
            }
        })
        .collect();

    rsx! {
        div {
            if updateable.is_empty() {
                div {
                    style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                    p { style: "font-size: 24px;", "✓" }
                    p { "All installed modules are up to date." }
                }
            } else {
                div {
                    style: "margin-bottom: 16px;",
                    h3 { style: "margin: 0 0 4px 0;", "{updateable.len()} update(s) available" }
                    p { style: "color: var(--fsn-color-text-muted); font-size: 13px; margin: 0;",
                        "Run "
                        code { "fsn deploy" }
                        " to apply updates."
                    }
                }
                table {
                    style: "width: 100%; border-collapse: collapse;",
                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fsn-color-border-default); font-size: 12px; color: var(--fsn-color-text-muted);",
                            th { style: "text-align: left; padding: 8px;",  "CONTAINER" }
                            th { style: "text-align: left; padding: 8px;",  "INSTALLED" }
                            th { style: "text-align: left; padding: 8px;",  "AVAILABLE" }
                        }
                    }
                    tbody {
                        for (name, installed, catalog) in updateable.iter().cloned().collect::<Vec<_>>() {
                            tr {
                                style: "border-bottom: 1px solid var(--fsn-color-border-default);",
                                td { style: "padding: 10px 8px; font-weight: 500;",  "{name}" }
                                td { style: "padding: 10px 8px; font-size: 13px; color: var(--fsn-color-text-muted);", "v{installed}" }
                                td { style: "padding: 10px 8px;",
                                    span {
                                        style: "font-size: 13px; background: var(--fsn-color-warning); color: black; padding: 2px 8px; border-radius: 4px;",
                                        "v{catalog}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

