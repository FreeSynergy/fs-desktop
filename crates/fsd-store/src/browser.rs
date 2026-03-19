/// Package browser — fetches the live catalog and renders a filtered package grid.
use dioxus::prelude::*;
use fsd_db::package_registry::PackageRegistry;
use fsn_components::{LoadingOverlay, SpinnerSize};
use fsn_store::{Catalog, StoreClient};

use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::{PackageCard, PackageEntry};

/// Language codes built into the desktop (always considered "installed").
const BUILTIN_LANG_CODES: &[&str] = &["de", "en", "fr", "es", "it", "pt"];

/// Built-in desktop managers — always available in the store catalog,
/// even when the Node is offline. Install state is read from PackageRegistry.
struct BuiltinManager {
    id:          &'static str,
    name:        &'static str,
    description: &'static str,
    icon:        &'static str,
    category:    &'static str,
}

const BUILTIN_MANAGERS: &[BuiltinManager] = &[
    BuiltinManager {
        id:          "manager-language",
        name:        "Language Manager",
        description: "Install and switch interface language packs.",
        icon:        "🌐",
        category:    "managers.language",
    },
    BuiltinManager {
        id:          "manager-theme",
        name:        "Theme Manager",
        description: "Browse and apply desktop color themes.",
        icon:        "🎨",
        category:    "managers.theme",
    },
    BuiltinManager {
        id:          "manager-icons",
        name:        "Icons Manager",
        description: "Install custom icon sets for the desktop.",
        icon:        "🖼",
        category:    "managers.icons",
    },
    BuiltinManager {
        id:          "manager-container-app",
        name:        "Container App Manager",
        description: "Manage Podman container apps and services.",
        icon:        "📦",
        category:    "managers.container_app",
    },
    BuiltinManager {
        id:          "manager-bots",
        name:        "Bots Manager",
        description: "Configure and manage automation bots.",
        icon:        "🤖",
        category:    "managers.bots",
    },
];

/// Install-state filter for the package browser.
#[derive(Clone, PartialEq, Debug)]
pub enum InstallFilter {
    All,
    Installed,
    Available,
    /// Only packages where `update_available` is true.
    Updatable,
}

/// Prepend the raw GitHub base URL to relative icon paths.
///
/// A path is considered relative if it does NOT start with `http`, does NOT
/// start with `<` (inline SVG / emoji markup), and has more than one character
/// (i.e. is not a single emoji glyph).
pub fn resolve_icon(icon: &str) -> String {
    if icon.starts_with("http") || icon.starts_with('<') || icon.chars().count() <= 1 {
        icon.to_string()
    } else {
        format!(
            "https://raw.githubusercontent.com/FreeSynergy/Store/main/{}",
            icon
        )
    }
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
                // Always include built-in managers first (no network needed).
                let mut entries = builtin_manager_entries();

                match StoreClient::node_store().fetch_catalog::<NodePackage>("Node", false).await {
                    Ok(catalog) => {
                        entries.extend(catalog_to_entries(catalog));
                        // Also load shared catalog (language packs, themes, etc.)
                        if let Ok(shared) = StoreClient::node_store()
                            .fetch_catalog::<NodePackage>("shared", false)
                            .await
                        {
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

/// Builds PackageEntry items for all built-in desktop managers.
/// Install state is read live from PackageRegistry so it updates after install.
fn builtin_manager_entries() -> Vec<PackageEntry> {
    let installed_ids: std::collections::HashSet<String> = PackageRegistry::load()
        .into_iter()
        .map(|p| p.id)
        .collect();

    BUILTIN_MANAGERS.iter().map(|m| {
        PackageEntry {
            id:               m.id.to_string(),
            name:             m.name.to_string(),
            description:      m.description.to_string(),
            version:          "1.0.0".to_string(),
            category:         m.category.to_string(),
            kind:             PackageKind::Manager,
            capabilities:     vec![],
            tags:             vec!["manager".to_string(), "builtin".to_string()],
            icon:             Some(m.icon.to_string()),
            store_path:       None,
            installed:        installed_ids.contains(m.id),
            update_available: false,
            license:          String::new(),
            author:           String::new(),
        }
    }).collect()
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
            let icon = p.icon.map(|i| resolve_icon(&i));
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
            icon:             locale.icon.map(|i| resolve_icon(&i)),
            store_path:       locale.path,
            installed,
            update_available: false,
            license:          String::new(),
            author:           String::new(),
        });
    }

    entries
}
