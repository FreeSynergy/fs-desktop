/// Package browser — fetches the live catalog and renders a filtered package grid.
use dioxus::prelude::*;
use fsd_db::package_registry::PackageRegistry;
use fsn_components::{LoadingOverlay, SpinnerSize};
use fsn_store::{Catalog, StoreClient};

use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::{PackageCard, PackageEntry};

/// Package browser component. `kind` filters by package type (None = show all).
#[component]
pub fn PackageBrowser(
    search: String,
    kind: Option<PackageKind>,
    on_select: EventHandler<PackageEntry>,
) -> Element {
    let packages: Signal<Vec<PackageEntry>> = use_signal(Vec::new);
    let mut loading: Signal<bool>           = use_signal(|| true);
    let mut error: Signal<Option<String>>   = use_signal(|| None);

    {
        let packages = packages.clone();
        use_future(move || {
            let mut packages = packages.clone();
            async move {
                match StoreClient::node_store().fetch_catalog::<NodePackage>("Node", false).await {
                    Ok(catalog) => {
                        let mut entries = catalog_to_entries(catalog);
                        // Also load shared catalog (language packs, themes, etc.)
                        if let Ok(shared) = StoreClient::node_store()
                            .fetch_catalog::<NodePackage>("shared", false)
                            .await
                        {
                            entries.extend(catalog_to_entries(shared));
                        }
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

    let query = search.to_lowercase();
    // Split query into individual words — all must match (AND logic)
    let query_words: Vec<String> = query
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();

    let filtered: Vec<PackageEntry> = packages
        .read()
        .iter()
        .filter(|p| {
            let matches_search = query_words.is_empty() || query_words.iter().all(|word| {
                p.name.to_lowercase().contains(word.as_str())
                    || p.description.to_lowercase().contains(word.as_str())
                    || p.category.to_lowercase().contains(word.as_str())
                    || p.tags.iter().any(|t| t.to_lowercase().contains(word.as_str()))
            });
            let matches_kind = kind.as_ref().map_or(true, |k| &p.kind == k);
            matches_search && matches_kind
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "fsd-browser",
            if *loading.read() {
                LoadingOverlay {
                    size: SpinnerSize::Lg,
                    message: Some("Loading catalog…".to_string()),
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-color-error); background: rgba(239,68,68,0.1); \
                            border: 1px solid var(--fsn-color-error); border-radius: 6px; \
                            padding: 12px; font-size: 13px;",
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
                        PackageCard {
                            key: "{pkg.id}",
                            package: pkg.clone(),
                            on_details: {
                                let p = pkg.clone();
                                move |_| on_select.call(p.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

fn catalog_to_entries(catalog: Catalog<NodePackage>) -> Vec<PackageEntry> {
    let installed_ids: std::collections::HashSet<String> = PackageRegistry::load()
        .into_iter()
        .map(|p| p.id)
        .collect();

    catalog
        .packages
        .into_iter()
        .map(|p| {
            let installed = installed_ids.contains(&p.id);
            PackageEntry {
                id:               p.id.clone(),
                name:             p.name.clone(),
                description:      p.description.clone(),
                version:          p.version.clone(),
                category:         p.category.clone(),
                kind:             p.kind.clone(),
                capabilities:     p.capabilities.clone(),
                tags:             p.tags.clone(),
                icon:             p.icon.clone(),
                store_path:       p.path.clone(),
                installed,
                update_available: false,
            }
        })
        .collect()
}
