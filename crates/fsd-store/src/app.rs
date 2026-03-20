/// Store — root component with package-type sidebar navigation.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};
use fsn_store::StoreClient;

use crate::browser::PackageBrowser;
use crate::installed_list::InstalledList;
use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::PackageEntry;
use crate::package_detail::PackageDetail;
use crate::store_settings::StoreSettings;

#[derive(Clone, PartialEq, Debug)]
pub enum StoreTab {
    All,
    Apps,
    Managers,
    ContainerApps,
    Languages,
    Themes,
    Widgets,
    Bots,
    MessengerAdapters,
    Bridges,
    Tasks,
    Bundles,
    Installed,
    Updates,
    Settings,
}

impl StoreTab {
    /// Returns the PackageKind filter for this tab (None = show all).
    pub fn kind_filter(&self) -> Option<PackageKind> {
        match self {
            Self::Apps             => Some(PackageKind::App),
            Self::Managers         => Some(PackageKind::Manager),
            Self::ContainerApps    => Some(PackageKind::Container),
            Self::Languages        => Some(PackageKind::Language),
            Self::Themes           => Some(PackageKind::Theme),
            Self::Widgets          => Some(PackageKind::Widget),
            Self::Bots             => Some(PackageKind::BotCommand),
            Self::MessengerAdapters => Some(PackageKind::MessengerAdapter),
            Self::Bridges          => Some(PackageKind::Bridge),
            Self::Tasks            => Some(PackageKind::Task),
            Self::Bundles          => Some(PackageKind::Bundle),
            Self::All | Self::Installed | Self::Updates | Self::Settings => None,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::All              => fsn_i18n::t("store.tab.all"),
            Self::Apps             => fsn_i18n::t("store.tab.apps"),
            Self::Managers         => fsn_i18n::t("store.tab.managers"),
            Self::ContainerApps    => fsn_i18n::t("store.tab.container_apps"),
            Self::Languages        => fsn_i18n::t("store.tab.languages"),
            Self::Themes           => fsn_i18n::t("store.tab.themes"),
            Self::Widgets          => fsn_i18n::t("store.tab.widgets"),
            Self::Bots             => fsn_i18n::t("store.tab.bots"),
            Self::MessengerAdapters => fsn_i18n::t("store.tab.messenger_adapters"),
            Self::Bridges          => fsn_i18n::t("store.tab.bridges"),
            Self::Tasks            => fsn_i18n::t("store.tab.tasks"),
            Self::Bundles          => fsn_i18n::t("store.tab.bundles"),
            Self::Installed        => fsn_i18n::t("store.tab.installed"),
            Self::Updates          => fsn_i18n::t("store.tab.updates"),
            Self::Settings         => fsn_i18n::t("store.tab.settings"),
        }
    }

    /// SVG icon for this tab.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::All           => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>"#,
            Self::Apps          => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 9h6"/><path d="M9 12h6"/><path d="M9 15h4"/></svg>"#,
            Self::Managers      => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"#,
            Self::ContainerApps => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="4" rx="1"/><rect x="2" y="10" width="20" height="4" rx="1"/><rect x="2" y="17" width="20" height="4" rx="1"/><circle cx="6" cy="5" r="1" fill="currentColor"/><circle cx="6" cy="12" r="1" fill="currentColor"/><circle cx="6" cy="19" r="1" fill="currentColor"/></svg>"#,
            Self::Languages     => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>"#,
            Self::Themes        => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r="0.5" fill="currentColor"/><circle cx="17.5" cy="10.5" r="0.5" fill="currentColor"/><circle cx="8.5" cy="7.5" r="0.5" fill="currentColor"/><circle cx="6.5" cy="12.5" r="0.5" fill="currentColor"/><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10c.19 0 .37-.01.56-.02a1 1 0 0 0 .94-1V19a2 2 0 0 1 2-2h3a2 2 0 0 0 2-2v-1c0-5.52-4.48-10-10-10z"/></svg>"#,
            Self::Widgets       => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>"#,
            Self::Bots              => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M12 11V3"/><circle cx="12" cy="3" r="1" fill="currentColor"/><line x1="8" y1="16" x2="8" y2="16" stroke-width="3"/><line x1="16" y1="16" x2="16" y2="16" stroke-width="3"/><path d="M9 20h6"/></svg>"#,
            Self::MessengerAdapters => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/><path d="M8 10h8"/><path d="M8 14h5"/></svg>"#,
            Self::Bridges           => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>"#,
            Self::Tasks         => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><polyline points="3 6 4 7 6 5"/><polyline points="3 12 4 13 6 11"/><polyline points="3 18 4 19 6 17"/></svg>"#,
            Self::Bundles       => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>"#,
            Self::Installed     => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>"#,
            Self::Updates       => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/></svg>"#,
            Self::Settings      => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"#,
        }
    }

    /// Stable string ID (not translated — used for routing/selection).
    pub fn id(&self) -> &'static str {
        match self {
            Self::All               => "All",
            Self::Apps              => "Apps",
            Self::Managers          => "Managers",
            Self::ContainerApps     => "ContainerApps",
            Self::Languages         => "Languages",
            Self::Themes            => "Themes",
            Self::Widgets           => "Widgets",
            Self::Bots              => "Bots",
            Self::MessengerAdapters => "MessengerAdapters",
            Self::Bridges           => "Bridges",
            Self::Tasks             => "Tasks",
            Self::Bundles           => "Bundles",
            Self::Installed         => "Installed",
            Self::Updates           => "Updates",
            Self::Settings          => "Settings",
        }
    }

    /// Look up a tab by its ID string.
    pub fn from_id(id: &str) -> Self {
        match id {
            "Apps"              => Self::Apps,
            "Managers"          => Self::Managers,
            "ContainerApps"     => Self::ContainerApps,
            "Languages"         => Self::Languages,
            "Themes"            => Self::Themes,
            "Widgets"           => Self::Widgets,
            "Bots"              => Self::Bots,
            "MessengerAdapters" => Self::MessengerAdapters,
            "Bridges"           => Self::Bridges,
            "Tasks"             => Self::Tasks,
            "Bundles"           => Self::Bundles,
            "Installed"         => Self::Installed,
            "Updates"           => Self::Updates,
            "Settings"          => Self::Settings,
            _                   => Self::All,
        }
    }
}

