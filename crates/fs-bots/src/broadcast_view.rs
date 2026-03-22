use dioxus::prelude::*;
use chrono::Utc;
use crate::model::{BroadcastRecord, ChannelTarget, MessagingBot};

#[component]
pub fn BroadcastView(bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
    let mut message = use_signal(String::new);
    let targets = bot.targets.clone();
    let recent = bot.recent_broadcasts.clone();
    let enabled_count = targets.iter().filter(|t| t.enabled).count();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",

            div {
                label {
                    style: "display: block; font-size: 12px; font-weight: 600; \
                            color: var(--fs-color-text-muted); margin-bottom: 6px; \
                            text-transform: uppercase; letter-spacing: 0.06em;",
                    "Message"
                }
                textarea {
                    style: "width: 100%; min-height: 100px; \
                            background: var(--fs-color-bg-overlay); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); \
                            padding: 10px 12px; font-size: 13px; font-family: inherit; \
                            color: var(--fs-color-text-primary); resize: vertical; \
                            box-sizing: border-box;",
                    placeholder: "Type your broadcast message…",
                    oninput: move |e| message.set(e.value()),
                }
            }

            div {
                label {
                    style: "display: block; font-size: 12px; font-weight: 600; \
                            color: var(--fs-color-text-muted); margin-bottom: 8px; \
                            text-transform: uppercase; letter-spacing: 0.06em;",
                    "Send to"
                }
                div { style: "display: flex; flex-direction: column; gap: 4px;",
                    for t in &targets {
                        TargetRow { target: t.clone() }
                    }
                }
            }

            button {
                style: "align-self: flex-start; background: var(--fs-color-primary); \
                        color: #fff; border: none; border-radius: var(--fs-radius-md); \
                        padding: 10px 20px; font-size: 14px; font-weight: 600; \
                        cursor: pointer;",
                onclick: {
                    let msg = message.read().clone();
                    let mut updated = bot.clone();
                    move |_| {
                        if msg.trim().is_empty() { return; }
                        let record = BroadcastRecord {
                            message: msg.clone(),
                            sent_at: Utc::now(),
                            target_count: enabled_count,
                        };
                        updated.recent_broadcasts.insert(0, record.clone());
                        if updated.recent_broadcasts.len() > 20 {
                            updated.recent_broadcasts.truncate(20);
                        }
                        on_update.call(updated.clone());
                    }
                },
                "📤 Send Broadcast"
            }

            if !recent.is_empty() {
                div {
                    div {
                        style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                                letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 8px;",
                        "Recent Broadcasts"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        for record in &recent {
                            BroadcastRecordRow { record: record.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TargetRow(target: ChannelTarget) -> Element {
    let check = if target.enabled { "☑" } else { "☐" };
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 8px; font-size: 13px; \
                    color: var(--fs-color-text-primary); padding: 4px 0;",
            span { style: "color: var(--fs-color-primary);", "{check}" }
            span { style: "color: var(--fs-color-text-muted); font-size: 11px;", "{target.platform}:" }
            span { "{target.name}" }
        }
    }
}

#[component]
fn BroadcastRecordRow(record: BroadcastRecord) -> Element {
    let preview = if record.message.len() > 50 {
        format!("{}…", &record.message[..47])
    } else {
        record.message.clone()
    };
    let secs = (Utc::now() - record.sent_at).num_seconds();
    let time_ago = if secs < 60 { format!("{secs}s ago") }
        else if secs < 3600 { format!("{}m ago", secs / 60) }
        else { format!("{}h ago", secs / 3600) };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; font-size: 12px; \
                    color: var(--fs-color-text-muted); padding: 5px 0; \
                    border-bottom: 1px solid var(--fs-color-border-default);",
            span { "📤" }
            span {
                style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; \
                        color: var(--fs-color-text-primary);",
                "\"{preview}\""
            }
            span { "— {time_ago}" }
            span { "— {record.target_count} targets" }
        }
    }
}
