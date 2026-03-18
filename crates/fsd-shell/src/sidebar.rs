/// ShellSidebar — left-side navigation panel for the desktop shell.
/// Uses the FsnSidebar CSS class (icons-only 48px, expands to 220px on hover).
use dioxus::prelude::*;
use fsn_components::{FsnSidebarItem, FsnSidebar};
use fsn_i18n;

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
                SidebarNavItem { id: "tasks".into(),     label: fsn_i18n::t("shell.nav.tasks"),     icon: "📋".into() },
                SidebarNavItem { id: "bots".into(),      label: fsn_i18n::t("shell.nav.bots"),      icon: "🤖".into() },
                SidebarNavItem { id: "conductor".into(), label: fsn_i18n::t("shell.nav.conductor"), icon: "🎛".into() },
                SidebarNavItem { id: "browser".into(),   label: fsn_i18n::t("shell.nav.browser"),   icon: "🌐".into() },
                SidebarNavItem { id: "lenses".into(),    label: fsn_i18n::t("shell.nav.lenses"),    icon: "🔍".into() },
                SidebarNavItem { id: "store".into(),     label: fsn_i18n::t("shell.nav.store"),     icon: "📦".into() },
                SidebarNavItem { id: "builder".into(),   label: fsn_i18n::t("shell.nav.builder"),   icon: "🔧".into() },
            ],
        },
        SidebarSection {
            label: "System",
            items: vec![
                SidebarNavItem { id: "settings".into(), label: fsn_i18n::t("shell.nav.settings"),     icon: "⚙".into() },
                SidebarNavItem { id: "profile".into(),  label: fsn_i18n::t("shell.nav.profile"),      icon: "👤".into() },
                SidebarNavItem { id: "ai".into(),       label: fsn_i18n::t("shell.nav.ai_assistant"), icon: "🤖".into() },
                SidebarNavItem { id: "help".into(),     label: fsn_i18n::t("shell.nav.help"),         icon: "❓".into() },
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
