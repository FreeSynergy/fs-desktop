/// ShellHeader — 60px fixed header with breadcrumbs and user avatar menu.
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

/// Shell header component.
#[component]
pub fn ShellHeader(
    breadcrumbs: Vec<Breadcrumb>,
    user_name: String,
    user_avatar: Option<String>,
) -> Element {
    rsx! {
        header {
            class: "fsd-header",
            style: "display: flex; align-items: center; height: 60px; padding: 0 16px; gap: 16px; \
                    background: var(--fsn-color-bg-sidebar, #0f172a); \
                    border-bottom: 1px solid var(--fsn-color-border-default, #334155); \
                    z-index: 100; flex-shrink: 0;",

            // Brand
            div {
                style: "display: flex; align-items: center; flex-shrink: 0; \
                        padding-right: 16px; border-right: 1px solid var(--fsn-color-border-default, #334155);",
                span {
                    style: "font-size: 15px; font-weight: 700; color: var(--fsn-color-primary, #06b6d4); \
                            white-space: nowrap; letter-spacing: -0.01em;",
                    "FreeSynergy"
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

#[component]
fn Breadcrumbs(items: Vec<Breadcrumb>) -> Element {
    rsx! {
        nav {
            style: "display: flex; align-items: center; gap: 6px; overflow: hidden;",
            for (idx, crumb) in items.iter().enumerate() {
                if idx > 0 {
                    span {
                        style: "color: var(--fsn-color-text-muted, #64748b); font-size: 12px; flex-shrink: 0;",
                        "›"
                    }
                }
                span {
                    style: "font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; \
                            color: {if idx + 1 == items.len() { \"var(--fsn-color-text-primary, #e2e8f0)\" } else { \"var(--fsn-color-text-muted, #94a3b8)\" }};",
                    if let Some(icon) = &crumb.icon {
                        span { style: "margin-right: 4px;", "{icon}" }
                    }
                    "{crumb.label}"
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
