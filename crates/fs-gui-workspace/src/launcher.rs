/// `AppLauncher` — full-screen overlay with grouped app tiles + search bar.
///
/// Layout:
///  - Search bar at the top
///  - Apps grouped by `AppEntry::group` (accordion: click group header to collapse)
///  - Each group row: icon grid → repeat(auto-fill, 120px)
///  - Pagination for large groups (future: currently shows all)
use dioxus::prelude::*;
use fs_i18n;

use crate::icons::{ICON_CHEVRON_DOWN, ICON_CHEVRON_RIGHT};

use crate::taskbar::AppEntry;

/// How many app groups to show per page in the launcher.
const GROUPS_PER_PAGE: usize = 3;

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
    #[must_use]
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
    let mut page: Signal<usize> = use_signal(|| 0);

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
    let total_pages = groups.len().div_ceil(GROUPS_PER_PAGE).max(1);

    // Clamp page to valid range when query changes and group count shrinks
    let cur_page = (*page.read()).min(total_pages - 1);

    let page_groups: Vec<AppGroup> = groups
        .into_iter()
        .skip(cur_page * GROUPS_PER_PAGE)
        .take(GROUPS_PER_PAGE)
        .collect();

    rsx! {
        // Backdrop
        div {
            class: "fs-launcher__backdrop",
            style: "position: fixed; inset: 0; z-index: 9000; \
                    background: rgba(0,0,0,0.82); backdrop-filter: blur(12px); \
                    display: flex; flex-direction: column; align-items: center; \
                    justify-content: flex-start; padding-top: 60px;",
            onclick: move |_| props.on_close.call(()),

            // Inner panel
            div {
                class: "fs-launcher__panel",
                style: "width: min(800px, 95vw); display: flex; flex-direction: column; gap: 20px; \
                        max-height: 80vh; overflow: hidden;",
                onclick: move |evt: MouseEvent| evt.stop_propagation(),

                // ── Header ─────────────────────────────────────────────────
                div {
                    style: "text-align: center; color: var(--fs-primary); font-size: 18px; font-weight: 700; \
                            letter-spacing: -0.02em;",
                    "FreeSynergy"
                    span {
                        style: "color: var(--fs-text-muted); font-size: 12px; font-weight: 400; margin-left: 8px;",
                        "by KalEl"
                    }
                }

                // ── Search bar ─────────────────────────────────────────────
                input {
                    class: "fs-launcher__search",
                    style: "width: 100%; padding: 12px 16px; border-radius: var(--fs-radius-md); \
                            background: var(--fs-bg-input); \
                            border: 1px solid var(--fs-border); \
                            color: var(--fs-text-primary); font-size: 15px; \
                            outline: none; box-sizing: border-box; \
                            transition: border-color 150ms ease;",
                    r#type: "text",
                    placeholder: fs_i18n::t("shell.launcher.search_placeholder").to_string(),
                    value: props.query.clone(),
                    autofocus: true,
                    oninput: move |evt| {
                        *page.write() = 0; // reset to first page on new query
                        props.on_query_change.call(evt.value());
                    },
                    onkeydown: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Escape {
                            props.on_close.call(());
                        }
                    },
                }

                // ── App groups (current page) ──────────────────────────────
                div {
                    class: "fs-scrollable",
                    style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; \
                            padding-bottom: 8px;",
                    if filtered.is_empty() {
                        div {
                            style: "text-align: center; color: var(--fs-text-muted); padding: 48px 0; font-size: 14px;",
                            {fs_i18n::t_with("shell.launcher.no_apps", &[("query", props.query.as_str())])}
                        }
                    } else {
                        for group in &page_groups {
                            AppGroupSection {
                                key: "{group.id}",
                                group: group.clone(),
                                on_launch: props.on_launch,
                            }
                        }
                    }
                }

                // ── Pagination bar ─────────────────────────────────────────
                if total_pages > 1 {
                    LauncherPagination {
                        current: cur_page,
                        total: total_pages,
                        on_prev: move |()| {
                            let p = *page.read();
                            if p > 0 { *page.write() = p - 1; }
                        },
                        on_next: move |()| {
                            let p = *page.read();
                            if p + 1 < total_pages { *page.write() = p + 1; }
                        },
                        on_goto: move |idx| *page.write() = idx,
                    }
                }
            }
        }
    }
}

