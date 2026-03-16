/// Help View — context-sensitive help and keyboard shortcuts reference.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};
use fsd_settings::{ShortcutsConfig, register_actions, resolve_shortcut};

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
    HelpTopic { id: "conductor",       title: "Conductor",          summary: "Manage services, bots, and containers from the Conductor view." },
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

    let sidebar_items: Vec<FsnSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsnSidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-help-view",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fsn-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fsn-border); \
                        flex-shrink: 0; background: var(--fsn-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fsn-text-primary);",
                    "Help & Documentation"
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                FsnSidebar {
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
                            background: var(--fsn-color-bg-input, #0f172a); \
                            border: 1px solid var(--fsn-color-border-default, #334155); \
                            color: var(--fsn-color-text-primary, #e2e8f0); font-size: 14px; \
                            outline: none; box-sizing: border-box;",
                    oninput: move |evt| query.set(evt.value()),
                }
            }
            div {
                class: "fsn-scrollable", style: "flex: 1; overflow-y: auto; padding: 0 24px 16px;",
                if filtered.is_empty() {
                    p { style: "color: var(--fsn-color-text-muted); font-size: 14px;",
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
            class: "fsn-scrollable", style: "overflow-y: auto; padding: 16px 24px; height: 100%;",
            p {
                style: "font-size: 12px; color: var(--fsn-text-muted); margin-bottom: 16px;",
                "Shortcuts can be customized in Settings → Shortcuts."
            }
            for cat in &categories {
                {
                    let cat_actions: Vec<&fsd_settings::ActionDef> = actions.iter().filter(|a| a.category == *cat).collect();
                    rsx! {
                        div { style: "margin-bottom: 20px;",
                            div {
                                style: "font-size: 11px; font-weight: 600; text-transform: uppercase; \
                                        letter-spacing: 0.08em; color: var(--fsn-text-muted); \
                                        margin-bottom: 6px; padding-bottom: 4px; \
                                        border-bottom: 1px solid var(--fsn-border);",
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
                                            span { style: "color: var(--fsn-text-primary);", "{action.label}" }
                                            span {
                                                style: "font-family: var(--fsn-font-mono); font-size: 12px; \
                                                        color: var(--fsn-text-secondary); \
                                                        background: var(--fsn-bg-elevated); \
                                                        padding: 2px 8px; border-radius: var(--fsn-radius-sm); \
                                                        border: 1px solid var(--fsn-border);",
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
            class: "fsd-help-topic",
            style: "padding: 14px 16px; border-radius: 8px; \
                    background: var(--fsn-color-bg-surface, #1e293b); \
                    border: 1px solid var(--fsn-color-border-default, #334155); \
                    cursor: pointer;",
            h3 {
                style: "margin: 0 0 4px; font-size: 15px; \
                        color: var(--fsn-color-primary, #06b6d4);",
                "{topic.title}"
            }
            p {
                style: "margin: 0; font-size: 13px; \
                        color: var(--fsn-color-text-muted, #94a3b8);",
                "{topic.summary}"
            }
        }
    }
}
