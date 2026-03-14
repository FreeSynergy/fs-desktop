/// Notification / Toast system — ephemeral messages stacked in the top-right corner.
use std::time::Duration;

use dioxus::prelude::*;

/// Severity level of a notification.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum NotificationKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationKind {
    fn accent_color(&self) -> &'static str {
        match self {
            Self::Info    => "#06b6d4",
            Self::Success => "#22c55e",
            Self::Warning => "#f59e0b",
            Self::Error   => "#ef4444",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Info    => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Error   => "✕",
        }
    }
}

/// A single notification entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: u64,
    pub kind: NotificationKind,
    pub title: String,
    pub body: Option<String>,
}

/// Manages the stack of active notifications.
#[derive(Clone, Default)]
pub struct NotificationManager {
    next_id: u64,
    items: Vec<Notification>,
}

impl NotificationManager {
    /// Push a new notification. Returns its ID.
    pub fn push(&mut self, kind: NotificationKind, title: impl Into<String>, body: Option<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(Notification { id, kind, title: title.into(), body });
        // Keep at most 5 toasts visible
        if self.items.len() > 5 {
            self.items.remove(0);
        }
        id
    }

    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|n| n.id != id);
    }

    pub fn items(&self) -> &[Notification] {
        &self.items
    }
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct NotificationStackProps {
    pub notifications: Vec<Notification>,
    pub on_dismiss: EventHandler<u64>,
}

/// Renders all active toasts stacked in the top-right corner.
#[component]
pub fn NotificationStack(props: NotificationStackProps) -> Element {
    rsx! {
        div {
            class: "fsd-notifications",
            style: "position: fixed; top: 16px; right: 16px; z-index: 9999; \
                    display: flex; flex-direction: column; gap: 8px; \
                    pointer-events: none; width: 320px;",

            for notif in &props.notifications {
                Toast {
                    key: "{notif.id}",
                    notification: notif.clone(),
                    on_dismiss: props.on_dismiss,
                }
            }
        }
    }
}

/// A single toast notification.
#[component]
fn Toast(notification: Notification, on_dismiss: EventHandler<u64>) -> Element {
    let id = notification.id;
    let accent = notification.kind.accent_color();
    let icon = notification.kind.icon();

    rsx! {
        div {
            class: "fsd-toast",
            style: "display: flex; align-items: flex-start; gap: 10px; \
                    background: var(--fsn-color-bg-sidebar, #0f172a); \
                    border: 1px solid {accent}; border-radius: 8px; padding: 12px 14px; \
                    pointer-events: all; position: relative; \
                    box-shadow: 0 4px 16px rgba(0,0,0,0.5);",

            // Accent icon
            span {
                style: "font-size: 16px; color: {accent}; flex-shrink: 0; margin-top: 1px;",
                "{icon}"
            }

            // Content
            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: "font-size: 13px; font-weight: 600; \
                            color: var(--fsn-color-text-primary, #e2e8f0); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{notification.title}"
                }

                if let Some(body) = &notification.body {
                    div {
                        style: "font-size: 12px; color: var(--fsn-color-text-muted, #94a3b8); \
                                margin-top: 3px; overflow: hidden; \
                                display: -webkit-box; -webkit-line-clamp: 2;",
                        "{body}"
                    }
                }
            }

            // Dismiss button
            button {
                style: "position: absolute; top: 6px; right: 8px; \
                        background: none; border: none; cursor: pointer; \
                        color: var(--fsn-color-text-muted, #94a3b8); font-size: 14px; \
                        line-height: 1; padding: 2px 4px;",
                title: "Dismiss",
                onclick: move |_| on_dismiss.call(id),
                "×"
            }
        }
    }
}
