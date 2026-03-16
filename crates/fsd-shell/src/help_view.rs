/// Help View — context-sensitive help for the FreeSynergy.Desktop.
///
/// Planned features:
/// - Keyword search over help topics (backed by fsn-help from FreeSynergy.Lib)
/// - Context-aware topics pushed by the active app
/// - Quick links to documentation and community resources
///
/// Currently renders a stub with the topic index.
use dioxus::prelude::*;

/// A single help topic entry.
#[derive(Clone, PartialEq)]
struct HelpTopic {
    /// Short slug used for lookup.
    id: &'static str,
    /// Display title.
    title: &'static str,
    /// Brief description.
    summary: &'static str,
}

const TOPICS: &[HelpTopic] = &[
    HelpTopic { id: "getting-started", title: "Getting Started",         summary: "Learn how to set up your first FreeSynergy.Node deployment." },
    HelpTopic { id: "conductor",       title: "Conductor",               summary: "Manage services, bots, and containers from the Conductor view." },
    HelpTopic { id: "store",           title: "Module Store",            summary: "Browse, install, and update service modules from the store." },
    HelpTopic { id: "studio",          title: "Studio",                  summary: "Create custom modules, plugins, and language packs." },
    HelpTopic { id: "settings",        title: "Settings",                summary: "Configure appearance, language, service roles, and AI connections." },
    HelpTopic { id: "ai-assistant",    title: "AI Assistant",            summary: "Use your local Ollama instance as an integrated AI helper." },
    HelpTopic { id: "troubleshooting", title: "Troubleshooting",         summary: "Common issues and how to resolve them." },
    HelpTopic { id: "keyboard",        title: "Keyboard Shortcuts",      summary: "Speed up your workflow with keyboard shortcuts." },
];

/// Root component for the Help view.
#[component]
pub fn HelpApp() -> Element {
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
        div {
            class: "fsd-help-view",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fsn-color-bg-base);",

            // Header
            div {
                style: "padding: 20px 24px 12px; \
                        background: var(--fsn-color-bg-surface, #1e293b); \
                        border-bottom: 1px solid var(--fsn-color-border-default, #334155);",
                h2 {
                    style: "margin: 0 0 10px; font-size: 20px; \
                            color: var(--fsn-color-text-primary, #e2e8f0);",
                    "Help & Documentation"
                }
                input {
                    r#type: "text",
                    placeholder: "Search help topics…",
                    style: "width: 100%; max-width: 480px; padding: 8px 12px; border-radius: 6px; \
                            background: var(--fsn-color-bg-input, #0f172a); \
                            border: 1px solid var(--fsn-color-border-default, #334155); \
                            color: var(--fsn-color-text-primary, #e2e8f0); font-size: 14px; \
                            outline: none; box-sizing: border-box;",
                    value: query.read().clone(),
                    oninput: move |evt| query.set(evt.value()),
                }
            }

            // Topic list
            div {
                style: "flex: 1; overflow-y: auto; padding: 16px 24px;",
                if filtered.is_empty() {
                    p {
                        style: "color: var(--fsn-color-text-muted, #94a3b8); font-size: 14px;",
                        "No topics found for \"{query.read().clone()}\""
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        for topic in filtered {
                            HelpTopicCard { topic: topic.clone() }
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
