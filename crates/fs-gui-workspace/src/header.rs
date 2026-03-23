/// ShellHeader — 60px fixed header with menu bar, breadcrumbs and user avatar menu.
use dioxus::prelude::*;
use fs_i18n;
use crate::icons::{ICON_PROFILE, ICON_SETTINGS, ICON_SIGN_OUT, ICON_CHEVRON_RIGHT, ICON_CHEVRON_DOWN};
use crate::notification::{NotificationBell, NotificationHistory};

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
    pub label: String,
    pub items: Vec<MenuAction>,
}

/// A single action in a submenu (leaf item only).
#[derive(Clone, PartialEq, Debug)]
pub struct SubAction {
    pub label: String,
    pub id: &'static str,
}

/// A single action in a menu dropdown.
#[derive(Clone, PartialEq, Debug)]
pub enum MenuAction {
    Action { label: String, shortcut: Option<&'static str>, id: &'static str },
    SubMenu { label: String, items: Vec<SubAction> },
    Separator,
}

fn default_menu() -> Vec<MenuItem> {
    vec![
        MenuItem {
            label: "FreeSynergy".into(),
            items: vec![
                MenuAction::Action { label: fs_i18n::t("shell.menu.about").into(),    shortcut: None,               id: "about" },
                MenuAction::Action { label: fs_i18n::t("settings.title").into(),      shortcut: Some("Ctrl+,"),     id: "settings" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.launcher").into(), shortcut: Some("Ctrl+Space"), id: "launcher" },
                MenuAction::Separator,
                MenuAction::Action { label: fs_i18n::t("shell.menu.quit").into(),     shortcut: Some("Ctrl+Q"),     id: "quit" },
            ],
        },
        MenuItem {
            label: fs_i18n::t("shell.menu.view").into(),
            items: vec![
                MenuAction::Action { label: fs_i18n::t("shell.menu.fullscreen").into(), shortcut: Some("F11"), id: "fullscreen" },
                MenuAction::Separator,
                MenuAction::SubMenu {
                    label: fs_i18n::t("shell.menu.theme").into(),
                    items: vec![
                        SubAction { label: "Midnight Blue".into(), id: "theme-midnight-blue" },
                        SubAction { label: "Cloud White".into(),   id: "theme-cloud-white" },
                        SubAction { label: "Cupertino".into(),     id: "theme-cupertino" },
                        SubAction { label: "Nordic".into(),        id: "theme-nordic" },
                        SubAction { label: "Rose Pine".into(),     id: "theme-rose-pine" },
                    ],
                },
                MenuAction::SubMenu {
                    label: fs_i18n::t("shell.menu.rendering_mode").into(),
                    items: vec![
                        SubAction { label: "Desktop".into(), id: "render-desktop" },
                        SubAction { label: "Web".into(),     id: "render-web" },
                    ],
                },
            ],
        },
        MenuItem {
            label: fs_i18n::t("shell.menu.services").into(),
            items: vec![
                MenuAction::Action { label: fs_i18n::t("shell.menu.open_container").into(), shortcut: None, id: "open-container-app" },
                MenuAction::Separator,
                MenuAction::Action { label: fs_i18n::t("shell.menu.start_all").into(), shortcut: None, id: "start-all" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.stop_all").into(),  shortcut: None, id: "stop-all" },
            ],
        },
        MenuItem {
            label: fs_i18n::t("shell.menu.tools").into(),
            items: vec![
                MenuAction::Action { label: fs_i18n::t("shell.menu.open_store").into(),       shortcut: Some("Ctrl+S"), id: "open-store" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.open_studio").into(),      shortcut: None,           id: "open-studio" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.open_tasks").into(),       shortcut: Some("Ctrl+T"), id: "open-tasks" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.open_bots").into(),        shortcut: None,           id: "open-bots" },
                MenuAction::Separator,
                MenuAction::Action { label: fs_i18n::t("shell.menu.install_package").into(), shortcut: Some("Ctrl+I"), id: "install-package" },
            ],
        },
        MenuItem {
            label: fs_i18n::t("shell.menu.help").into(),
            items: vec![
                MenuAction::Action { label: fs_i18n::t("shell.menu.help").into(),               shortcut: Some("F1"), id: "help" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.keyboard_shortcuts").into(), shortcut: None,       id: "shortcuts" },
                MenuAction::Action { label: fs_i18n::t("shell.menu.documentation").into(),      shortcut: None,       id: "documentation" },
                MenuAction::Separator,
                MenuAction::Action { label: fs_i18n::t("shell.menu.report_bug").into(), shortcut: None, id: "report-bug" },
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
    #[props(default)]
    history: NotificationHistory,
    #[props(default)]
    on_mark_read: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        header {
            class: "fs-header",
            style: "display: flex; align-items: center; height: 60px; padding: 0 8px 0 16px; gap: 8px; \
                    background: var(--fs-bg-sidebar, #0a0f1a); \
                    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18)); \
                    z-index: 100; flex-shrink: 0; position: relative;",

            // Drag handle + OS window controls (desktop only).
            OsWindowDragHandle {}

            // Brand
            div {
                style: "display: flex; align-items: center; flex-shrink: 0; \
                        padding-right: 12px; border-right: 1px solid var(--fs-border);",
                span {
                    style: "font-size: 14px; font-weight: 700; color: var(--fs-primary); \
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

            // Notification bell
            NotificationBell {
                history,
                on_mark_read: move |_| {
                    if let Some(h) = &on_mark_read {
                        h.call(());
                    }
                },
            }

            // User avatar menu
            AvatarMenu { user_name, user_avatar }

            // OS window controls (maximize + close) — desktop only.
            OsWindowControls {}
        }
    }
}

/// Invisible drag handle that covers the full header area.
/// Sits at z-index: -1 so buttons above it still receive events.
/// When `decorations = false`, this allows dragging the OS window by the header.
/// Double-click toggles maximize / restore.
#[cfg(feature = "desktop")]
#[component]
fn OsWindowDragHandle() -> Element {
    rsx! {
        div {
            style: "position: absolute; inset: 0; z-index: -1;",
            onmousedown: move |_| {
                dioxus::desktop::window().drag();
            },
            ondblclick: move |_| {
                dioxus::desktop::window().toggle_maximized();
            },
        }
    }
}

/// Stub for non-desktop builds.
#[cfg(not(feature = "desktop"))]
#[component]
fn OsWindowDragHandle() -> Element {
    rsx! {}
}

/// OS-level window controls: maximize/restore + close.
/// `minimize()` is not exposed in dioxus-desktop 0.6; toggle_maximized and close are available.
#[cfg(feature = "desktop")]
#[component]
fn OsWindowControls() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 2px; flex-shrink: 0; margin-left: 4px;",
            // Maximize / Restore
            button {
                class: "fs-window-btn",
                title: fs_i18n::t("shell.window.maximize").to_string(),
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                onclick: move |_| {
                    dioxus::desktop::window().toggle_maximized();
                },
                dangerous_inner_html: r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="6" height="6" rx="0.5"/></svg>"#,
            }
            // Close
            button {
                class: "fs-window-btn fs-window-btn--close",
                title: fs_i18n::t("shell.window.close").to_string(),
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                onclick: move |_| {
                    dioxus::desktop::window().close();
                },
                dangerous_inner_html: r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="2" y1="2" x2="8" y2="8"/><line x1="8" y1="2" x2="2" y2="8"/></svg>"#,
            }
        }
    }
}

/// Stub for non-desktop builds.
#[cfg(not(feature = "desktop"))]
#[component]
fn OsWindowControls() -> Element {
    rsx! {}
}

// ── Menu Bar ──────────────────────────────────────────────────────────────────

#[component]
fn MenuBar(menus: Vec<MenuItem>, on_action: EventHandler<String>) -> Element {
    let mut open_idx: Signal<Option<usize>> = use_signal(|| None);

    rsx! {
        nav {
            class: "fs-menubar",
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
    let bg = if is_open { "var(--fs-bg-elevated)" } else { "transparent" };
    rsx! {
        div { style: "position: relative;",
            button {
                style: "background: {bg}; border: none; cursor: pointer; padding: 4px 10px; \
                        border-radius: var(--fs-radius-sm); font-size: 13px; \
                        color: var(--fs-text-secondary); white-space: nowrap; \
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
                    background: var(--fs-bg-elevated); \
                    border: 1px solid var(--fs-border); border-radius: var(--fs-radius-md); \
                    min-width: 200px; z-index: 500; padding: 4px 0; \
                    box-shadow: var(--fs-shadow);",
            // Close backdrop
            div {
                style: "position: fixed; inset: 0; z-index: -1;",
                onclick: move |_| on_close.call(()),
            }
            for item in &items {
                match item {
                    MenuAction::Separator => rsx! {
                        hr { style: "border: none; border-top: 1px solid var(--fs-border); margin: 4px 0;" }
                    },
                    MenuAction::Action { label, shortcut, id } => {
                        let id_owned = id.to_string();
                        rsx! {
                            button {
                                style: "display: flex; align-items: center; justify-content: space-between; \
                                        width: 100%; padding: 6px 16px; background: none; border: none; \
                                        cursor: pointer; font-size: 13px; text-align: left; \
                                        color: var(--fs-text-primary); gap: 24px;",
                                onclick: move |_| {
                                    on_action.call(id_owned.clone());
                                    on_close.call(());
                                },
                                span { "{label}" }
                                if let Some(sc) = shortcut {
                                    span {
                                        style: "font-size: 11px; color: var(--fs-text-muted); flex-shrink: 0;",
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
    label: String,
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
                        color: var(--fs-text-primary); gap: 24px;",
                span { "{label}" }
                span {
                    style: "color: var(--fs-text-muted); flex-shrink: 0; display: flex; align-items: center;",
                    dangerous_inner_html: ICON_CHEVRON_RIGHT
                }
            }

            // Flyout submenu (shown when hovered)
            if *hovered.read() {
                div {
                    style: "position: absolute; top: 0; left: 100%; \
                            background: var(--fs-bg-elevated); \
                            border: 1px solid var(--fs-border); border-radius: var(--fs-radius-md); \
                            min-width: 160px; z-index: 600; padding: 4px 0; \
                            box-shadow: var(--fs-shadow);",
                    for sub in items.clone() {
                        button {
                            key: "{sub.id}",
                            style: "display: flex; align-items: center; width: 100%; \
                                    padding: 6px 16px; background: none; border: none; \
                                    cursor: pointer; font-size: 13px; text-align: left; \
                                    color: var(--fs-text-primary);",
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
                        "var(--fs-color-text-primary, #e2e8f0)"
                    } else {
                        "var(--fs-color-text-muted, #94a3b8)"
                    };
                    let crumb_style = format!(
                        "font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: {color};"
                    );
                    rsx! {
                        if idx > 0 {
                            span {
                                style: "color: var(--fs-color-text-muted, #64748b); font-size: 12px; flex-shrink: 0;",
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
                        color: var(--fs-color-text-primary, #e2e8f0);",
                onclick: move |_| { let v = *open.read(); *open.write() = !v; },

                div {
                    style: "width: 30px; height: 30px; border-radius: 50%; flex-shrink: 0; overflow: hidden; \
                            background: var(--fs-color-primary, #06b6d4); \
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
                span { style: "color: var(--fs-color-text-muted); display: flex; align-items: center;", dangerous_inner_html: ICON_CHEVRON_DOWN }
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
                    background: var(--fs-color-bg-panel, #1e293b); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    border-radius: 8px; min-width: 160px; z-index: 300; \
                    box-shadow: 0 8px 24px rgba(0,0,0,0.5); padding: 4px 0;",
            AvatarMenuItem { icon: ICON_PROFILE,   label: fs_i18n::t("shell.avatar.profile"),  on_click: on_close.clone() }
            AvatarMenuItem { icon: ICON_SETTINGS,  label: fs_i18n::t("settings.title"),         on_click: on_close.clone() }
            hr { style: "border: none; border-top: 1px solid var(--fs-color-border-default, #334155); margin: 4px 0;" }
            AvatarMenuItem { icon: ICON_SIGN_OUT,  label: fs_i18n::t("shell.avatar.sign_out"),  on_click: on_close }
        }
    }
}

#[component]
fn AvatarMenuItem(
    icon: &'static str,
    label: String,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; width: 100%; padding: 8px 16px; \
                    background: none; border: none; cursor: pointer; font-size: 13px; text-align: left; \
                    color: var(--fs-color-text-primary, #e2e8f0);",
            onclick: on_click,
            span { style: "display: flex; align-items: center; flex-shrink: 0;", dangerous_inner_html: icon }
            span { "{label}" }
        }
    }
}
