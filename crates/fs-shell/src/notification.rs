/// Notification / Toast system — ephemeral messages stacked in the top-right corner.
use dioxus::prelude::*;
use fs_i18n;

use crate::icons::{ICON_BELL, ICON_CLOSE, ICON_NOTIF_INFO, ICON_NOTIF_SUCCESS, ICON_NOTIF_WARNING, ICON_NOTIF_ERROR};

/// Severity level of a notification.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum NotificationKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Trait that gives a notification kind its visual identity.
pub trait NotificationStyle {
    fn accent_color(&self) -> &'static str;
    fn status_icon(&self)  -> &'static str;
}

impl NotificationStyle for NotificationKind {
    fn accent_color(&self) -> &'static str {
        match self {
            Self::Info    => "#06b6d4",
            Self::Success => "#22c55e",
            Self::Warning => "#f59e0b",
            Self::Error   => "#ef4444",
        }
    }

    fn status_icon(&self) -> &'static str {
        match self {
            Self::Info    => ICON_NOTIF_INFO,
            Self::Success => ICON_NOTIF_SUCCESS,
            Self::Warning => ICON_NOTIF_WARNING,
            Self::Error   => ICON_NOTIF_ERROR,
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
            class: "fs-notifications",
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

// ── NotificationHistory + NotificationBell ────────────────────────────────────

/// A persistent notification history entry.
#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    pub id: u64,
    pub kind: NotificationKind,
    pub title: String,
    pub body: Option<String>,
    pub read: bool,
}

/// Manages the notification history (bell panel).
#[derive(Clone, Default, PartialEq)]
pub struct NotificationHistory {
    next_id: u64,
    entries: Vec<HistoryEntry>,
}

impl NotificationHistory {
    pub fn push(&mut self, kind: NotificationKind, title: impl Into<String>, body: Option<String>) {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(0, HistoryEntry { id, kind, title: title.into(), body, read: false });
        if self.entries.len() > 50 {
            self.entries.truncate(50);
        }
    }

    pub fn mark_all_read(&mut self) {
        for e in &mut self.entries {
            e.read = true;
        }
    }

    pub fn unread_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.read).count()
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }
}

/// Top-bar bell icon + dropdown panel.
#[component]
pub fn NotificationBell(
    history: NotificationHistory,
    on_mark_read: EventHandler<()>,
) -> Element {
    let mut open = use_signal(|| false);
    let unread   = history.unread_count();
    let entries  = history.entries().to_vec();

    rsx! {
        div { style: "position: relative; flex-shrink: 0;",
            button {
                onclick: move |_| {
                    let v = *open.read();
                    open.set(!v);
                    if !v { on_mark_read.call(()); }
                },
                style: "position: relative; background: none; border: none; cursor: pointer; \
                        padding: 6px 8px; border-radius: 6px; \
                        color: var(--fs-color-text-muted); display: flex; align-items: center;",
                span { dangerous_inner_html: ICON_BELL }
                if unread > 0 {
                    span {
                        style: "position: absolute; top: 2px; right: 2px; \
                                background: #ef4444; color: #fff; \
                                font-size: 9px; font-weight: 700; \
                                border-radius: 999px; min-width: 14px; height: 14px; \
                                display: flex; align-items: center; justify-content: center; \
                                padding: 0 3px;",
                        "{unread}"
                    }
                }
            }

            if *open.read() {
                div {
                    style: "position: fixed; inset: 0; z-index: 299;",
                    onclick: move |_| open.set(false),
                }
                div {
                    style: "position: absolute; top: calc(100% + 8px); right: 0; \
                            width: 340px; max-height: 440px; \
                            background: var(--fs-color-bg-surface, #0f172a); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); \
                            box-shadow: 0 8px 32px rgba(0,0,0,0.5); \
                            z-index: 300; display: flex; flex-direction: column; overflow: hidden;",

                    div {
                        style: "padding: 12px 16px; \
                                border-bottom: 1px solid var(--fs-color-border-default); \
                                display: flex; align-items: center; justify-content: space-between;",
                        span { style: "font-size: 14px; font-weight: 600; color: var(--fs-color-text-primary);",
                            {fs_i18n::t("shell.notifications.title")}
                        }
                        if !entries.is_empty() {
                            {
                                let n_str = entries.len().to_string();
                                rsx! {
                                    span { style: "font-size: 11px; color: var(--fs-color-text-muted);",
                                        {fs_i18n::t_with("shell.notifications.count", &[("n", n_str.as_str())])}
                                    }
                                }
                            }
                        }
                    }

                    div { class: "fs-scrollable", style: "overflow-y: auto; flex: 1;",
                        if entries.is_empty() {
                            div {
                                style: "padding: 24px; text-align: center; \
                                        color: var(--fs-color-text-muted); font-size: 13px;",
                                {fs_i18n::t("shell.notifications.empty")}
                            }
                        }
                        for entry in &entries {
                            BellEntry { key: "{entry.id}", entry: entry.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BellEntry(entry: HistoryEntry) -> Element {
    let accent = entry.kind.accent_color();
    let icon   = entry.kind.status_icon();
    let bg     = if entry.read { "transparent" } else { "rgba(6,182,212,0.05)" };

    rsx! {
        div {
            style: "display: flex; gap: 10px; padding: 10px 16px; background: {bg}; \
                    border-bottom: 1px solid var(--fs-color-border-default);",
            span { style: "color: {accent}; flex-shrink: 0; display: flex; align-items: center; padding-top: 1px;",
                dangerous_inner_html: icon
            }
            div { style: "flex: 1; min-width: 0;",
                div {
                    style: "font-size: 12px; font-weight: 600; color: var(--fs-color-text-primary); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{entry.title}"
                }
                if let Some(body) = &entry.body {
                    div {
                        style: "font-size: 11px; color: var(--fs-color-text-muted); \
                                margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        "{body}"
                    }
                }
            }
        }
    }
}

// ── Toast ─────────────────────────────────────────────────────────────────────

/// A single toast notification.
#[component]
fn Toast(notification: Notification, on_dismiss: EventHandler<u64>) -> Element {
    let id = notification.id;
    let accent = notification.kind.accent_color();
    let icon = notification.kind.status_icon();

    rsx! {
        div {
            class: "fs-toast",
            style: "display: flex; align-items: flex-start; gap: 10px; \
                    background: var(--fs-color-bg-sidebar, #0f172a); \
                    border: 1px solid {accent}; border-radius: 8px; padding: 12px 14px; \
                    pointer-events: all; position: relative; \
                    box-shadow: 0 4px 16px rgba(0,0,0,0.5);",

            // Accent icon
            span {
                style: "color: {accent}; flex-shrink: 0; display: flex; align-items: center; margin-top: 1px;",
                dangerous_inner_html: icon
            }

            // Content
            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: "font-size: 13px; font-weight: 600; \
                            color: var(--fs-color-text-primary, #e2e8f0); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{notification.title}"
                }

                if let Some(body) = &notification.body {
                    div {
                        style: "font-size: 12px; color: var(--fs-color-text-muted, #94a3b8); \
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
                        color: var(--fs-color-text-muted, #94a3b8); \
                        display: flex; align-items: center; padding: 2px 4px;",
                title: fs_i18n::t("shell.notifications.dismiss"),
                onclick: move |_| on_dismiss.call(id),
                span { dangerous_inner_html: ICON_CLOSE }
            }
        }
    }
}
