/// AI View — stub for a future AI assistant integrated into the desktop.
///
/// Planned features:
/// - Ollama-backed local LLM assistant
/// - Context-aware help (reads current service state)
/// - Module generation assistance
///
/// Currently renders a placeholder until the Ollama integration is wired up.
use dioxus::prelude::*;

/// Root component for the AI view.
#[component]
pub fn AiApp() -> Element {
    rsx! {
        div {
            class: "fsd-ai-view",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fsn-color-bg-base); padding: 32px; \
                    align-items: center; justify-content: center; gap: 16px;",

            div {
                style: "font-size: 48px; line-height: 1;",
                "🤖"
            }
            h2 {
                style: "margin: 0; font-size: 22px; \
                        color: var(--fsn-color-text-primary, #e2e8f0);",
                "AI Assistant"
            }
            p {
                style: "margin: 0; color: var(--fsn-color-text-muted, #94a3b8); \
                        font-size: 14px; text-align: center; max-width: 360px;",
                "Local AI assistant powered by Ollama. \
                 Connect your Ollama instance in Settings to enable chat, \
                 module generation, and context-aware help."
            }
            div {
                style: "padding: 12px 20px; border-radius: 6px; \
                        background: var(--fsn-color-bg-surface, #1e293b); \
                        border: 1px solid var(--fsn-color-border-default, #334155); \
                        color: var(--fsn-color-text-muted, #94a3b8); \
                        font-size: 13px;",
                "Coming soon — configure Ollama in Settings → AI"
            }
        }
    }
}