// ── Pagination bar ────────────────────────────────────────────────────────────

#[component]
fn LauncherPagination(
    current: usize,
    total: usize,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
    on_goto: EventHandler<usize>,
) -> Element {
    let at_start = current == 0;
    let at_end = current + 1 >= total;
    let op_prev = if at_start { "0.3" } else { "1.0" };
    let op_next = if at_end { "0.3" } else { "1.0" };
    let n_str = (current + 1).to_string();
    let total_str = total.to_string();
    let page_label = fs_i18n::t_with(
        "shell.launcher.page",
        &[("n", n_str.as_str()), ("total", total_str.as_str())],
    );
    // Pre-compute dot colors — avoids if/else inside rsx! format strings
    let dots: Vec<(usize, &'static str)> = (0..total)
        .map(|i| {
            (
                i,
                if i == current {
                    "var(--fs-primary)"
                } else {
                    "var(--fs-text-muted)"
                },
            )
        })
        .collect();
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: center; \
                    gap: 8px; padding: 8px 0 4px; flex-shrink: 0;",

            // Previous button
            button {
                style: "background: none; border: 1px solid var(--fs-border); \
                        border-radius: var(--fs-radius-sm); color: var(--fs-text-secondary); \
                        font-size: 13px; cursor: pointer; padding: 2px 10px; \
                        opacity: {op_prev};",
                disabled: at_start,
                onclick: move |_| on_prev.call(()),
                "◄"
            }

            // Page label + dot indicators
            span {
                style: "font-size: 12px; color: var(--fs-text-muted); min-width: 60px; text-align: center;",
                "{page_label}"
            }
            for (i, bg) in dots {
                button {
                    key: "{i}",
                    style: "width: 8px; height: 8px; border-radius: 50%; border: none; \
                            cursor: pointer; padding: 0; display: inline-block; \
                            background: {bg};",
                    onclick: move |_| on_goto.call(i),
                }
            }

            // Next button
            button {
                style: "background: none; border: 1px solid var(--fs-border); \
                        border-radius: var(--fs-radius-sm); color: var(--fs-text-secondary); \
                        font-size: 13px; cursor: pointer; padding: 2px 10px; \
                        opacity: {op_next};",
                disabled: at_end,
                onclick: move |_| on_next.call(()),
                "►"
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
            style: "background: var(--fs-bg-card); border-radius: var(--fs-radius-md); \
                    border: 1px solid var(--fs-border); overflow: hidden;",

            // Group header (accordion toggle)
            button {
                style: "display: flex; align-items: center; gap: 8px; width: 100%; padding: 10px 16px; \
                        background: none; border: none; border-bottom: 1px solid var(--fs-border); \
                        cursor: pointer; color: var(--fs-text-secondary); font-size: 12px; font-weight: 600; \
                        text-transform: uppercase; letter-spacing: 0.07em; text-align: left;",
                onclick: move |_| {
                    let v = *expanded.read();
                    *expanded.write() = !v;
                },
                span { style: "color: var(--fs-text-muted); display: flex; align-items: center;",
                    if *expanded.read() {
                        span { dangerous_inner_html: ICON_CHEVRON_DOWN }
                    } else {
                        span { dangerous_inner_html: ICON_CHEVRON_RIGHT }
                    }
                }
                "{group.label}"
                span {
                    style: "margin-left: auto; font-size: 11px; color: var(--fs-text-muted); font-weight: 400;",
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
            class: "fs-launcher__tile",
            style: "display: flex; flex-direction: column; align-items: center; gap: 6px; \
                    padding: 12px 8px; border-radius: var(--fs-radius-md); \
                    border: 1px solid transparent; \
                    background: var(--fs-bg-surface); \
                    cursor: pointer; color: var(--fs-text-primary); \
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
                            background: var(--fs-primary);",
                }
            }
        }
    }
}
