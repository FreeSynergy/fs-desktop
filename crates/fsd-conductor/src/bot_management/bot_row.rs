// bot_row.rs — BotRow: single bot list entry with toggle and delete.

use dioxus::prelude::*;

use super::model::Bot;
use super::save_bots;

#[component]
pub fn BotRow(
    idx: usize,
    bot: Bot,
    mut bots: Signal<Vec<Bot>>,
    mut status_msg: Signal<Option<String>>,
) -> Element {
    let trigger_label = bot.trigger.label();
    let action_label  = bot.action.label();
    let opacity       = if bot.enabled { "1" } else { "0.55" };

    rsx! {
        div {
            key: "{idx}",
            style: "display: flex; align-items: center; gap: 12px; padding: 12px 14px; \
                    background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); \
                    margin-bottom: 8px; border: 1px solid var(--fsn-color-border-default); \
                    opacity: {opacity};",

            // Toggle
            input {
                r#type: "checkbox",
                checked: bot.enabled,
                style: "cursor: pointer; width: 16px; height: 16px; flex-shrink: 0;",
                onchange: move |_| {
                    bots.write()[idx].enabled = !bot.enabled;
                    save_bots(bots, status_msg);
                },
            }

            // Info
            div { style: "flex: 1; min-width: 0;",
                div { style: "font-weight: 500; font-size: 14px;", "{bot.name}" }
                if !bot.description.is_empty() {
                    div {
                        style: "font-size: 12px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                        "{bot.description}"
                    }
                }
                div { style: "display: flex; gap: 8px; margin-top: 4px;",
                    span {
                        style: "font-size: 11px; padding: 2px 8px; border-radius: 9999px; \
                                background: var(--fsn-color-bg-overlay); color: var(--fsn-color-text-muted);",
                        "⏱ {trigger_label}"
                    }
                    span {
                        style: "font-size: 11px; padding: 2px 8px; border-radius: 9999px; \
                                background: var(--fsn-color-bg-overlay); color: var(--fsn-color-text-muted);",
                        "▶ {action_label}"
                    }
                }
            }

            // Delete
            button {
                style: "color: var(--fsn-color-error); background: none; border: none; \
                        cursor: pointer; font-size: 18px; flex-shrink: 0;",
                onclick: move |_| {
                    bots.write().remove(idx);
                    save_bots(bots, status_msg);
                },
                "✕"
            }
        }
    }
}
