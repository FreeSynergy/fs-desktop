use crate::icons::ICON_HELP;
/// Help View — context-sensitive help and keyboard shortcuts reference.
/// Also exports `HelpSidebarPanel`: the collapsible right-side help panel for the Desktop.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, SidebarSide, FS_SIDEBAR_CSS};
use fs_settings::{register_actions, resolve_shortcut, ShortcutsConfig};
use serde_json;

#[derive(Clone, PartialEq, Debug)]
enum HelpSection {
    Topics,
    Shortcuts,
}

impl HelpSection {
    fn id(&self) -> &'static str {
        match self {
            Self::Topics => "topics",
            Self::Shortcuts => "shortcuts",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Topics => "Topics",
            Self::Shortcuts => "Shortcuts",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Topics => {
                r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>"#
            }
            Self::Shortcuts => {
                r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M8 14h8"/></svg>"#
            }
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "topics" => Some(Self::Topics),
            "shortcuts" => Some(Self::Shortcuts),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq)]
struct HelpTopic {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
}

const TOPICS: &[HelpTopic] = &[
    HelpTopic {
        id: "getting-started",
        title: "Getting Started",
        summary: "Learn how to set up your first FreeSynergy.Node deployment.",
    },
    HelpTopic {
        id: "container-app",
        title: "Container",
        summary: "Manage services, bots, and containers from the Container App view.",
    },
    HelpTopic {
        id: "store",
        title: "Module Store",
        summary: "Browse, install, and update service modules from the store.",
    },
    HelpTopic {
        id: "studio",
        title: "Studio",
        summary: "Create custom modules, plugins, and language packs.",
    },
    HelpTopic {
        id: "settings",
        title: "Settings",
        summary: "Configure appearance, language, service roles, and AI connections.",
    },
    HelpTopic {
        id: "ai-assistant",
        title: "AI Assistant",
        summary: "Use your local Ollama instance as an integrated AI helper.",
    },
    HelpTopic {
        id: "troubleshooting",
        title: "Troubleshooting",
        summary: "Common issues and how to resolve them.",
    },
];

const ALL_SECTIONS: &[HelpSection] = &[HelpSection::Topics, HelpSection::Shortcuts];

/// Root component for the Help view (standalone window).
/// Uses Sidebar on the left side for navigation between Topics and Shortcuts.
#[component]
pub fn HelpApp() -> Element {
    let mut active = use_signal(|| HelpSection::Topics);

    let sidebar_items: Vec<SidebarItem> = ALL_SECTIONS
        .iter()
        .map(|s| SidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        div {
            class: "fs-help-view",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fs-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fs-border); \
                        flex-shrink: 0; background: var(--fs-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fs-text-primary);",
                    "Help & Documentation"
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                Sidebar {
                    items: sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = HelpSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                div { style: "flex: 1; overflow: hidden;",
                    match *active.read() {
                        HelpSection::Topics    => rsx! { TopicsView {} },
                        HelpSection::Shortcuts => rsx! { ShortcutsReference {} },
                    }
                }
            }
        }
    }
}

#[component]
fn TopicsView() -> Element {
    let mut query = use_signal(String::new);

    let q = query.read().to_lowercase();
    let filtered: Vec<&HelpTopic> = TOPICS
        .iter()
        .filter(|t| {
            q.is_empty()
                || t.title.to_lowercase().contains(&q)
                || t.summary.to_lowercase().contains(&q)
        })
        .collect();

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100%;",
            div { style: "padding: 12px 24px;",
                input {
                    r#type: "text",
                    placeholder: "Search help topics…",
                    style: "width: 100%; max-width: 480px; padding: 8px 12px; border-radius: 6px; \
                            background: var(--fs-color-bg-input, #0f172a); \
                            border: 1px solid var(--fs-color-border-default, #334155); \
                            color: var(--fs-color-text-primary, #e2e8f0); font-size: 14px; \
                            outline: none; box-sizing: border-box;",
                    oninput: move |evt| query.set(evt.value()),
                }
            }
            div {
                class: "fs-scrollable", style: "flex: 1; overflow-y: auto; padding: 0 24px 16px;",
                if filtered.is_empty() {
                    p { style: "color: var(--fs-color-text-muted); font-size: 14px;",
                        "No topics found."
                    }
                } else {
                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                        for topic in filtered {
                            HelpTopicCard { topic: topic.clone() }
                        }
                    }
                }
            }
        }
    }
}

