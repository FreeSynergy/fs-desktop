/// ShellHeader — 60px fixed header with menu bar, breadcrumbs and user avatar menu.
use dioxus::prelude::*;

/// A single breadcrumb entry.
#[derive(Clone, PartialEq, Debug)]
pub struct Breadcrumb {
    pub label: String,
    pub icon: Option<String>,
}

impl Breadcrumb {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), icon: None }
    }
}

/// A menu item descriptor for the menu bar.
#[derive(Clone, PartialEq, Debug)]
pub struct MenuItem {
    pub label: &'static str,
    pub items: Vec<MenuAction>,
}

/// A single action in a submenu (leaf item only).
#[derive(Clone, PartialEq, Debug)]
pub struct SubAction {
    pub label: &'static str,
    pub id: &'static str,
}

/// A single action in a menu dropdown.
#[derive(Clone, PartialEq, Debug)]
pub enum MenuAction {
    Action { label: &'static str, shortcut: Option<&'static str>, id: &'static str },
    SubMenu { label: &'static str, items: Vec<SubAction> },
    Separator,
}

fn default_menu() -> Vec<MenuItem> {
    vec![
        MenuItem {
            label: "FreeSynergy",
            items: vec![
                MenuAction::Action { label: "About FreeSynergy…",    shortcut: None,          id: "about" },
                MenuAction::Action { label: "Settings",              shortcut: Some("Ctrl+,"), id: "settings" },
                MenuAction::Separator,
                MenuAction::Action { label: "Quit",                  shortcut: Some("Ctrl+Q"), id: "quit" },
            ],
        },
        MenuItem {
            label: "File",
            items: vec![
                MenuAction::Action { label: "New Project",   shortcut: Some("Ctrl+N"), id: "new-project" },
                MenuAction::Action { label: "Open Project…", shortcut: None,           id: "open-project" },
                MenuAction::Separator,
                MenuAction::Action { label: "Export…", shortcut: None, id: "export" },
                MenuAction::Action { label: "Import…", shortcut: None, id: "import" },
            ],
        },
        MenuItem {
            label: "View",
            items: vec![
                MenuAction::Action { label: "Toggle Sidebar",  shortcut: Some("Ctrl+B"), id: "toggle-sidebar" },
                MenuAction::Action { label: "Fullscreen",      shortcut: Some("F11"),    id: "fullscreen" },
                MenuAction::Separator,
                MenuAction::SubMenu {
                    label: "Sidebar Position ▸",
                    items: vec![
                        SubAction { label: "Left",   id: "sidebar-left" },
                        SubAction { label: "Right",  id: "sidebar-right" },
                        SubAction { label: "Top",    id: "sidebar-top" },
                        SubAction { label: "Bottom", id: "sidebar-bottom" },
                    ],
                },
                MenuAction::SubMenu {
                    label: "Theme ▸",
                    items: vec![
                        SubAction { label: "Midnight Blue", id: "theme-midnight-blue" },
                        SubAction { label: "Light",         id: "theme-light" },
                    ],
                },
                MenuAction::SubMenu {
                    label: "Rendering Mode ▸",
                    items: vec![
                        SubAction { label: "Desktop", id: "render-desktop" },
                        SubAction { label: "Native",  id: "render-native" },
                        SubAction { label: "Web",     id: "render-web" },
                    ],
                },
            ],
        },
        MenuItem {
            label: "Services",
            items: vec![
                MenuAction::Action { label: "Start All",            shortcut: None,           id: "start-all" },
                MenuAction::Action { label: "Stop All",             shortcut: None,           id: "stop-all" },
                MenuAction::Separator,
                MenuAction::Action { label: "Install Service…",     shortcut: Some("Ctrl+I"), id: "install-service" },
                MenuAction::Action { label: "Open Store",           shortcut: Some("Ctrl+S"), id: "open-store" },
            ],
        },
        MenuItem {
            label: "Help",
            items: vec![
                MenuAction::Action { label: "Help",               shortcut: Some("F1"), id: "help" },
                MenuAction::Action { label: "Keyboard Shortcuts", shortcut: None,       id: "shortcuts" },
                MenuAction::Action { label: "Documentation",      shortcut: None,       id: "documentation" },
                MenuAction::Separator,
                MenuAction::Action { label: "Report a Bug…",     shortcut: None,       id: "report-bug" },
            ],
        },
    ]
}

/// Shell header component.
#[component]
pub fn ShellHeader(
    breadcrumbs: Vec<Breadcrumb>,
    user_name: String,
    user_avatar: Option<String>,
    #[props(default)]
    on_menu_action: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        header {
            class: "fsd-header",
            style: "display: flex; align-items: center; height: 60px; padding: 0 8px 0 16px; gap: 8px; \
                    background: var(--fsn-bg-sidebar, #0a0f1a); \
                    border-bottom: 1px solid var(--fsn-border, rgba(148,170,200,0.18)); \
                    z-index: 100; flex-shrink: 0;",

            // Brand
            div {
                style: "display: flex; align-items: center; flex-shrink: 0; \
                        padding-right: 12px; border-right: 1px solid var(--fsn-border);",
                span {
                    style: "font-size: 14px; font-weight: 700; color: var(--fsn-primary); \
                            white-space: nowrap; letter-spacing: -0.01em;",
                    "FreeSynergy"
                }
            }

            // Menu bar
            MenuBar {
                menus: default_menu(),
                on_action: {
                    let on_menu_action = on_menu_action.clone();
                    move |id: String| {
                        if let Some(handler) = &on_menu_action {
                            handler.call(id);
                        }
                    }
                }
            }

            // Breadcrumbs
            Breadcrumbs { items: breadcrumbs }

            div { style: "flex: 1;" }

            // User avatar menu
            AvatarMenu { user_name, user_avatar }
        }
    }
}

// ── Menu Bar ──────────────────────────────────────────────────────────────────

#[component]
fn MenuBar(menus: Vec<MenuItem>, on_action: EventHandler<String>) -> Element {
    let mut open_idx: Signal<Option<usize>> = use_signal(|| None);

    rsx! {
        nav {
            class: "fsd-menubar",
            style: "display: flex; align-items: center; gap: 2px; flex-shrink: 0;",
            for (idx, menu) in menus.iter().enumerate() {
                MenuBarItem {
                    key: "{menu.label}",
                    menu: menu.clone(),
                    is_open: *open_idx.read() == Some(idx),
                    on_toggle: {
                        move |_| {
                            let current = *open_idx.read();
                            *open_idx.write() = if current == Some(idx) { None } else { Some(idx) };
                        }
                    },
                    on_close: move |_| *open_idx.write() = None,
                    on_action: on_action.clone(),
                }
            }
        }
    }
}

#[component]
fn MenuBarItem(
    menu: MenuItem,
    is_open: bool,
    on_toggle: EventHandler<MouseEvent>,
    on_close: EventHandler<()>,
    on_action: EventHandler<String>,
) -> Element {
    let bg = if is_open { "var(--fsn-bg-elevated)" } else { "transparent" };
    rsx! {
        div { style: "position: relative;",
            button {
                style: "background: {bg}; border: none; cursor: pointer; padding: 4px 10px; \
                        border-radius: var(--fsn-radius-sm); font-size: 13px; \
                        color: var(--fsn-text-secondary); white-space: nowrap; \
                        transition: background 150ms ease;",
                onclick: on_toggle,
                "{menu.label}"
            }
            if is_open {
                MenuDropdown {
                    items: menu.items.clone(),
                    on_close,
                    on_action,
                }
            }
        }
    }
}

#[component]
fn MenuDropdown(
    items: Vec<MenuAction>,
    on_close: EventHandler<()>,
    on_action: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            style: "position: absolute; top: calc(100% + 4px); left: 0; \
                    background: var(--fsn-bg-elevated); \
                    border: 1px solid var(--fsn-border); border-radius: var(--fsn-radius-md); \
                    min-width: 200px; z-index: 500; padding: 4px 0; \
                    box-shadow: var(--fsn-shadow);",
            // Close backdrop
            div {
                style: "position: fixed; inset: 0; z-index: -1;",
                onclick: move |_| on_close.call(()),
            }
            for item in &items {
                match item {
                    MenuAction::Separator => rsx! {
                        hr { style: "border: none; border-top: 1px solid var(--fsn-border); margin: 4px 0;" }
                    },
                    MenuAction::Action { label, shortcut, id } => {
                        let id_owned = id.to_string();
                        rsx! {
                            button {
                                style: "display: flex; align-items: center; justify-content: space-between; \
                                        width: 100%; padding: 6px 16px; background: none; border: none; \
                                        cursor: pointer; font-size: 13px; text-align: left; \
                                        color: var(--fsn-text-primary); gap: 24px;",
                                onclick: move |_| {
                                    on_action.call(id_owned.clone());
                                    on_close.call(());
                                },
                                span { "{label}" }
                                if let Some(sc) = shortcut {
                                    span {
                                        style: "font-size: 11px; color: var(--fsn-text-muted); flex-shrink: 0;",
                                        "{sc}"
                                    }
                                }
                            }
                        }
                    }
                    MenuAction::SubMenu { label, items } => {
                        let sub_items = items.clone();
                        rsx! {
                            SubMenuRow {
                                label,
                                items: sub_items,
                                on_action: on_action.clone(),
                                on_close: on_close.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A menu row that reveals a flyout submenu on hover.
#[component]
fn SubMenuRow(
    label: &'static str,
    items: Vec<SubAction>,
    on_action: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        div {
            style: "position: relative;",
            onmouseenter: move |_| *hovered.write() = true,
            onmouseleave: move |_| *hovered.write() = false,

            // Row button (no onclick — opens on hover)
            button {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        width: 100%; padding: 6px 16px; background: none; border: none; \
                        cursor: default; font-size: 13px; text-align: left; \
                        color: var(--fsn-text-primary); gap: 24px;",
                span { "{label}" }
                span {
                    style: "font-size: 11px; color: var(--fsn-text-muted); flex-shrink: 0;",
                    "▶"
                }
            }

            // Flyout submenu (shown when hovered)
            if *hovered.read() {
                div {
                    style: "position: absolute; top: 0; left: 100%; \
                            background: var(--fsn-bg-elevated); \
                            border: 1px solid var(--fsn-border); border-radius: var(--fsn-radius-md); \
                            min-width: 160px; z-index: 600; padding: 4px 0; \
                            box-shadow: var(--fsn-shadow);",
                    for sub in items.clone() {
                        button {
                            key: "{sub.id}",
                            style: "display: flex; align-items: center; width: 100%; \
                                    padding: 6px 16px; background: none; border: none; \
                                    cursor: pointer; font-size: 13px; text-align: left; \
                                    color: var(--fsn-text-primary);",
                            onclick: {
                                let id = sub.id.to_string();
                                move |_| {
                                    on_action.call(id.clone());
                                    on_close.call(());
                                }
                            },
                            "{sub.label}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Breadcrumbs(items: Vec<Breadcrumb>) -> Element {
    rsx! {
        nav {
            style: "display: flex; align-items: center; gap: 6px; overflow: hidden;",
            for (idx, crumb) in items.iter().enumerate() {
                {
                    let color = if idx + 1 == items.len() {
                        "var(--fsn-color-text-primary, #e2e8f0)"
                    } else {
                        "var(--fsn-color-text-muted, #94a3b8)"
                    };
                    let crumb_style = format!(
                        "font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: {color};"
                    );
                    rsx! {
                        if idx > 0 {
                            span {
                                style: "color: var(--fsn-color-text-muted, #64748b); font-size: 12px; flex-shrink: 0;",
                                "›"
                            }
                        }
                        span {
                            style: "{crumb_style}",
                            if let Some(icon) = &crumb.icon {
                                span { style: "margin-right: 4px;", "{icon}" }
                            }
                            "{crumb.label}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AvatarMenu(user_name: String, user_avatar: Option<String>) -> Element {
    let mut open    = use_signal(|| false);
    let initial: String = user_name.chars().next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    rsx! {
        div { style: "position: relative; flex-shrink: 0;",
            button {
                style: "display: flex; align-items: center; gap: 8px; background: none; border: none; \
                        cursor: pointer; padding: 4px 8px; border-radius: 6px; \
                        color: var(--fsn-color-text-primary, #e2e8f0);",
                onclick: move |_| { let v = *open.read(); *open.write() = !v; },

                div {
                    style: "width: 30px; height: 30px; border-radius: 50%; flex-shrink: 0; overflow: hidden; \
                            background: var(--fsn-color-primary, #06b6d4); \
                            display: flex; align-items: center; justify-content: center; \
                            font-size: 13px; font-weight: 600; color: #fff;",
                    if let Some(url) = &user_avatar {
                        img { src: "{url}", width: "30", height: "30", style: "border-radius: 50%;" }
                    } else {
                        "{initial}"
                    }
                }
                span {
                    style: "font-size: 13px; max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{user_name}"
                }
                span { style: "font-size: 10px; color: var(--fsn-color-text-muted);", "▾" }
            }

            if *open.read() {
                AvatarDropdown { on_close: move |_| *open.write() = false }
            }
        }
    }
}

#[component]
fn AvatarDropdown(on_close: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            style: "position: absolute; right: 0; top: calc(100% + 4px); \
                    background: var(--fsn-color-bg-panel, #1e293b); \
                    border: 1px solid var(--fsn-color-border-default, #334155); \
                    border-radius: 8px; min-width: 160px; z-index: 300; \
                    box-shadow: 0 8px 24px rgba(0,0,0,0.5); padding: 4px 0;",
            AvatarMenuItem { icon: "👤", label: "Profile",  on_click: on_close.clone() }
            AvatarMenuItem { icon: "⚙",  label: "Settings", on_click: on_close.clone() }
            hr { style: "border: none; border-top: 1px solid var(--fsn-color-border-default, #334155); margin: 4px 0;" }
            AvatarMenuItem { icon: "⎋",  label: "Sign out", on_click: on_close }
        }
    }
}

#[component]
fn AvatarMenuItem(
    icon: &'static str,
    label: &'static str,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; width: 100%; padding: 8px 16px; \
                    background: none; border: none; cursor: pointer; font-size: 13px; text-align: left; \
                    color: var(--fsn-color-text-primary, #e2e8f0);",
            onclick: on_click,
            span { style: "font-size: 14px;", "{icon}" }
            span { "{label}" }
        }
    }
}
