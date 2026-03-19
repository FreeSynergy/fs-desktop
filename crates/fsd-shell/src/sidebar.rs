/// ShellSidebar — left-side navigation panel for the desktop shell.
/// Uses the FsnSidebar CSS class (icons-only 48px, expands to 220px on hover).
use dioxus::prelude::*;
use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
use fsn_components::{FsnSidebarItem, FsnSidebar};
use fsn_i18n;

/// A single navigation item in the sidebar.
/// Items with non-empty `children` are rendered as folders (bundles).
#[derive(Clone, PartialEq, Debug)]
pub struct SidebarNavItem {
    pub id:       String,
    pub label:    String,
    pub icon:     String,
    pub children: Vec<SidebarNavItem>,
}

/// A section grouping navigation items.
#[derive(Clone, PartialEq, Debug)]
pub struct SidebarSection {
    pub label: &'static str,
    pub items: Vec<SidebarNavItem>,
}

// ── OOP Trait ────────────────────────────────────────────────────────────────

/// Any type that can present itself as a sidebar navigation item.
/// Programs expose their own id, icon, and label — the sidebar just renders them.
pub trait SidebarEntry {
    fn nav_item(&self) -> SidebarNavItem;
}

/// An installed app or manager package provides its own nav item.
impl SidebarEntry for InstalledPackage {
    fn nav_item(&self) -> SidebarNavItem {
        let key   = format!("shell.nav.{}", self.id);
        let label = fsn_i18n::t(&key);
        let label = if label == key { self.name.clone() } else { label };
        SidebarNavItem {
            id:       self.id.clone(),
            label,
            icon:     self.icon.clone(),
            children: vec![],
        }
    }
}

/// A bundle groups several installed packages under a single folder entry.
/// The bundle itself exposes its own id, icon, and label — OOP, no hard-coding.
pub struct ManagerBundle(pub Vec<InstalledPackage>);

impl SidebarEntry for ManagerBundle {
    fn nav_item(&self) -> SidebarNavItem {
        SidebarNavItem {
            id:       "managers-folder".into(),
            label:    fsn_i18n::t("shell.nav.managers"),
            icon:     ICON_MANAGERS.into(),
            children: self.0.iter().map(|m| m.nav_item()).collect(),
        }
    }
}

// ── Dynamic registry reads ───────────────────────────────────────────────────

/// All installed apps (`kind = "app"`) as nav items.
fn installed_app_items() -> Vec<SidebarNavItem> {
    PackageRegistry::by_kind("app")
        .iter()
        .map(|pkg| pkg.nav_item())
        .collect()
}

/// Managers bundle — only returned when at least one manager is installed.
fn installed_manager_bundle() -> Option<SidebarNavItem> {
    let managers = PackageRegistry::by_kind("manager");
    if managers.is_empty() {
        None
    } else {
        Some(ManagerBundle(managers).nav_item())
    }
}

// ── Sidebar sections ─────────────────────────────────────────────────────────

const ICON_MANAGERS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>"#;
const ICON_SETTINGS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"#;

/// Default sidebar sections for the shell.
///
/// Only shows installed apps from PackageRegistry (`kind = "app"`) and,
/// if present, the managers bundle (`kind = "manager"`).
/// No hardcoded system items — everything must be installed first.
pub fn default_sidebar_sections() -> Vec<SidebarSection> {
    let mut items = installed_app_items();

    // Managers folder — only shows when at least one manager is installed.
    if let Some(bundle) = installed_manager_bundle() {
        items.push(bundle);
    }

    vec![SidebarSection { label: "Apps", items }]
}

// ── Component ────────────────────────────────────────────────────────────────

/// Converts a `SidebarNavItem` into a `FsnSidebarItem`, recursively for folders.
fn nav_item_to_fsn(item: &SidebarNavItem) -> FsnSidebarItem {
    if item.children.is_empty() {
        FsnSidebarItem::new(item.id.clone(), item.icon.clone(), item.label.clone())
    } else {
        let children = item.children.iter().map(nav_item_to_fsn).collect();
        FsnSidebarItem::folder(item.id.clone(), item.icon.clone(), item.label.clone(), children)
    }
}

/// Shell sidebar navigation — collapsible (48px → 220px on hover), FsnSidebar style.
/// Settings is always pinned at the bottom of the sidebar.
#[component]
pub fn ShellSidebar(
    sections:  Vec<SidebarSection>,
    active_id: String,
    on_select: EventHandler<String>,
) -> Element {
    let items: Vec<FsnSidebarItem> = sections.iter()
        .flat_map(|s| s.items.iter().map(nav_item_to_fsn))
        .collect();

    let pinned_items = vec![
        FsnSidebarItem::new("settings", ICON_SETTINGS, fsn_i18n::t("shell.nav.settings")),
    ];

    rsx! {
        FsnSidebar {
            items,
            pinned_items,
            active_id,
            on_select,
        }
    }
}
