/// Help View — context-sensitive help and keyboard shortcuts reference.
/// Also exports HelpSidebarPanel: the collapsible right-side help panel for the Desktop.
use dioxus::prelude::*;
use fs_components::{FsSidebar, FsSidebarItem, FS_SIDEBAR_CSS};
use fs_settings::{ShortcutsConfig, register_actions, resolve_shortcut};
use serde_json;

#[derive(Clone, PartialEq, Debug)]
enum HelpSection {
    Topics,
    Shortcuts,
}

impl HelpSection {
    fn id(&self) -> &'static str {
        match self {
            Self::Topics    => "topics",
            Self::Shortcuts => "shortcuts",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Topics    => "Topics",
            Self::Shortcuts => "Shortcuts",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Topics    => "📚",
            Self::Shortcuts => "⌨",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "topics"    => Some(Self::Topics),
            "shortcuts" => Some(Self::Shortcuts),
            _           => None,
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
    HelpTopic { id: "getting-started", title: "Getting Started",    summary: "Learn how to set up your first FreeSynergy.Node deployment." },
    HelpTopic { id: "container-app",   title: "Container",      summary: "Manage services, bots, and containers from the Container App view." },
    HelpTopic { id: "store",           title: "Module Store",       summary: "Browse, install, and update service modules from the store." },
    HelpTopic { id: "studio",          title: "Studio",             summary: "Create custom modules, plugins, and language packs." },
    HelpTopic { id: "settings",        title: "Settings",           summary: "Configure appearance, language, service roles, and AI connections." },
    HelpTopic { id: "ai-assistant",    title: "AI Assistant",       summary: "Use your local Ollama instance as an integrated AI helper." },
    HelpTopic { id: "troubleshooting", title: "Troubleshooting",    summary: "Common issues and how to resolve them." },
];

const ALL_SECTIONS: &[HelpSection] = &[HelpSection::Topics, HelpSection::Shortcuts];

/// Root component for the Help view.
#[component]
pub fn HelpApp() -> Element {
    let mut active = use_signal(|| HelpSection::Topics);

    let sidebar_items: Vec<FsSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsSidebarItem::new(s.id(), s.icon(), s.label()))
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

