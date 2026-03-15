/// Package browser — fetches the live catalog from the Node.Store and
/// renders a filterable grid of package cards.
use dioxus::prelude::*;
use fsn_store::{Catalog, StoreClient};

use crate::node_package::NodePackage;
use crate::package_card::{PackageCard, PackageEntry};

// ── PackageBrowser ────────────────────────────────────────────────────────────

/// Package browser component — fetches and renders available packages.
#[component]
pub fn PackageBrowser(search: String) -> Element {
    // Loaded catalog packages
    let packages: Signal<Vec<PackageEntry>> = use_signal(Vec::new);
    let mut loading: Signal<bool>           = use_signal(|| true);
    let mut error: Signal<Option<String>>   = use_signal(|| None);

    // Fetch on mount — re-runs when component remounts
    {
        let packages = packages.clone();
        use_future(move || {
            let mut packages = packages.clone();
            async move {
                match StoreClient::node_store().fetch_catalog::<NodePackage>("Node", false).await {
                    Ok(catalog) => {
                        let entries = catalog_to_entries(catalog);
                        packages.set(entries);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load catalog: {e}")));
                    }
                }
                loading.set(false);
            }
        });
    }

    // Filter by search query
    let query = search.to_lowercase();
    let filtered: Vec<PackageEntry> = packages
        .read()
        .iter()
        .filter(|p| {
            query.is_empty()
                || p.name.to_lowercase().contains(&query)
                || p.description.to_lowercase().contains(&query)
                || p.category.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        div {
            class: "fsd-browser",

            // Loading spinner
            if *loading.read() {
                div {
                    style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                    p { "Loading catalog…" }
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-color-error); background: rgba(239,68,68,0.1); border: 1px solid var(--fsn-color-error); border-radius: 6px; padding: 12px; font-size: 13px;",
                    p { strong { "Store unavailable" } }
                    p { "{err}" }
                    p { style: "color: var(--fsn-color-text-muted); font-size: 12px;",
                        "Using offline cache if available. Check your internet connection."
                    }
                }
            } else if filtered.is_empty() {
                div {
                    style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                    p { "No packages match \"{search}\"." }
                }
            } else {
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px;",
                    for pkg in filtered {
                        PackageCard { package: pkg }
                    }
                }
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Convert a fetched `Catalog<NodePackage>` into `PackageEntry` display objects.
fn catalog_to_entries(catalog: Catalog<NodePackage>) -> Vec<PackageEntry> {
    catalog
        .packages
        .into_iter()
        .map(|p| PackageEntry {
            id:               p.id.clone(),
            name:             p.name.clone(),
            description:      p.description.clone(),
            version:          p.version.clone(),
            category:         p.category.clone(),
            icon:             p.icon.clone(),
            installed:        false, // filled in by InstalledList
            update_available: false,
        })
        .collect()
}
