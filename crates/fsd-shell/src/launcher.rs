/// AppLauncher — full-screen overlay grid with search, opened via the taskbar ⊞ button.
use dioxus::prelude::*;

use crate::taskbar::AppEntry;

/// State exposed to the desktop for open/close.
#[derive(Clone, Default, PartialEq)]
pub struct LauncherState {
    pub open: bool,
    pub query: String,
}

impl LauncherState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if !self.open {
            self.query.clear();
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AppLauncherProps {
    /// All registered applications.
    pub apps: Vec<AppEntry>,
    /// Current search query (two-way via Signal in parent).
    pub query: String,
    pub on_query_change: EventHandler<String>,
    /// Called when an app is selected.
    pub on_launch: EventHandler<String>,
    /// Called when the overlay is dismissed (Escape / background click).
    pub on_close: EventHandler<()>,
}

/// Full-screen semi-transparent overlay with app grid + search bar.
#[component]
pub fn AppLauncher(props: AppLauncherProps) -> Element {
    let filtered: Vec<&AppEntry> = props
        .apps
        .iter()
        .filter(|a| {
            props.query.is_empty()
                || a.id.to_lowercase().contains(&props.query.to_lowercase())
                || a.label_key.to_lowercase().contains(&props.query.to_lowercase())
        })
        .collect();

    rsx! {
        // Backdrop
        div {
            class: "fsd-launcher__backdrop",
            style: "position: fixed; inset: 0; z-index: 9000; \
                    background: rgba(0,0,0,0.75); backdrop-filter: blur(8px); \
                    display: flex; flex-direction: column; align-items: center; \
                    justify-content: flex-start; padding-top: 80px;",
            onclick: move |_| props.on_close.call(()),

            // Inner panel — stopPropagation so click doesn't close
            div {
                class: "fsd-launcher__panel",
                style: "width: min(720px, 95vw); display: flex; flex-direction: column; gap: 24px;",
                onclick: move |evt: MouseEvent| evt.stop_propagation(),

                // ── FreeSynergy header ─────────────────────────────────────
                div {
                    style: "text-align: center; color: var(--fsn-color-primary, #06b6d4); font-size: 18px; font-weight: 600;",
                    "FreeSynergy.Node"
                    span {
                        style: "color: var(--fsn-color-text-muted, #94a3b8); font-size: 12px; margin-left: 8px;",
                        "by KalEl"
                    }
                }

                // ── Search bar ─────────────────────────────────────────────
                input {
                    class: "fsd-launcher__search",
                    style: "width: 100%; padding: 12px 16px; border-radius: 8px; \
                            background: var(--fsn-color-bg-input, #1e293b); \
                            border: 1px solid var(--fsn-color-border-default, #334155); \
                            color: var(--fsn-color-text-primary, #e2e8f0); font-size: 15px; \
                            outline: none; box-sizing: border-box;",
                    r#type: "text",
                    placeholder: "Search apps…",
                    value: props.query.clone(),
                    autofocus: true,
                    oninput: move |evt| props.on_query_change.call(evt.value()),
                    onkeydown: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Escape {
                            props.on_close.call(());
                        }
                    },
                }

                // ── App grid ───────────────────────────────────────────────
                if filtered.is_empty() {
                    div {
                        style: "text-align: center; color: var(--fsn-color-text-muted, #94a3b8); padding: 32px;",
                        "No apps found for "{props.query.clone()}""
                    }
                } else {
                    div {
                        class: "fsd-launcher__grid",
                        style: "display: grid; \
                                grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); \
                                gap: 12px;",
                        for app in filtered {
                            AppTile {
                                key: "{app.id}",
                                app: app.clone(),
                                on_click: {
                                    let id = app.id.clone();
                                    move |_| props.on_launch.call(id.clone())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A single icon + label in the launcher grid.
#[component]
fn AppTile(app: AppEntry, on_click: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "fsd-launcher__tile",
            style: "display: flex; flex-direction: column; align-items: center; gap: 8px; \
                    padding: 16px 8px; border-radius: 8px; border: 1px solid transparent; \
                    background: var(--fsn-color-bg-panel, rgba(30,41,59,0.8)); \
                    cursor: pointer; color: var(--fsn-color-text-primary, #e2e8f0); \
                    transition: background 0.15s, border-color 0.15s;
                    &:hover { background: var(--fsn-color-bg-active); border-color: var(--fsn-color-primary); }",
            onclick: on_click,

            span {
                style: "font-size: 32px; line-height: 1;",
                "{app.icon}"
            }
            span {
                style: "font-size: 12px; text-align: center; \
                        overflow: hidden; text-overflow: ellipsis; \
                        white-space: nowrap; max-width: 100%;",
                "{app.label_key}"
            }
            if app.is_running() {
                div {
                    style: "width: 6px; height: 6px; border-radius: 50%; \
                            background: var(--fsn-color-primary, #06b6d4);",
                }
            }
        }
    }
}
