/// ShellSidebar — left-side navigation panel for the desktop shell.
/// Uses the FsSidebar CSS class (icons-only 48px, expands to 220px on hover).
use dioxus::prelude::*;
use fs_db_desktop::package_registry::{InstalledPackage, PackageRegistry};
use fs_components::{FsSidebarItem, FsSidebar};
use fs_i18n;

use crate::icons::{ICON_MANAGERS, ICON_SETTINGS};

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
        let label = fs_i18n::t(&key);
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
            label:    fs_i18n::t("shell.nav.managers"),
            icon:     ICON_MANAGERS.into(),
            children: self.0.iter().map(|m| m.nav_item()).collect(),
        }
    }
}

// ── Dynamic registry reads ───────────────────────────────────────────────────

/// All installed apps (`kind = "app"`) as nav items.
/// `fs-desktop` is excluded — it is the shell itself, not an openable app.
fn installed_app_items() -> Vec<SidebarNavItem> {
    PackageRegistry::by_kind("app")
        .iter()
        .filter(|pkg| pkg.id != "fs-desktop")
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

/// Converts a `SidebarNavItem` into a `FsSidebarItem`, recursively for folders.
fn nav_item_to_fsn(item: &SidebarNavItem) -> FsSidebarItem {
    if item.children.is_empty() {
        FsSidebarItem::new(item.id.clone(), item.icon.clone(), item.label.clone())
    } else {
        let children = item.children.iter().map(nav_item_to_fsn).collect();
        FsSidebarItem::folder(item.id.clone(), item.icon.clone(), item.label.clone(), children)
    }
}

/// Shell sidebar navigation — collapsible (48px → 220px on hover), FsSidebar style.
/// Settings is pinned at the bottom only when the Desktop package is installed.
#[component]
pub fn ShellSidebar(
    sections:  Vec<SidebarSection>,
    active_id: String,
    on_select: EventHandler<String>,
) -> Element {
    let items: Vec<FsSidebarItem> = sections.iter()
        .flat_map(|s| s.items.iter().map(nav_item_to_fsn))
        .collect();

    let pinned_items: Vec<FsSidebarItem> = if PackageRegistry::is_installed("fs-desktop") {
        vec![FsSidebarItem::new("settings", ICON_SETTINGS, fs_i18n::t("shell.nav.settings"))]
    } else {
        vec![]
    };

    rsx! {
        FsSidebar {
            items,
            pinned_items,
            active_id,
            on_select,
        }
    }
}
