/// ShellSidebar — left-side navigation panel for the desktop shell.
/// Uses the FsnSidebar CSS class (icons-only 48px, expands to 220px on hover).
use dioxus::prelude::*;
use fsn_components::{FsnSidebarItem, FsnSidebar};

/// A single navigation item in the sidebar.
#[derive(Clone, PartialEq, Debug)]
pub struct SidebarNavItem {
    pub id: String,
    pub label: String,
    pub icon: String,
}

/// A section grouping navigation items.
#[derive(Clone, PartialEq, Debug)]
pub struct SidebarSection {
    pub label: &'static str,
    pub items: Vec<SidebarNavItem>,
}

/// Default sidebar sections for the shell.
pub fn default_sidebar_sections() -> Vec<SidebarSection> {
    vec![
        SidebarSection {
            label: "Apps",
            items: vec![
                SidebarNavItem { id: "tasks".into(),     label: "Tasks".into(),     icon: "📋".into() },
                SidebarNavItem { id: "bots".into(),      label: "Bots".into(),      icon: "🤖".into() },
                SidebarNavItem { id: "conductor".into(), label: "Conductor".into(), icon: "🎛".into() },
                SidebarNavItem { id: "store".into(),     label: "Store".into(),     icon: "📦".into() },
                SidebarNavItem { id: "studio".into(),    label: "Studio".into(),    icon: "🔧".into() },
            ],
        },
        SidebarSection {
            label: "System",
            items: vec![
                SidebarNavItem { id: "settings".into(), label: "Settings".into(),      icon: "⚙".into() },
                SidebarNavItem { id: "profile".into(),  label: "Profile".into(),       icon: "👤".into() },
                SidebarNavItem { id: "ai".into(),       label: "AI Assistant".into(),  icon: "🤖".into() },
                SidebarNavItem { id: "help".into(),     label: "Help".into(),          icon: "❓".into() },
            ],
        },
    ]
}

/// Shell sidebar navigation — collapsible (48px → 220px on hover), FsnSidebar style.
#[component]
pub fn ShellSidebar(
    sections: Vec<SidebarSection>,
    active_id: String,
    on_select: EventHandler<String>,
) -> Element {
    // Flatten all sections into a single item list for FsnSidebar
    let items: Vec<FsnSidebarItem> = sections.iter()
        .flat_map(|s| s.items.iter().map(|item| {
            FsnSidebarItem::new(item.id.clone(), item.icon.clone(), item.label.clone())
        }))
        .collect();

    rsx! {
        FsnSidebar {
            items,
            active_id,
            on_select,
        }
    }
}
