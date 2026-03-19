/// Package browser — fetches the live catalog and renders a filtered package grid.
use dioxus::prelude::*;
use fsd_db::package_registry::PackageRegistry;
use fsn_components::{LoadingOverlay, SpinnerSize};
use fsn_store::{Catalog, StoreClient};

use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::{PackageCard, PackageEntry};

/// Language codes built into the desktop (always considered "installed").
const BUILTIN_LANG_CODES: &[&str] = &["de", "en", "fr", "es", "it", "pt"];

/// Install-state filter for the package browser.
#[derive(Clone, PartialEq, Debug)]
pub enum InstallFilter {
    All,
    Installed,
    Available,
    /// Only packages where `update_available` is true.
    Updatable,
}

/// Resolve an icon field to a usable URL, or `None` if it cannot be used as an `<img>` src.
///
/// Returns `Some(url)` only for:
/// - Absolute HTTP(S) URLs
/// - Relative paths (converted to raw.githubusercontent.com URLs)
///
/// Returns `None` for emoji glyphs, single characters, or empty strings so the
/// caller can fall back to the `MissingIcon` placeholder instead of a broken `<img>`.
pub fn resolve_icon(icon: &str) -> Option<String> {
    if icon.is_empty() {
        return None;
    }
    // Single Unicode scalar = emoji or single glyph — not a URL
    if icon.chars().count() <= 1 {
        return None;
    }
    if icon.starts_with("http") {
        return Some(icon.to_string());
    }
    if icon.starts_with('<') {
        // Inline SVG markup — not a path, skip for now
        return None;
    }
    // Relative path → prepend GitHub raw base
    Some(format!(
        "https://raw.githubusercontent.com/FreeSynergy/Store/main/{}",
        icon
    ))
}

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
    let mut install_filter = use_signal(|| InstallFilter::All);

    {
        let packages = packages.clone();
        use_future(move || {
            let mut packages = packages.clone();
            async move {
                let mut client = StoreClient::node_store();
                let mut entries = Vec::new();

                // Desktop catalog: managers
                if let Ok(desktop) = client.fetch_catalog::<NodePackage>("desktop", false).await {
                    entries.extend(catalog_to_entries(desktop));
                }

                match client.fetch_catalog::<NodePackage>("node", false).await {
                    Ok(catalog) => {
                        entries.extend(catalog_to_entries(catalog));
                        // Shared catalog: language packs, themes, widgets, etc.
                        if let Ok(shared) = client.fetch_catalog::<NodePackage>("shared", false).await {
                            entries.extend(catalog_to_entries(shared));
                        }
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load catalog: {e}")));
                    }
                }
                packages.set(entries);
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

    let cur_filter = install_filter.read().clone();
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
            let matches_kind    = kind.as_ref().map_or(true, |k| &p.kind == k);
            let matches_install = match &cur_filter {
                InstallFilter::All       => true,
                InstallFilter::Installed => p.installed,
                InstallFilter::Available => !p.installed,
                InstallFilter::Updatable => p.update_available,
            };
            matches_search && matches_kind && matches_install
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "fsd-browser",
            // ── Install filter bar ──────────────────────────────────────────────
            div {
                style: "display: flex; gap: 6px; margin-bottom: 12px;",
                for (label, variant) in [
                    (fsn_i18n::t("store.filter.all"),       InstallFilter::All),
                    (fsn_i18n::t("store.filter.installed"), InstallFilter::Installed),
                    (fsn_i18n::t("store.filter.available"), InstallFilter::Available),
                    (fsn_i18n::t("store.filter.updatable"), InstallFilter::Updatable),
                ] {
                    {
                        let active = *install_filter.read() == variant;
                        let v = variant.clone();
                        rsx! {
                            button {
                                key: "{label}",
                                style: if active {
                                    "padding: 4px 12px; font-size: 12px; border-radius: var(--fsn-radius-sm); \
                                     border: 1px solid var(--fsn-color-primary); cursor: pointer; \
                                     background: var(--fsn-color-primary); color: white;"
                                } else {
                                    "padding: 4px 12px; font-size: 12px; border-radius: var(--fsn-radius-sm); \
                                     border: 1px solid var(--fsn-color-border-default); cursor: pointer; \
                                     background: transparent; color: var(--fsn-color-text-primary);"
                                },
                                onclick: move |_| install_filter.set(v.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }

            if *loading.read() {
                LoadingOverlay {
                    size: SpinnerSize::Lg,
                    message: Some(fsn_i18n::t("store.loading_catalog")),
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-color-error); background: rgba(239,68,68,0.1); \
                            border: 1px solid var(--fsn-color-error); border-radius: 6px; \
                            padding: 12px; font-size: 13px;",
                    p { strong { {fsn_i18n::t("store.unavailable")} } }
                    p { "{err}" }
                    p { style: "color: var(--fsn-color-text-muted); font-size: 12px;",
                        {fsn_i18n::t("store.offline_hint")}
                    }
                }
            } else if filtered.is_empty() {
                div {
                    style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                    p { {fsn_i18n::t_with("store.no_match", &[("search", search.as_str())])} }
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

    let mut entries: Vec<PackageEntry> = catalog
        .packages
        .into_iter()
        .map(|p| {
            let installed = installed_ids.contains(&p.id);
            let icon = p.icon.and_then(|i| resolve_icon(&i));
            PackageEntry {
                id:               p.id,
                name:             p.name,
                description:      p.description,
                version:          p.version,
                category:         p.category,
                kind:             p.kind,
                capabilities:     p.capabilities,
                tags:             p.tags,
                icon,
                store_path:       p.path,
                installed,
                update_available: false,
                license:          p.license,
                author:           p.author,
            }
        })
        .collect();

    // Add locales as Language packages
    for locale in catalog.locales {
        let installed = installed_ids.contains(&locale.code)
            || BUILTIN_LANG_CODES.contains(&locale.code.as_str());
        entries.push(PackageEntry {
            id:               locale.code.clone(),
            name:             locale.name.clone(),
            description:      format!(
                "{} language pack · {}% complete",
                locale.name, locale.completeness
            ),
            version:          locale.version,
            category:         "i18n.language".to_string(),
            kind:             PackageKind::Language,
            capabilities:     vec![],
            tags:             vec!["language".to_string(), locale.direction],
            icon:             locale.icon.and_then(|i| resolve_icon(&i)),
            store_path:       locale.path,
            installed,
            update_available: false,
            license:          "MIT".to_string(),
            author:           "Kal El".to_string(),
        });
    }

    entries
}