                FsSidebar {
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
        .filter(|t| q.is_empty() || t.title.to_lowercase().contains(&q) || t.summary.to_lowercase().contains(&q))
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
    categories.sort();
    categories.dedup();

    rsx! {
        div {
            class: "fs-scrollable", style: "overflow-y: auto; padding: 16px 24px; height: 100%;",
            p {
                style: "font-size: 12px; color: var(--fs-text-muted); margin-bottom: 16px;",
                "Shortcuts can be customized in Settings → Shortcuts."
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
// Layout when open (280 px):
//   ┌──────────────────┐
//   │ Header: Help [×] │
//   ├──────────────────┤
//   │ [Topics][Kbd]    │  ← tab strip
//   ├──────────────────┤
//   │ topic list /     │
//   │ shortcuts        │  ← scrollable content
//   ├──────────────────┤  (only when AI is running)
//   │ AI Chat          │  ← flex: 1
//   │ [input ↵]        │
//   └──────────────────┘

#[derive(Clone, PartialEq, Debug)]
enum SidebarTab {
    Topics,
    Shortcuts,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChatMsg {
    pub role:    &'static str,
    pub content: String,
}

/// Collapsible right-side help panel for the Desktop shell.
///
/// Collapsed (44 px): shows a vertical "❓ Help" tab on the right edge.
/// Hover over the tab or panel: expands leftward with a 300 ms CSS transition.
/// Width is resizable by dragging the left edge of the expanded panel.
/// AI chat section height is resizable by dragging the divider above it.
/// AI availability is checked periodically via HTTP ping; the section is hidden
/// when the AI service is not responding, and a notification is emitted on change.
#[component]
pub fn HelpSidebarPanel(
    #[props(default)]
    on_ai_offline: Option<EventHandler<()>>,
    #[props(default)]
    on_ai_online: Option<EventHandler<()>>,
) -> Element {
    let mut tab = use_signal(|| SidebarTab::Topics);

    // ── Panel geometry ────────────────────────────────────────────────────
    let mut panel_width:  Signal<f64> = use_signal(|| 280.0);
    let mut ai_height:    Signal<f64> = use_signal(|| 260.0);
    let mut hovered:      Signal<bool> = use_signal(|| false);

    // Width-resize state (dragging the left edge)
    let mut resizing_w:   Signal<bool> = use_signal(|| false);
    let mut resize_w_sx:  Signal<f64>  = use_signal(|| 0.0);
    let mut resize_w_sw:  Signal<f64>  = use_signal(|| 280.0);

    // AI-section height resize state (dragging the divider)
    let mut resizing_ai:  Signal<bool> = use_signal(|| false);
    let mut resize_ai_sy: Signal<f64>  = use_signal(|| 0.0);
    let mut resize_ai_sh: Signal<f64>  = use_signal(|| 260.0);

    // ── AI health ─────────────────────────────────────────────────────────
    // true  = AI responded to HTTP ping within 3 s
    // false = no response / not running
    let mut ai_online:     Signal<bool> = use_signal(|| false);
    let mut ai_was_online: Signal<bool> = use_signal(|| false);

    // Background task: ping every 30 s, emit callbacks on state change.
    {
        let on_offline = on_ai_offline.clone();
        let on_online  = on_ai_online.clone();
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
                        if let Some(ref cb) = on_online  { cb.call(()); }
                    } else if was {
                        if let Some(ref cb) = on_offline { cb.call(()); }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });
    }

    // ── Chat state ────────────────────────────────────────────────────────
    let mut messages: Signal<Vec<ChatMsg>> = use_signal(Vec::new);
    let mut input     = use_signal(String::new);
    let mut thinking  = use_signal(|| false);
    let ai_url        = fs_ai::ai_api_url();

    // ── Derived values ────────────────────────────────────────────────────
    let is_ai_online  = *ai_online.read();
    let is_resizing   = *resizing_w.read() || *resizing_ai.read();
    let is_open       = *hovered.read() || is_resizing;
    let pw            = *panel_width.read();
    let aih           = *ai_height.read();
    // At 44 px (collapsed) the body is hidden by overflow; full width when open.
    let effective_w   = if is_open { pw + 44.0 } else { 44.0 };

    const PANEL_CSS: &str = r#"
.fs-help-sidebar {
    flex-shrink: 0; display: flex; flex-direction: row;
    overflow: hidden;
    transition: width 300ms ease;
}
/* ── Body (left part, hidden when collapsed) ── */
.fs-help-sidebar__body {
    flex: 1; display: flex; flex-direction: row; min-width: 0; overflow: hidden;
}
.fs-help-sidebar__drag-edge {
    width: 6px; flex-shrink: 0; cursor: ew-resize;
    background: transparent;
    transition: background 120ms;
}
.fs-help-sidebar__drag-edge:hover { background: var(--fs-border-focus); }
.fs-help-sidebar__inner {
    flex: 1; display: flex; flex-direction: column; overflow: hidden;
    background: var(--fs-bg-surface);
    border-left: 1px solid var(--fs-border);
    min-width: 0;
}
/* ── Collapsed tab strip (right, always visible) ── */
.fs-help-sidebar__tab-strip {
    width: 44px; flex-shrink: 0;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: var(--fs-bg-surface);
    border-left: 1px solid var(--fs-border);
    gap: 6px; cursor: default; user-select: none;
}
.fs-help-sidebar__tab-strip span {
    writing-mode: vertical-rl; text-orientation: mixed;
    font-size: 11px; color: var(--fs-text-secondary);
    letter-spacing: 0.06em;
}
/* ── Header ── */
.fs-help-sidebar__header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 14px; border-bottom: 1px solid var(--fs-border);
    flex-shrink: 0; background: var(--fs-bg-elevated);
}
/* ── Tabs ── */
.fs-help-sidebar__tabs {
    display: flex; border-bottom: 1px solid var(--fs-border); flex-shrink: 0;
}
.fs-help-sidebar__tab {
    flex: 1; padding: 7px 0; text-align: center;
    font-size: 12px; cursor: pointer;
    background: none; border: none; border-bottom: 2px solid transparent;
    color: var(--fs-text-muted); font-family: inherit;
    transition: color 120ms, border-color 120ms;
}
.fs-help-sidebar__tab--active {
    color: var(--fs-primary); border-bottom-color: var(--fs-primary);
}
/* ── Content ── */
.fs-help-sidebar__content { flex: 1; overflow-y: auto; min-height: 0; }
/* ── AI resize divider ── */
.fs-help-sidebar__ai-resize {
    height: 6px; flex-shrink: 0; cursor: ns-resize;
    background: var(--fs-border);
    transition: background 120ms;
}
.fs-help-sidebar__ai-resize:hover { background: var(--fs-border-focus); }
/* ── AI chat section ── */
.fs-help-sidebar__ai {
    flex-shrink: 0; display: flex; flex-direction: column; overflow: hidden;
}
.fs-help-sidebar__ai-title {
    padding: 6px 12px; font-size: 11px; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--fs-accent); border-bottom: 1px solid var(--fs-border);
    flex-shrink: 0;
}
.fs-help-sidebar__chat-msgs {
    flex: 1; overflow-y: auto; padding: 8px 12px;
    display: flex; flex-direction: column; gap: 6px; min-height: 0;
}
.fs-help-sidebar__chat-input-row {
    display: flex; gap: 6px; padding: 8px 10px;
    border-top: 1px solid var(--fs-border); flex-shrink: 0;
}
.fs-help-sidebar__chat-input {
    flex: 1; padding: 7px 10px; border-radius: 6px;
    background: var(--fs-bg-input); border: 1px solid var(--fs-border);
    color: var(--fs-text-primary); font-size: 13px; font-family: inherit;
    outline: none; resize: none; height: 34px; line-height: 1.4;
}
.fs-help-sidebar__chat-send {
    padding: 0 12px; border-radius: 6px;
    background: var(--fs-primary); border: none;
    color: #fff; font-size: 13px; cursor: pointer; flex-shrink: 0;
}
.fs-help-sidebar__chat-send:disabled { opacity: 0.4; cursor: not-allowed; }
"#;

    rsx! {
        style { "{PANEL_CSS}" }

        // ── Width-resize overlay (fullscreen, active while dragging left edge) ─
        if *resizing_w.read() {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; \
                        pointer-events: all; cursor: ew-resize;",
                onmousemove: move |evt: MouseEvent| {
                    let c = evt.data().client_coordinates();
                    // dragging left = bigger panel (start_x - current_x = positive delta)
                    let dx = *resize_w_sx.read() - c.x;
                    let new_w = (*resize_w_sw.read() + dx).max(180.0).min(600.0);
                    panel_width.set(new_w);
                },
                onmouseup: move |_| resizing_w.set(false),
            }
        }

        // ── AI-height resize overlay ───────────────────────────────────────
        if *resizing_ai.read() {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; \
                        pointer-events: all; cursor: ns-resize;",
                onmousemove: move |evt: MouseEvent| {
                    let c = evt.data().client_coordinates();
                    // dragging down = bigger AI section
                    let dy = c.y - *resize_ai_sy.read();
                    let new_h = (*resize_ai_sh.read() + dy).max(100.0).min(450.0);
                    ai_height.set(new_h);
                },
                onmouseup: move |_| resizing_ai.set(false),
            }
        }

        div {
            class: "fs-help-sidebar",
            style: "width: {effective_w}px;",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| {
                if !*resizing_w.read() && !*resizing_ai.read() {
                    hovered.set(false);
                }
            },

            // ── Body: left edge drag + inner content ───────────────────────
            // Hidden by overflow:hidden when effective_w == 44 px.
            div { class: "fs-help-sidebar__body",

                // Left-edge drag handle — resize panel width
                div {
                    class: "fs-help-sidebar__drag-edge",
                    onmousedown: move |evt: MouseEvent| {
                        evt.stop_propagation();
                        let c = evt.data().client_coordinates();
                        resize_w_sx.set(c.x);
                        resize_w_sw.set(*panel_width.read());
                        resizing_w.set(true);
                    },
                }

                div { class: "fs-help-sidebar__inner",

                    // ── Header ──────────────────────────────────────────────
                    div { class: "fs-help-sidebar__header",
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

                    // ── Tab strip ────────────────────────────────────────────
                    div { class: "fs-help-sidebar__tabs",
                        button {
                            class: if *tab.read() == SidebarTab::Topics {
                                "fs-help-sidebar__tab fs-help-sidebar__tab--active"
                            } else { "fs-help-sidebar__tab" },
                            onclick: move |_| tab.set(SidebarTab::Topics),
                            "📚 Topics"
                        }
                        button {
                            class: if *tab.read() == SidebarTab::Shortcuts {
                                "fs-help-sidebar__tab fs-help-sidebar__tab--active"
                            } else { "fs-help-sidebar__tab" },
                            onclick: move |_| tab.set(SidebarTab::Shortcuts),
                            "⌨ Shortcuts"
                        }
                    }

                    // ── Content ──────────────────────────────────────────────
                    div { class: "fs-help-sidebar__content fs-scrollable",
                        match *tab.read() {
                            SidebarTab::Topics    => rsx! { SidebarTopicsView {} },
                            SidebarTab::Shortcuts => rsx! { SidebarShortcutsView {} },
                        }
                    }

                    // ── AI section (only when AI is online) ──────────────────
                    if is_ai_online {
                        // Divider — drag to resize AI section height
                        div {
                            class: "fs-help-sidebar__ai-resize",
                            onmousedown: move |evt: MouseEvent| {
                                evt.stop_propagation();
                                let c = evt.data().client_coordinates();
                                resize_ai_sy.set(c.y);
                                resize_ai_sh.set(*ai_height.read());
                                resizing_ai.set(true);
                            },
                        }

                        div {
                            class: "fs-help-sidebar__ai",
                            style: "height: {aih}px;",

                            div { class: "fs-help-sidebar__ai-title", "AI Assistant" }

                            div { class: "fs-help-sidebar__chat-msgs fs-scrollable",
                                if messages.read().is_empty() {
                                    p {
                                        style: "color: var(--fs-text-muted); font-size: 12px; \
                                                text-align: center; margin: 12px 0;",
                                        "Ask me anything about FreeSynergy…"
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
                                                            line-height: 1.5; white-space: pre-wrap;",
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
                                        "AI is thinking…"
                                    }
                                }
                            }

                            div { class: "fs-help-sidebar__chat-input-row",
                                input {
                                    r#type: "text",
                                    class: "fs-help-sidebar__chat-input",
                                    placeholder: "Ask a question…",
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
                                    class: "fs-help-sidebar__chat-send",
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
                                    "↵"
                                }
                            }
                        }
                    }
                }
            }

            // ── Tab strip (always visible, right edge) ─────────────────────
            div { class: "fs-help-sidebar__tab-strip",
                span { "❓" }
                span { "Help" }
            }
        }
    }
}

// ── Sidebar-specific compact views ───────────────────────────────────────────

/// Compact topic list for the sidebar (no search box to save space).
#[component]
fn SidebarTopicsView() -> Element {
    rsx! {
        div { style: "padding: 8px;",
            for topic in TOPICS {
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

/// Compact shortcuts list for the sidebar.
#[component]
fn SidebarShortcutsView() -> Element {
    let config  = ShortcutsConfig::load();
    let actions = register_actions();

    rsx! {
        div { style: "padding: 8px 12px;",
            for action in &actions {
                {
                    let shortcut = resolve_shortcut(action, &config).unwrap_or("—");
                    rsx! {
                        div {
                            key: "{action.id}",
                            style: "display: flex; align-items: center; justify-content: space-between; \
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

    let result = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await;

    match result {
        Err(e) => format!("Request failed: {e}"),
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Err(e) => format!("Parse error: {e}"),
                Ok(json) => {
                    json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("(no response)")
                        .to_string()
                }
            }
        }
    }
}
