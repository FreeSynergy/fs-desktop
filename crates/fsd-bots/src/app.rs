/// Bot Manager — manage messenger accounts and bot configurations.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};
use fsn_i18n;

use crate::accounts_view::AccountsView;
use crate::broadcast_view::BroadcastView;
use crate::gatekeeper_view::GatekeeperView;
use crate::groups_view::GroupsView;
use crate::model::{BotKind, MessagingBot, MessagingBotsConfig};

/// Active section in the Bot Manager.
#[derive(Clone, PartialEq, Debug)]
pub enum BotSection {
    Accounts,
    Bots,
    Broadcast,
    Gatekeeper,
    Groups,
}

impl BotSection {
    pub fn id(&self) -> &str {
        match self {
            Self::Accounts   => "accounts",
            Self::Bots       => "bots",
            Self::Broadcast  => "broadcast",
            Self::Gatekeeper => "gatekeeper",
            Self::Groups     => "groups",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Accounts   => fsn_i18n::t("bots.section.accounts"),
            Self::Bots       => fsn_i18n::t("bots.section.bots"),
            Self::Broadcast  => fsn_i18n::t("bots.section.broadcast"),
            Self::Gatekeeper => fsn_i18n::t("bots.section.gatekeeper"),
            Self::Groups     => fsn_i18n::t("bots.section.groups"),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Accounts   => "🔑",
            Self::Bots       => "🤖",
            Self::Broadcast  => "📢",
            Self::Gatekeeper => "🔒",
            Self::Groups     => "📁",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "accounts"   => Some(Self::Accounts),
            "bots"       => Some(Self::Bots),
            "broadcast"  => Some(Self::Broadcast),
            "gatekeeper" => Some(Self::Gatekeeper),
            "groups"     => Some(Self::Groups),
            _            => None,
        }
    }
}

const ALL_SECTIONS: &[BotSection] = &[
    BotSection::Accounts,
    BotSection::Bots,
    BotSection::Broadcast,
    BotSection::Gatekeeper,
    BotSection::Groups,
];

/// Root component of the Bot Manager.
#[component]
pub fn BotManagerApp() -> Element {
    let mut active = use_signal(|| BotSection::Accounts);
    let mut bots   = use_signal(MessagingBotsConfig::load);
    let mut selected_idx: Signal<Option<usize>> = use_signal(|| Some(0));

    let sidebar_items: Vec<FsnSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsnSidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    // Find first broadcast/gatekeeper bot for the dedicated views
    let broadcast_bot   = bots.read().iter().find(|b| b.kind == BotKind::Broadcast).cloned();
    let gatekeeper_bot  = bots.read().iter().find(|b| b.kind == BotKind::Gatekeeper).cloned();
    let broadcast_idx   = bots.read().iter().position(|b| b.kind == BotKind::Broadcast);
    let gatekeeper_idx  = bots.read().iter().position(|b| b.kind == BotKind::Gatekeeper);

    let bot_list = bots.read().clone();
    let sel_idx  = *selected_idx.read();
    let selected = sel_idx.and_then(|i| bot_list.get(i).cloned());

    let active_bot_id = sel_idx
        .and_then(|i| bot_list.get(i))
        .map(|b| b.id.clone())
        .unwrap_or_default();

    let bots_sidebar_items: Vec<FsnSidebarItem> = bot_list.iter()
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
                    {fsn_i18n::t("bots.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                FsnSidebar {
                    items:     sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = BotSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                div {
                    style: "flex: 1; overflow: auto; padding: 20px;",

                    match *active.read() {
                        BotSection::Accounts => rsx! {
                            AccountsView {}
                        },

                        BotSection::Bots => rsx! {
                            div { style: "display: flex; height: 100%; overflow: hidden;",
                                // Bot list sidebar
                                div {
                                    style: "width: 220px; border-right: 1px solid var(--fsn-border); overflow-y: auto;",
                                    FsnSidebar {
                                        items: bots_sidebar_items,
                                        active_id: active_bot_id,
                                        on_select: move |id: String| {
                                            let idx = bots.read().iter().position(|b| b.id == id);
                                            selected_idx.set(idx);
                                        },
                                    }
                                }
                                // Bot detail
                                div { style: "flex: 1; overflow: auto; padding: 0 20px;",
                                    match selected {
                                        None => rsx! {
                                            div {
                                                style: "display: flex; align-items: center; \
                                                        justify-content: center; height: 200px; \
                                                        color: var(--fsn-color-text-muted); font-size: 13px;",
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
                        },

                        BotSection::Broadcast => rsx! {
                            match broadcast_bot {
                                Some(bot) => rsx! {
                                    BroadcastView {
                                        bot,
                                        on_update: move |updated: MessagingBot| {
                                            if let Some(i) = broadcast_idx {
                                                bots.write()[i] = updated;
                                                let _ = MessagingBotsConfig::save(&*bots.read());
                                            }
                                        }
                                    }
                                },
                                None => rsx! {
                                    div {
                                        style: "color: var(--fsn-color-text-muted); font-size: 13px;",
                                        "No Broadcast bot configured."
                                    }
                                },
                            }
                        },

                        BotSection::Gatekeeper => rsx! {
                            match gatekeeper_bot {
                                Some(bot) => rsx! {
                                    GatekeeperView {
                                        bot,
                                        on_update: move |updated: MessagingBot| {
                                            if let Some(i) = gatekeeper_idx {
                                                bots.write()[i] = updated;
                                                let _ = MessagingBotsConfig::save(&*bots.read());
                                            }
                                        }
                                    }
                                },
                                None => rsx! {
                                    div {
                                        style: "color: var(--fsn-color-text-muted); font-size: 13px;",
                                        "No Gatekeeper bot configured."
                                    }
                                },
                            }
                        },

                        BotSection::Groups => rsx! {
                            GroupsView {}
                        },
                    }
                }
            }
        }
    }
}

// ── BotDetail ─────────────────────────────────────────────────────────────────

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