const ALL_TABS: &[StoreTab] = &[
    StoreTab::All,
    StoreTab::Apps,
    StoreTab::Managers,
    StoreTab::ContainerApps,
    StoreTab::Languages,
    StoreTab::Themes,
    StoreTab::Widgets,
    StoreTab::Bots,
    StoreTab::MessengerAdapters,
    StoreTab::Bridges,
    StoreTab::Tasks,
    StoreTab::Bundles,
    StoreTab::Installed,
    StoreTab::Updates,
    StoreTab::Settings,
];

/// Root Store component.
#[component]
pub fn StoreApp() -> Element {
    let mut active_tab = use_signal(|| StoreTab::All);
    let mut search = use_signal(String::new);
    let mut detail: Signal<Option<PackageEntry>> = use_signal(|| None);

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

    if let Some(pkg) = detail.read().clone() {
        return rsx! {
            PackageDetail {
                package: pkg,
                on_back: move |_| detail.set(None),
            }
        };
    }

    let tab = active_tab.read().clone();
    let kind_filter = tab.kind_filter();

    // ALL_TABS without Settings → regular sidebar items
    let sidebar_items: Vec<FsnSidebarItem> = ALL_TABS.iter()
        .filter(|t| !matches!(t, StoreTab::Settings))
        .map(|t| FsnSidebarItem::new(t.id(), t.icon(), t.label()))
        .collect();

    // Settings is always pinned at the bottom
    let pinned_items: Vec<FsnSidebarItem> = vec![
        FsnSidebarItem::new(
            StoreTab::Settings.id(),
            StoreTab::Settings.icon(),
            StoreTab::Settings.label(),
        ),
    ];

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-store",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fsn-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fsn-border); \
                        flex-shrink: 0; background: var(--fsn-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fsn-text-primary);",
                    {fsn_i18n::t("store.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

            // Left sidebar navigation — Settings pinned at bottom
            FsnSidebar {
                items:        sidebar_items,
                pinned_items,
                active_id:    active_tab.read().id().to_string(),
                on_select:    move |id: String| active_tab.set(StoreTab::from_id(&id)),
            }

            // Right content area
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                // Search header (hidden on Settings)
                if *active_tab.read() != StoreTab::Settings {
                    div {
                        style: "padding: 16px; background: var(--fsn-color-bg-surface); \
                                border-bottom: 1px solid var(--fsn-color-border-default);",
                        input {
                            r#type: "search",
                            placeholder: fsn_i18n::t("store.search_placeholder"),
                            style: "width: 100%; padding: 8px 12px; \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); font-size: 14px; \
                                    background: var(--fsn-bg-input); color: var(--fsn-text-primary);",
                            oninput: move |e| *search.write() = e.value(),
                        }
                    }
                }

                // Content
                div {
                    style: "flex: 1; overflow: auto; padding: 16px;",
                    match *active_tab.read() {
                        StoreTab::Installed => rsx! {
                            InstalledList {
                                catalog_versions: catalog_versions.read().clone(),
                            }
                        },
                        StoreTab::Updates => rsx! {
                            UpdatesList {
                                catalog_versions: catalog_versions.read().clone(),
                            }
                        },
                        StoreTab::Settings => rsx! {
                            StoreSettings {}
                        },
                        _ => rsx! {
                            PackageBrowser {
                                search: search.read().clone(),
                                kind: kind_filter,
                                on_select: move |pkg| detail.set(Some(pkg)),
                            }
                        },
                    }
                }
            }
            } // end sidebar + content row
        }
    }
}

#[component]
fn UpdatesList(catalog_versions: Vec<(String, String)>) -> Element {
    rsx! {
        div {
            style: "text-align: center; color: var(--fsn-text-muted); padding: 48px;",
            p { style: "font-size: 24px; margin-bottom: 12px;", "↑" }
            p { style: "margin-bottom: 8px;", {fsn_i18n::t("store.updates.no_metadata")} }
            p { style: "font-size: 13px;",
                "Run "
                code { style: "background: var(--fsn-bg-elevated); padding: 2px 6px; border-radius: 4px;",
                    "fsn deploy"
                }
                " "
                {fsn_i18n::t("store.updates.run_deploy")}
            }
            if !catalog_versions.is_empty() {
                p { style: "margin-top: 16px; font-size: 13px;",
                    {fsn_i18n::t_with("store.updates.catalog_count", &[("n", catalog_versions.len().to_string().as_str())])}
                }
            }
        }
    }
}
