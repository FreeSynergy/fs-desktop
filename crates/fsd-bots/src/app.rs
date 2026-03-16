/// Bot Manager — usage interface for messaging bots.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};

use crate::broadcast_view::BroadcastView;
use crate::gatekeeper_view::GatekeeperView;
use crate::model::{BotKind, MessagingBot, MessagingBotsConfig};

#[component]
pub fn BotManagerApp() -> Element {
    let mut bots = use_signal(MessagingBotsConfig::load);
    let mut selected_idx: Signal<Option<usize>> = use_signal(|| Some(0));

    let bot_list = bots.read().clone();
    let sel_idx = *selected_idx.read();
    let selected = sel_idx.and_then(|i| bot_list.get(i).cloned());

    let active_id = sel_idx
        .and_then(|i| bot_list.get(i))
        .map(|b| b.id.clone())
        .unwrap_or_default();

    let sidebar_items: Vec<FsnSidebarItem> = bot_list.iter()
        .map(|b| FsnSidebarItem::new(b.id.clone(), b.kind.icon().to_string(), b.name.clone()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fsn-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fsn-border); \
                        flex-shrink: 0; background: var(--fsn-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fsn-text-primary);",
                    "Bots"
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                FsnSidebar {
                    items: sidebar_items,
                    active_id,
                    on_select: move |id: String| {
                        let idx = bots.read().iter().position(|b| b.id == id);
                        selected_idx.set(idx);
                    },
                }

                div {
                    style: "flex: 1; overflow: auto; padding: 20px;",

                    match selected {
                        None => rsx! {
                            div {
                                style: "display: flex; align-items: center; justify-content: center; \
                                        height: 200px; color: var(--fsn-color-text-muted); font-size: 13px;",
                                "Select a bot from the list"
                            }
                        },
                        Some(bot) => rsx! {
                            BotDetail {
                                bot,
                                on_update: move |updated: MessagingBot| {
                                    if let Some(i) = sel_idx {
                                        bots.write()[i] = updated;
                                        let _ = MessagingBotsConfig::save(&*bots.read());
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn BotDetail(bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
    let status_color = if bot.enabled { "#22c55e" } else { "#64748b" };
    let status_label = if bot.enabled { "● Running" } else { "○ Stopped" };
    let kind = bot.kind.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px;",

            div { style: "display: flex; align-items: center; gap: 12px;",
                span { style: "font-size: 28px;", "{bot.kind.icon()}" }
                div {
                    h2 { style: "margin: 0; font-size: 18px; color: var(--fsn-color-text-primary);",
                        "{bot.name}"
                    }
                    span { style: "font-size: 12px; color: {status_color};", "{status_label}" }
                }
            }

            match kind {
                BotKind::Broadcast => rsx! {
                    BroadcastView { bot, on_update }
                },
                BotKind::Gatekeeper => rsx! {
                    GatekeeperView { bot, on_update }
                },
                _ => rsx! {
                    div {
                        style: "background: var(--fsn-color-bg-overlay); \
                                border-radius: var(--fsn-radius-md); \
                                padding: 20px; color: var(--fsn-color-text-muted); font-size: 13px;",
                        "This bot type does not have a usage interface yet."
                    }
                },
            }
        }
    }
}
