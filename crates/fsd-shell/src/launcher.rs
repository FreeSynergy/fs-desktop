/// AppLauncher — full-screen overlay with grouped app tiles + search bar.
///
/// Layout:
///  - Search bar at the top
///  - Apps grouped by `AppEntry::group` (accordion: click group header to collapse)
///  - Each group row: icon grid → repeat(auto-fill, 120px)
///  - Pagination for large groups (future: currently shows all)
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

/// A named group of apps displayed in the launcher.
#[derive(Clone, PartialEq, Debug)]
pub struct AppGroup {
    pub id: String,
    pub label: String,
    pub apps: Vec<AppEntry>,
}

impl AppGroup {
    /// Build groups from a flat app list using `AppEntry::group`.
    /// Apps without a group fall into "Other".
    /// Insertion order is preserved by tracking order in a separate Vec.
    pub fn from_entries(entries: &[AppEntry]) -> Vec<AppGroup> {
        let mut order: Vec<String> = Vec::new();
        let mut map: std::collections::HashMap<String, Vec<AppEntry>> =
            std::collections::HashMap::new();
        for app in entries {
            let key = app.group.clone().unwrap_or_else(|| "Other".into());
            if !map.contains_key(&key) {
                order.push(key.clone());
            }
            map.entry(key).or_default().push(app.clone());
        }
        order
            .into_iter()
            .map(|label| {
                let apps = map.remove(&label).unwrap_or_default();
                AppGroup {
                    id: label.to_lowercase().replace(' ', "-"),
                    label,
                    apps,
                }
            })
            .collect()
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AppLauncherProps {
    pub apps: Vec<AppEntry>,
    pub query: String,
    pub on_query_change: EventHandler<String>,
    pub on_launch: EventHandler<String>,
    pub on_close: EventHandler<()>,
}

/// Full-screen overlay with grouped app tiles + search bar.
#[component]
pub fn AppLauncher(props: AppLauncherProps) -> Element {
    let query = props.query.to_lowercase();

    // Filter apps and rebuild groups for display
    let filtered: Vec<AppEntry> = props
        .apps
        .iter()
        .filter(|a| {
            query.is_empty()
                || a.id.to_lowercase().contains(&query)
                || a.label_key.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    let groups = AppGroup::from_entries(&filtered);

    rsx! {
        // Backdrop
        div {
            class: "fsd-launcher__backdrop",
            style: "position: fixed; inset: 0; z-index: 9000; \
                    background: rgba(0,0,0,0.82); backdrop-filter: blur(12px); \
                    display: flex; flex-direction: column; align-items: center; \
                    justify-content: flex-start; padding-top: 60px;",
            onclick: move |_| props.on_close.call(()),

            // Inner panel
            div {
                class: "fsd-launcher__panel",
                style: "width: min(800px, 95vw); display: flex; flex-direction: column; gap: 20px; \
                        max-height: 80vh; overflow: hidden;",
                onclick: move |evt: MouseEvent| evt.stop_propagation(),

                // ── Header ─────────────────────────────────────────────────
                div {
                    style: "text-align: center; color: var(--fsn-primary); font-size: 18px; font-weight: 700; \
                            letter-spacing: -0.02em;",
                    "FreeSynergy"
                    span {
                        style: "color: var(--fsn-text-muted); font-size: 12px; font-weight: 400; margin-left: 8px;",
                        "by KalEl"
                    }
                }

                // ── Search bar ─────────────────────────────────────────────
                input {
                    class: "fsd-launcher__search",
                    style: "width: 100%; padding: 12px 16px; border-radius: var(--fsn-radius-md); \
                            background: var(--fsn-bg-input); \
                            border: 1px solid var(--fsn-border); \
                            color: var(--fsn-text-primary); font-size: 15px; \
                            outline: none; box-sizing: border-box; \
                            transition: border-color 150ms ease;",
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

                // ── App groups ────────────────────────────────────────────
                div {
                    style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; \
                            padding-bottom: 8px;",
                    if filtered.is_empty() {
                        div {
                            style: "text-align: center; color: var(--fsn-text-muted); padding: 48px 0; font-size: 14px;",
                            "No apps found for \"{props.query}\""
                        }
                    } else {
                        for group in &groups {
                            AppGroupSection {
                                key: "{group.id}",
                                group: group.clone(),
                                on_launch: props.on_launch.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── App Group Section (accordion) ────────────────────────────────────────────

#[component]
fn AppGroupSection(group: AppGroup, on_launch: EventHandler<String>) -> Element {
    let mut expanded = use_signal(|| true);

    rsx! {
        div {
            style: "background: var(--fsn-bg-card); border-radius: var(--fsn-radius-md); \
                    border: 1px solid var(--fsn-border); overflow: hidden;",

            // Group header (accordion toggle)
            button {
                style: "display: flex; align-items: center; gap: 8px; width: 100%; padding: 10px 16px; \
                        background: none; border: none; border-bottom: 1px solid var(--fsn-border); \
                        cursor: pointer; color: var(--fsn-text-secondary); font-size: 12px; font-weight: 600; \
                        text-transform: uppercase; letter-spacing: 0.07em; text-align: left;",
                onclick: move |_| {
                    let v = *expanded.read();
                    *expanded.write() = !v;
                },
                span { style: "font-size: 10px; color: var(--fsn-text-muted);",
                    if *expanded.read() { "▼" } else { "▶" }
                }
                "{group.label}"
                span {
                    style: "margin-left: auto; font-size: 11px; color: var(--fsn-text-muted); font-weight: 400;",
                    "{group.apps.len()}"
                }
            }

            // App grid (collapsible)
            if *expanded.read() {
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); \
                            gap: 8px; padding: 12px;",
                    for app in &group.apps {
                        AppTile {
                            key: "{app.id}",
                            app: app.clone(),
                            on_click: {
                                let id = app.id.clone();
                                move |_| on_launch.call(id.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

// ── App Tile ─────────────────────────────────────────────────────────────────

/// A single icon + label in the launcher grid.
#[component]
fn AppTile(app: AppEntry, on_click: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "fsd-launcher__tile",
            style: "display: flex; flex-direction: column; align-items: center; gap: 6px; \
                    padding: 12px 8px; border-radius: var(--fsn-radius-md); \
                    border: 1px solid transparent; \
                    background: var(--fsn-bg-surface); \
                    cursor: pointer; color: var(--fsn-text-primary); \
                    transition: background 150ms, border-color 150ms;",
            onclick: on_click,

            // Icon: prefer URL, fall back to emoji
            if let Some(url) = &app.icon_url {
                img {
                    src: "{url}",
                    alt: "{app.icon}",
                    width: "36",
                    height: "36",
                    style: "border-radius: 8px; object-fit: contain;",
                    // On error: show emoji as fallback (handled in JS)
                    onerror: "this.style.display='none'; this.nextSibling.style.display='block';",
                }
                span {
                    style: "font-size: 28px; display: none; line-height: 1;",
                    "{app.icon}"
                }
            } else {
                span { style: "font-size: 28px; line-height: 1;", "{app.icon}" }
            }

            span {
                style: "font-size: 11px; text-align: center; width: 100%; \
                        overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                "{app.label_key}"
            }

            // Running indicator dot
            if app.is_running() {
                div {
                    style: "width: 5px; height: 5px; border-radius: 50%; \
                            background: var(--fsn-primary);",
                }
            }
        }
    }
}
