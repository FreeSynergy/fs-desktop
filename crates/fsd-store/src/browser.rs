/// Package browser — fetches the live catalog from the Node.Store and
/// renders a filterable grid of package cards, with type-filter tabs.
use dioxus::prelude::*;
use fsn_components::{LoadingOverlay, SpinnerSize};
use fsn_store::{Catalog, StoreClient};

use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::{PackageCard, PackageEntry};

// ── PackageBrowser ────────────────────────────────────────────────────────────

/// Package browser component — fetches and renders available packages.
#[component]
pub fn PackageBrowser(search: String, on_select: EventHandler<PackageEntry>) -> Element {
    // Loaded catalog packages
    let packages: Signal<Vec<PackageEntry>> = use_signal(Vec::new);
    let mut loading: Signal<bool>           = use_signal(|| true);
    let mut error: Signal<Option<String>>   = use_signal(|| None);
    let mut kind_filter: Signal<Option<PackageKind>> = use_signal(|| None);

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

    // Filter by search query + kind
    let query = search.to_lowercase();
    let active_kind = kind_filter.read().clone();
    let filtered: Vec<PackageEntry> = packages
        .read()
        .iter()
        .filter(|p| {
            let matches_search = query.is_empty()
                || p.name.to_lowercase().contains(&query)
                || p.description.to_lowercase().contains(&query)
                || p.category.to_lowercase().contains(&query);
            let matches_kind = active_kind.as_ref().map_or(true, |k| &p.kind == k);
            matches_search && matches_kind
        })
        .cloned()
        .collect();

    rsx! {
        div {
            class: "fsd-browser",

            // Kind filter tabs
            div {
                style: "display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 16px;",
                KindFilterPill {
                    label: "All",
                    icon: "📦",
                    active: kind_filter.read().is_none(),
                    on_click: move |_| kind_filter.set(None),
                }
                for kind in PackageKind::ALL {
                    KindFilterPill {
                        key: "{kind:?}",
                        label: kind.label(),
                        icon: kind.icon(),
                        active: kind_filter.read().as_ref() == Some(kind),
                        on_click: {
                            let kind = kind.clone();
                            move |_| kind_filter.set(Some(kind.clone()))
                        },
                    }
                }
            }

            // Loading spinner
            if *loading.read() {
                LoadingOverlay {
                    size: SpinnerSize::Lg,
                    message: Some("Loading catalog…".to_string()),
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
                    if active_kind.is_some() {
                        p { "No {active_kind.as_ref().map(|k| k.label()).unwrap_or(\"\"):} packages match \"{search}\"." }
                    } else {
                        p { "No packages match \"{search}\"." }
                    }
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

// ── KindFilterPill ─────────────────────────────────────────────────────────────

#[component]
fn KindFilterPill(
    label: &'static str,
    icon: &'static str,
    active: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let bg = if active {
        "background: var(--fsn-color-primary); color: white;"
    } else {
        "background: var(--fsn-color-bg-surface); color: var(--fsn-color-text-secondary); \
         border: 1px solid var(--fsn-color-border-default);"
    };

    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 4px; padding: 4px 12px; \
                    border-radius: 999px; font-size: 13px; cursor: pointer; border: none; \
                    transition: background 0.15s; {bg}",
            onclick: on_click,
            span { "{icon}" }
            span { "{label}" }
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
            kind:             p.kind.clone(),
            capabilities:     p.capabilities.clone(),
            icon:             p.icon.clone(),
            installed:        false, // filled in by InstalledList
            update_available: false,
        })
        .collect()
}