/// Read-only shortcuts reference — auto-generated from the action registry.
#[component]
fn ShortcutsReference() -> Element {
    let config = ShortcutsConfig::load();
    let actions = register_actions();

    let mut categories: Vec<&str> = actions.iter().map(|a| a.category).collect();
    categories.sort_unstable();
    categories.dedup();

    rsx! {
        div {
            class: "fs-scrollable", style: "overflow-y: auto; padding: 16px 24px; height: 100%;",
            p {
                style: "font-size: 12px; color: var(--fs-text-muted); margin-bottom: 16px;",
                "Shortcuts can be customized in Settings \u{2192} Shortcuts."
            }
            for cat in &categories {
                {
                    let cat_actions: Vec<&fs_settings::ActionDef> = actions.iter().filter(|a| a.category == *cat).collect();
                    rsx! {
                        div { style: "margin-bottom: 20px;",
                            div {
                                style: "font-size: 11px; font-weight: 600; text-transform: uppercase; \
                                        letter-spacing: 0.08em; color: var(--fs-text-muted); \
                                        margin-bottom: 6px; padding-bottom: 4px; \
                                        border-bottom: 1px solid var(--fs-border);",
                                "{cat}"
                            }
                            for action in cat_actions {
                                {
                                    let shortcut = resolve_shortcut(action, &config)
                                        .unwrap_or("—");
                                    rsx! {
                                        div {
                                            key: "{action.id}",
                                            style: "display: flex; align-items: center; justify-content: space-between; \
                                                    padding: 5px 0; font-size: 13px;",
                                            span { style: "color: var(--fs-text-primary);", "{action.label}" }
                                            span {
                                                style: "font-family: var(--fs-font-mono); font-size: 12px; \
                                                        color: var(--fs-text-secondary); \
                                                        background: var(--fs-bg-elevated); \
                                                        padding: 2px 8px; border-radius: var(--fs-radius-sm); \
                                                        border: 1px solid var(--fs-border);",
                                                "{shortcut}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HelpTopicCard(topic: HelpTopic) -> Element {
    rsx! {
        div {
            class: "fs-help-topic",
            style: "padding: 14px 16px; border-radius: 8px; \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    cursor: pointer;",
            h3 {
                style: "margin: 0 0 4px; font-size: 15px; \
                        color: var(--fs-color-primary, #06b6d4);",
                "{topic.title}"
            }
            p {
                style: "margin: 0; font-size: 13px; \
                        color: var(--fs-color-text-muted, #94a3b8);",
                "{topic.summary}"
            }
        }
    }
}

// ── HelpSidebarPanel ──────────────────────────────────────────────────────────
// Collapsible right-side panel embedded in the Desktop shell.
// Uses the universal Sidebar { side: SidebarSide::Right } — the SAME component
// as every other sidebar in the application; only the panel CONTENT differs.
//
// Layout when open (280 px):
//   [tab-strip 44px] | [drag-edge 6px] [inner-content panel_width]
//
// The tab-strip (mirrored parallelogram) is always visible at the right edge.
// CSS :hover expands the panel leftward. force_open keeps it expanded while
// the user is drag-resizing the panel width or AI section height.

#[derive(Clone, PartialEq, Debug)]
enum SidebarTab {
    Topics,
    Shortcuts,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChatMsg {
    pub role: &'static str,
    pub content: String,
}

/// Collapsible right-side help panel for the Desktop shell.
///
/// Uses `Sidebar { side: SidebarSide::Right }` — the universal sidebar
/// component. The help icon appears in the tab-strip (mirrored parallelogram);
/// the panel body contains Topics/Shortcuts tabs, content, and the AI chat
/// section when the AI service is online.
///
/// `active_help_topic`: topic-id of the currently focused window (e.g. "store").
/// When set, the Topics tab shows that topic's entry at the top, highlighted.
///
/// Panel width is drag-resizable (left edge handle). AI section height is
/// drag-resizable (divider above it).
#[component]
pub fn HelpSidebarPanel(
    #[props(default)] on_ai_offline: Option<EventHandler<()>>,
    #[props(default)] on_ai_online: Option<EventHandler<()>>,
    #[props(default)] active_help_topic: Option<String>,
) -> Element {
    let mut tab = use_signal(|| SidebarTab::Topics);

    // ── Panel geometry ────────────────────────────────────────────────────
    let mut panel_width: Signal<f64> = use_signal(|| 280.0);
    let mut ai_height: Signal<f64> = use_signal(|| 260.0);

    // Width-resize drag state
    let mut resizing_w: Signal<bool> = use_signal(|| false);
    let mut resize_w_sx: Signal<f64> = use_signal(|| 0.0);
    let mut resize_w_sw: Signal<f64> = use_signal(|| 280.0);

    // AI-section height resize drag state
    let mut resizing_ai: Signal<bool> = use_signal(|| false);
    let mut resize_ai_sy: Signal<f64> = use_signal(|| 0.0);
    let mut resize_ai_sh: Signal<f64> = use_signal(|| 260.0);

    // ── AI health ─────────────────────────────────────────────────────────
    let mut ai_online: Signal<bool> = use_signal(|| false);
    let mut ai_was_online: Signal<bool> = use_signal(|| false);

    {
        let on_offline = on_ai_offline;
        let on_online = on_ai_online;
        use_future(move || async move {
            loop {
                let url = fs_ai::ai_api_url();
                let is_now = if let Some(ref api) = url {
                    reqwest::Client::new()
                        .get(format!("{api}/models"))
                        .timeout(std::time::Duration::from_secs(3))
                        .send()
                        .await
                        .is_ok()
                } else {
                    false
                };

                let was = *ai_was_online.read();
                ai_online.set(is_now);
                if is_now != was {
                    ai_was_online.set(is_now);
                    if is_now {
                        if let Some(ref cb) = on_online {
                            cb.call(());
                        }
                    } else if was {
                        if let Some(ref cb) = on_offline {
                            cb.call(());
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });
    }

    // ── Chat state ────────────────────────────────────────────────────────
    let mut messages: Signal<Vec<ChatMsg>> = use_signal(Vec::new);
    let mut input = use_signal(String::new);
    let mut thinking = use_signal(|| false);
    let ai_url = fs_ai::ai_api_url();

    // ── Derived values ────────────────────────────────────────────────────
    let is_ai_online = *ai_online.read();
    let is_resizing = *resizing_w.read() || *resizing_ai.read();
    let pw = *panel_width.read();
    let aih = *ai_height.read();

    // The help icon as the sole tab-strip entry.
    let help_item = SidebarItem::new("help", ICON_HELP, "Help");

    rsx! {
        // ── Width-resize fullscreen overlay (active while dragging) ───────
        if *resizing_w.read() {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; \
                        pointer-events: all; cursor: ew-resize;",
                onmousemove: move |evt: MouseEvent| {
                    let c = evt.data().client_coordinates();
                    let dx = *resize_w_sx.read() - c.x;
                    let new_w = (*resize_w_sw.read() + dx).clamp(180.0, 600.0);
                    panel_width.set(new_w);
                },
                onmouseup: move |_| resizing_w.set(false),
            }
        }

        // ── AI-section height resize overlay ──────────────────────────────
        if *resizing_ai.read() {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; \
                        pointer-events: all; cursor: ns-resize;",
                onmousemove: move |evt: MouseEvent| {
                    let c = evt.data().client_coordinates();
                    let dy = c.y - *resize_ai_sy.read();
                    let new_h = (*resize_ai_sh.read() + dy).clamp(100.0, 450.0);
                    ai_height.set(new_h);
                },
                onmouseup: move |_| resizing_ai.set(false),
            }
        }

        // ── Universal Sidebar (side = Right) ────────────────────────────
        // items: help icon in tab-strip. custom_panel: the panel body.
        Sidebar {
            items:        vec![help_item],
            active_id:    "help".to_string(),
            on_select:    move |_| {},
            side:         SidebarSide::Right,
            panel_width:  pw,
            force_open:   is_resizing,
            custom_panel: rsx! {
            // ── Panel body ────────────────────────────────────────────────
            div {
                style: "display: flex; flex-direction: row; flex: 1; overflow: hidden; \
                        align-self: stretch;",

                // Left-edge drag handle — drag to resize panel width
                div {
                    style: "width: 6px; flex-shrink: 0; cursor: ew-resize; \
                            background: transparent; transition: background 120ms;",
                    onmousedown: move |evt: MouseEvent| {
                        evt.stop_propagation();
                        let c = evt.data().client_coordinates();
                        resize_w_sx.set(c.x);
                        resize_w_sw.set(*panel_width.read());
                        resizing_w.set(true);
                    },
                }

                // Inner content
                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; \
                            min-width: 0;",

                    // Header
                    div {
                        style: "display: flex; align-items: center; \
                                justify-content: space-between; padding: 10px 14px; \
                                border-bottom: 1px solid var(--fs-border); \
                                flex-shrink: 0; background: var(--fs-bg-elevated);",
                        span {
                            style: "font-size: 14px; font-weight: 600; \
                                    color: var(--fs-text-primary);",
                            "Help"
                        }
                        if is_ai_online {
                            span {
                                style: "font-size: 10px; background: var(--fs-success-bg); \
                                        color: var(--fs-success); border-radius: 4px; \
                                        padding: 2px 6px; border: 1px solid var(--fs-success);",
                                "AI"
                            }
                        }
                    }

                    // Tab selector (Topics / Shortcuts)
                    div {
                        style: "display: flex; border-bottom: 1px solid var(--fs-border); \
                                flex-shrink: 0;",
                        button {
                            style: {
                                let active = *tab.read() == SidebarTab::Topics;
                                format!("flex: 1; padding: 7px 0; text-align: center; \
                                         font-size: 12px; cursor: pointer; background: none; \
                                         border: none; border-bottom: 2px solid {}; \
                                         color: {}; font-family: inherit; \
                                         transition: color 120ms, border-color 120ms;",
                                    if active { "var(--fs-primary)" } else { "transparent" },
                                    if active { "var(--fs-primary)" } else { "var(--fs-text-muted)" })
                            },
                            onclick: move |_| tab.set(SidebarTab::Topics),
                            "Topics"
                        }
                        button {
                            style: {
                                let active = *tab.read() == SidebarTab::Shortcuts;
                                format!("flex: 1; padding: 7px 0; text-align: center; \
                                         font-size: 12px; cursor: pointer; background: none; \
                                         border: none; border-bottom: 2px solid {}; \
                                         color: {}; font-family: inherit; \
                                         transition: color 120ms, border-color 120ms;",
                                    if active { "var(--fs-primary)" } else { "transparent" },
                                    if active { "var(--fs-primary)" } else { "var(--fs-text-muted)" })
                            },
                            onclick: move |_| tab.set(SidebarTab::Shortcuts),
                            "Shortcuts"
                        }
                    }

                    // Main content area
                    div {
                        style: "flex: 1; overflow-y: auto; min-height: 0;",
                        class: "fs-scrollable",
                        match *tab.read() {
                            SidebarTab::Topics    => rsx! {
                                SidebarTopicsView { active_topic: active_help_topic.clone() }
                            },
                            SidebarTab::Shortcuts => rsx! { SidebarShortcutsView {} },
                        }
                    }

                    // AI chat section (only when AI is online)
                    if is_ai_online {
                        // Divider — drag to resize AI section height
                        div {
                            style: "height: 6px; flex-shrink: 0; cursor: ns-resize; \
                                    background: var(--fs-border); \
                                    transition: background 120ms;",
                            onmousedown: move |evt: MouseEvent| {
                                evt.stop_propagation();
                                let c = evt.data().client_coordinates();
                                resize_ai_sy.set(c.y);
                                resize_ai_sh.set(*ai_height.read());
                                resizing_ai.set(true);
                            },
                        }

                        div {
                            style: "height: {aih}px; flex-shrink: 0; display: flex; \
                                    flex-direction: column; overflow: hidden;",

                            div {
                                style: "padding: 6px 12px; font-size: 11px; font-weight: 600; \
                                        text-transform: uppercase; letter-spacing: 0.08em; \
                                        color: var(--fs-accent); \
                                        border-bottom: 1px solid var(--fs-border); \
                                        flex-shrink: 0;",
                                "AI Assistant"
                            }

                            div {
                                style: "flex: 1; overflow-y: auto; padding: 8px 12px; \
                                        display: flex; flex-direction: column; \
                                        gap: 6px; min-height: 0;",
                                class: "fs-scrollable",
                                if messages.read().is_empty() {
                                    p {
                                        style: "color: var(--fs-text-muted); font-size: 12px; \
                                                text-align: center; margin: 12px 0;",
                                        "Ask me anything about FreeSynergy\u{2026}"
                                    }
                                }
                                for msg in messages.read().iter() {
                                    {
                                        let is_user = msg.role == "user";
                                        let (bg, align, color) = if is_user {
                                            ("var(--fs-primary)", "flex-end", "#fff")
                                        } else {
                                            ("var(--fs-bg-elevated)", "flex-start",
                                             "var(--fs-text-primary)")
                                        };
                                        rsx! {
                                            div {
                                                style: "display: flex; justify-content: {align};",
                                                div {
                                                    style: "max-width: 90%; padding: 6px 10px; \
                                                            border-radius: 8px; background: {bg}; \
                                                            color: {color}; font-size: 12px; \
                                                            line-height: 1.5; \
                                                            white-space: pre-wrap;",
                                                    "{msg.content}"
                                                }
                                            }
                                        }
                                    }
                                }
                                if *thinking.read() {
                                    div {
                                        style: "display: flex; align-items: center; gap: 4px; \
                                                color: var(--fs-text-muted); font-size: 11px;",
                                        "AI is thinking\u{2026}"
                                    }
                                }
                            }

                            div {
                                style: "display: flex; gap: 6px; padding: 8px 10px; \
                                        border-top: 1px solid var(--fs-border); \
                                        flex-shrink: 0;",
                                input {
                                    r#type: "text",
                                    style: "flex: 1; padding: 7px 10px; border-radius: 6px; \
                                            background: var(--fs-bg-input); \
                                            border: 1px solid var(--fs-border); \
                                            color: var(--fs-text-primary); font-size: 13px; \
                                            font-family: inherit; outline: none; \
                                            resize: none; height: 34px; line-height: 1.4;",
                                    placeholder: "Ask a question\u{2026}",
                                    value: "{input.read()}",
                                    oninput: move |e| input.set(e.value()),
                                    onkeydown: {
                                        let api = ai_url.clone();
                                        move |e: KeyboardEvent| {
                                            if e.key() == Key::Enter
                                                && !input.read().is_empty()
                                                && !*thinking.read()
                                            {
                                                let text = input.read().clone();
                                                input.set(String::new());
                                                thinking.set(true);
                                                messages.write().push(ChatMsg {
                                                    role: "user", content: text.clone()
                                                });
                                                let url = api.clone().unwrap_or_default();
                                                let msgs = messages.read().clone();
                                                spawn(async move {
                                                    let reply = chat_request(&url, &msgs).await;
                                                    messages.write().push(ChatMsg {
                                                        role: "assistant", content: reply
                                                    });
                                                    thinking.set(false);
                                                });
                                            }
                                        }
                                    },
                                }
                                button {
                                    style: "padding: 0 12px; border-radius: 6px; \
                                            background: var(--fs-primary); border: none; \
                                            color: #fff; font-size: 13px; \
                                            cursor: pointer; flex-shrink: 0;",
                                    disabled: *thinking.read() || input.read().is_empty(),
                                    onclick: {
                                        let api = ai_url.clone();
                                        move |_| {
                                            if input.read().is_empty() || *thinking.read() {
                                                return;
                                            }
                                            let text = input.read().clone();
                                            input.set(String::new());
                                            thinking.set(true);
                                            messages.write().push(ChatMsg {
                                                role: "user", content: text.clone()
                                            });
                                            let url = api.clone().unwrap_or_default();
                                            let msgs = messages.read().clone();
                                            spawn(async move {
                                                let reply = chat_request(&url, &msgs).await;
                                                messages.write().push(ChatMsg {
                                                    role: "assistant", content: reply
                                                });
                                                thinking.set(false);
                                            });
                                        }
                                    },
                                    "\u{21b5}"
                                }
                            }
                        }
                    }
                }
            }
            },  // close custom_panel rsx!
        }
    }
}

// ── Sidebar-specific compact views ───────────────────────────────────────────

/// Compact topic list for the sidebar (no search box to save space).
/// If `active_topic` matches a known topic id, that entry is shown first,
/// highlighted with a "Current App" label, followed by the remaining topics.
#[component]
fn SidebarTopicsView(#[props(default)] active_topic: Option<String>) -> Element {
    let pinned = active_topic
        .as_deref()
        .and_then(|id| TOPICS.iter().find(|t| t.id == id));

    rsx! {
        div { style: "padding: 8px;",
            // ── Pinned: current-app topic ──────────────────────────────────
            if let Some(topic) = pinned {
                div {
                    style: "margin-bottom: 8px;",
                    div {
                        style: "font-size: 10px; font-weight: 600; text-transform: uppercase; \
                                letter-spacing: 0.08em; color: var(--fs-accent); \
                                padding: 0 2px 4px;",
                        "Current App"
                    }
                    div {
                        style: "padding: 8px 10px; border-radius: 6px; \
                                border: 1px solid var(--fs-accent); \
                                background: rgba(34,211,238,0.08);",
                        p {
                            style: "margin: 0 0 2px; font-size: 13px; font-weight: 600; \
                                    color: var(--fs-accent);",
                            "{topic.title}"
                        }
                        p {
                            style: "margin: 0; font-size: 11px; color: var(--fs-text-muted); \
                                    line-height: 1.4;",
                            "{topic.summary}"
                        }
                    }
                }
            }

            // ── All topics ─────────────────────────────────────────────────
            for topic in TOPICS {
                // Skip the pinned topic so it doesn't appear twice
                if active_topic.as_deref() != Some(topic.id) {
                    div {
                        style: "padding: 8px 10px; border-radius: 6px; margin-bottom: 4px; \
                                cursor: pointer; \
                                border: 1px solid var(--fs-border);",
                        p {
                            style: "margin: 0 0 2px; font-size: 13px; font-weight: 500; \
                                    color: var(--fs-primary);",
                            "{topic.title}"
                        }
                        p {
                            style: "margin: 0; font-size: 11px; color: var(--fs-text-muted); \
                                    line-height: 1.4;",
                            "{topic.summary}"
                        }
                    }
                }
            }
        }
    }
}

/// Compact shortcuts list for the sidebar.
#[component]
fn SidebarShortcutsView() -> Element {
    let config = ShortcutsConfig::load();
    let actions = register_actions();

    rsx! {
        div { style: "padding: 8px 12px;",
            for action in &actions {
                {
                    let shortcut = resolve_shortcut(action, &config).unwrap_or("—");
                    rsx! {
                        div {
                            key: "{action.id}",
                            style: "display: flex; align-items: center; \
                                    justify-content: space-between; \
                                    padding: 4px 0; font-size: 12px; \
                                    border-bottom: 1px solid var(--fs-border);",
                            span { style: "color: var(--fs-text-primary);", "{action.label}" }
                            span {
                                style: "font-family: var(--fs-font-mono); font-size: 11px; \
                                        color: var(--fs-text-secondary); \
                                        background: var(--fs-bg-elevated); \
                                        padding: 1px 6px; border-radius: 3px; \
                                        border: 1px solid var(--fs-border);",
                                "{shortcut}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── AI chat HTTP helper ───────────────────────────────────────────────────────

/// Sends all messages to the OpenAI-compatible API and returns the assistant reply.
/// Errors are returned as a user-visible error string instead of panicking.
async fn chat_request(api_base: &str, messages: &[ChatMsg]) -> String {
    let url = format!("{api_base}/chat/completions");

    let body = serde_json::json!({
        "model": "default",
        "messages": messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>>(),
        "max_tokens": 512,
        "temperature": 0.4,
    });

    let result = reqwest::Client::new().post(&url).json(&body).send().await;

    match result {
        Err(e) => format!("Request failed: {e}"),
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Err(e) => format!("Parse error: {e}"),
            Ok(json) => json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("(no response)")
                .to_string(),
        },
    }
}
