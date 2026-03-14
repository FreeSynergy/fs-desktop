/// ShellSidebar — left-side navigation panel for the desktop shell.
use dioxus::prelude::*;

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
                SidebarNavItem { id: "conductor".into(), label: "Conductor".into(), icon: "⚙".into() },
                SidebarNavItem { id: "store".into(),     label: "Store".into(),     icon: "📦".into() },
                SidebarNavItem { id: "studio".into(),    label: "Studio".into(),    icon: "🔧".into() },
            ],
        },
        SidebarSection {
            label: "System",
            items: vec![
                SidebarNavItem { id: "settings".into(), label: "Settings".into(), icon: "⚙".into() },
                SidebarNavItem { id: "profile".into(),  label: "Profile".into(),  icon: "👤".into() },
            ],
        },
    ]
}

/// Shell sidebar navigation.
#[component]
pub fn ShellSidebar(
    sections: Vec<SidebarSection>,
    active_id: String,
    collapsed: bool,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<()>,
) -> Element {
    let width = if collapsed { "48px" } else { "240px" };
    rsx! {
        nav {
            class: "fsd-sidebar",
            style: "width: {width}; background: var(--fsn-color-bg-sidebar, #0f172a); \
                    border-right: 1px solid var(--fsn-color-border-default, #334155); \
                    display: flex; flex-direction: column; overflow: hidden; \
                    transition: width 200ms ease; flex-shrink: 0;",

            // Collapse toggle
            div {
                style: "padding: 6px; display: flex; justify-content: {if collapsed { \"center\" } else { \"flex-end\" }};",
                button {
                    style: "background: none; border: none; cursor: pointer; \
                            color: var(--fsn-color-text-muted, #94a3b8); font-size: 14px; \
                            padding: 4px 6px; border-radius: 4px; line-height: 1;",
                    title: if collapsed { "Expand" } else { "Collapse" },
                    onclick: move |_| on_toggle.call(()),
                    if collapsed { "▶" } else { "◀" }
                }
            }

            // Navigation sections
            div {
                style: "flex: 1; overflow-y: auto; padding-bottom: 8px;",
                for section in &sections {
                    SidebarSectionGroup {
                        section: section.clone(),
                        active_id: active_id.clone(),
                        collapsed,
                        on_select: on_select.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarSectionGroup(
    section: SidebarSection,
    active_id: String,
    collapsed: bool,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            style: "padding: 4px 6px;",
            if !collapsed {
                div {
                    style: "padding: 6px 8px 2px; font-size: 10px; font-weight: 600; \
                            text-transform: uppercase; letter-spacing: 0.08em; \
                            color: var(--fsn-color-text-muted, #64748b);",
                    "{section.label}"
                }
            }
            for item in &section.items {
                SidebarItemBtn {
                    key: "{item.id}",
                    item: item.clone(),
                    is_active: item.id == active_id,
                    collapsed,
                    on_click: {
                        let id = item.id.clone();
                        move |_| on_select.call(id.clone())
                    },
                }
            }
        }
    }
}

#[component]
fn SidebarItemBtn(
    item: SidebarNavItem,
    is_active: bool,
    collapsed: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let bg     = if is_active { "var(--fsn-color-bg-overlay, #1e293b)" } else { "transparent" };
    let color  = if is_active { "var(--fsn-color-primary, #06b6d4)" } else { "var(--fsn-color-text-muted, #94a3b8)" };
    let border = if is_active { "2px solid var(--fsn-color-primary, #06b6d4)" } else { "2px solid transparent" };
    let justify = if collapsed { "center" } else { "flex-start" };
    let padding = if collapsed { "8px" } else { "8px 10px" };
    rsx! {
        button {
            title: "{item.label}",
            style: "display: flex; align-items: center; gap: 10px; width: 100%; \
                    padding: {padding}; border: none; border-left: {border}; border-radius: 6px; \
                    cursor: pointer; background: {bg}; color: {color}; margin-bottom: 1px; \
                    justify-content: {justify};",
            onclick: on_click,
            span { style: "font-size: 18px; flex-shrink: 0;", "{item.icon}" }
            if !collapsed {
                span {
                    style: "font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    "{item.label}"
                }
            }
        }
    }
}
