/// Desktop widgets — standalone UI cards that can be placed on the desktop
/// or embedded in any layout.
///
/// Every widget receives `w` and `h` (current slot size in pixels) so it can
/// scale its content proportionally. At the default size (scale = 1.0) the
/// layout is identical to the original design. Enlarging the slot increases
/// the scale factor; content stops growing when it reaches a max-scale cap.
///
/// Persistence: widget layout is stored in `fs-desktop.db` via `crate::db`.
use chrono::Local;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// ── WidgetKind ─────────────────────────────────────────────────────────────

/// All widget types that can appear on the home layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WidgetKind {
    Clock,
    SystemInfo,
    Messages,
    MyTasks,
    QuickNotes,
    Weather,
    /// Status widget for one bot instance (id = bot name).
    BotStatus(String),
    /// A widget installed from the Store, identified by its package ID.
    Custom(String),
}

impl WidgetKind {
    /// Human-readable label shown in the widget picker.
    pub fn label(&self) -> String {
        match self {
            WidgetKind::Clock         => "Clock".to_string(),
            WidgetKind::SystemInfo    => "System Info".to_string(),
            WidgetKind::Messages      => "Messages".to_string(),
            WidgetKind::MyTasks       => "My Tasks".to_string(),
            WidgetKind::QuickNotes    => "Quick Notes".to_string(),
            WidgetKind::Weather       => "Weather".to_string(),
            WidgetKind::BotStatus(id) => format!("Bot: {id}"),
            WidgetKind::Custom(id)    => id.clone(),
        }
    }

    /// Default (width, height) in pixels for a newly placed widget.
    pub fn default_size(&self) -> (f64, f64) {
        match self {
            WidgetKind::Clock         => (220.0, 140.0),
            WidgetKind::SystemInfo    => (280.0, 190.0),
            WidgetKind::QuickNotes    => (300.0, 230.0),
            WidgetKind::Messages      => (320.0, 220.0),
            WidgetKind::MyTasks       => (320.0, 220.0),
            WidgetKind::Weather        => (260.0, 160.0),
            WidgetKind::BotStatus(_)   => (260.0, 140.0),
            WidgetKind::Custom(_)      => (280.0, 180.0),
        }
    }

    /// Emoji icon used as fallback when no icon URL is available.
    pub fn icon(&self) -> &'static str {
        match self {
            WidgetKind::Clock         => "🕐",
            WidgetKind::SystemInfo    => "🖥",
            WidgetKind::Messages      => "📬",
            WidgetKind::MyTasks       => "✅",
            WidgetKind::QuickNotes    => "📝",
            WidgetKind::Weather        => "🌤",
            WidgetKind::BotStatus(_)   => "🤖",
            WidgetKind::Custom(_)      => "🧩",
        }
    }

    /// We10X SVG icon URL for the widget picker panel.
    pub fn icon_url(&self) -> &'static str {
        match self {
            WidgetKind::Clock      => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/preferences-system-time.svg",
            WidgetKind::SystemInfo => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/utilities-system-monitor.svg",
            WidgetKind::Messages   => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/internet-mail.svg",
            WidgetKind::MyTasks    => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/evolution-tasks.svg",
            WidgetKind::QuickNotes => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/accessories-text-editor.svg",
            WidgetKind::Weather    => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/indicator-weather.svg",
            WidgetKind::BotStatus(_) => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/internet-chat.svg",
            WidgetKind::Custom(_)    => "https://raw.githubusercontent.com/yeyushengfan258/We10X-icon-theme/master/src/apps/scalable/preferences-plugin-script.svg",
        }
    }

    /// Built-in widget kinds, in picker order. Does not include Custom variants.
    pub fn all() -> Vec<WidgetKind> {
        vec![
            WidgetKind::Clock,
            WidgetKind::SystemInfo,
            WidgetKind::Messages,
            WidgetKind::MyTasks,
            WidgetKind::QuickNotes,
            WidgetKind::Weather,
        ]
    }

    /// All widget kinds including store-installed Custom widgets.
    pub fn all_with_custom() -> Vec<WidgetKind> {
        use fs_db_desktop::package_registry::{PackageKind, PackageRegistry};
        let mut kinds = Self::all();
        for pkg in PackageRegistry::by_kind(PackageKind::Widget) {
            kinds.push(WidgetKind::Custom(pkg.id));
        }
        kinds
    }

    /// Persistence key string.
    pub fn as_str(&self) -> String {
        match self {
            WidgetKind::Clock         => "Clock".to_string(),
            WidgetKind::SystemInfo    => "SystemInfo".to_string(),
            WidgetKind::Messages      => "Messages".to_string(),
            WidgetKind::MyTasks       => "MyTasks".to_string(),
            WidgetKind::QuickNotes    => "QuickNotes".to_string(),
            WidgetKind::Weather        => "Weather".to_string(),
            WidgetKind::BotStatus(id)  => format!("bot:{id}"),
            WidgetKind::Custom(id)     => format!("custom:{id}"),
        }
    }

    /// Parse from persistence key string.
    pub fn from_str(s: &str) -> Option<WidgetKind> {
        match s {
            "Clock"      => Some(WidgetKind::Clock),
            "SystemInfo" => Some(WidgetKind::SystemInfo),
            "Messages"   => Some(WidgetKind::Messages),
            "MyTasks"    => Some(WidgetKind::MyTasks),
            "QuickNotes" => Some(WidgetKind::QuickNotes),
            "Weather"    => Some(WidgetKind::Weather),
            s if s.starts_with("bot:") => {
                Some(WidgetKind::BotStatus(s["bot:".len()..].to_string()))
            }
            s if s.starts_with("custom:") => {
                Some(WidgetKind::Custom(s["custom:".len()..].to_string()))
            }
            _ => None,
        }
    }
}

// ── WidgetSlot ─────────────────────────────────────────────────────────────

/// A widget instance placed in the home layer layout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidgetSlot {
    /// Unique ID within this layout session.
    pub id: u32,
    /// Which widget to render.
    pub kind: WidgetKind,
    /// X position on the desktop (pixels from left edge).
    #[serde(default)]
    pub x: f64,
    /// Y position on the desktop (pixels from top edge).
    #[serde(default)]
    pub y: f64,
    /// Width of the widget card in pixels.
    #[serde(default)]
    pub w: f64,
    /// Height of the widget card in pixels.
    #[serde(default)]
    pub h: f64,
}

// ── Layout persistence ─────────────────────────────────────────────────────

/// Default widget layout: Clock + SystemInfo side by side.
pub fn default_widget_layout() -> Vec<WidgetSlot> {
    let kinds = [WidgetKind::Clock, WidgetKind::SystemInfo];
    kinds.iter().enumerate().map(|(i, kind)| {
        let (w, h) = kind.default_size();
        WidgetSlot {
            id:   i as u32,
            kind: kind.clone(),
            x:    24.0 + (i as f64) * 296.0,
            y:    24.0,
            w,
            h,
        }
    }).collect()
}

/// Loads widget layout from `fs-desktop.db`. Falls back to default if empty.
pub fn load_widget_layout(db: &fs_db_desktop::FsdDb) -> Vec<WidgetSlot> {
    let slots = crate::db::load_widgets_from_db(db);
    if slots.is_empty() { default_widget_layout() } else { slots }
}

/// Persists the current widget layout to `fs-desktop.db` (async, fire-and-forget).
pub fn save_widget_layout(db: std::sync::Arc<fs_db_desktop::FsdDb>, slots: &[WidgetSlot]) {
    crate::db::save_widgets_to_db(db, slots.to_vec());
}

// ── WidgetRenderer trait + render_widget dispatch ──────────────────────────

/// Trait that gives each `WidgetKind` its own rendering behavior.
///
/// Adding a new widget = adding a variant + one `match` arm here.
/// Never add dispatch `match` arms outside this trait impl.
pub trait WidgetRenderer {
    fn render(&self, w: f64, h: f64) -> Element;
}

impl WidgetRenderer for WidgetKind {
    fn render(&self, w: f64, h: f64) -> Element {
        match self {
            WidgetKind::Clock         => rsx! { ClockWidget { w, h } },
            WidgetKind::SystemInfo    => rsx! { SystemInfoWidget { w, h } },
            WidgetKind::QuickNotes    => rsx! { QuickNotesWidget { w, h } },
            WidgetKind::Messages      => rsx! { MessagesWidget { w, h } },
            WidgetKind::MyTasks       => rsx! { MyTasksWidget { w, h } },
            WidgetKind::BotStatus(id) => rsx! { BotStatusWidget { bot_name: id.clone(), w, h } },
            other                     => rsx! { PlaceholderWidget { kind: other.clone() } },
        }
    }
}

/// Dispatches a `WidgetKind` to its concrete Dioxus component.
///
/// `w` / `h` are the current slot dimensions in pixels — widgets use them
/// to scale their content proportionally (font size, gap, etc.).
pub fn render_widget(kind: &WidgetKind, w: f64, h: f64) -> Element {
    kind.render(w, h)
}

/// Returns all widgets for use in the picker (built-in + store-installed).
pub fn all_picker_widgets() -> Vec<WidgetKind> {
    WidgetKind::all_with_custom()
}

// ── Scaling helper ─────────────────────────────────────────────────────────

/// Computes a scale factor relative to `(default_w, default_h)`.
///
/// - Scale is always ≥ 1.0 (never shrink below the default design).
/// - Capped at `max_scale` so fonts don't grow infinitely.
fn content_scale(w: f64, h: f64, default_w: f64, default_h: f64, max_scale: f64) -> f64 {
    (w / default_w).min(h / default_h).clamp(1.0, max_scale)
}

// ── ClockWidget ───────────────────────────────────────────────────────────────

/// A clock widget that updates every second.
///
/// Displays the current time (HH:MM:SS) and date (Weekday, DD Month YYYY).
/// Fonts scale proportionally when the slot is enlarged beyond the default size.
#[component]
pub fn ClockWidget(w: f64, h: f64) -> Element {
    let mut time_str = use_signal(|| Local::now().format("%H:%M:%S").to_string());
    let mut date_str = use_signal(|| Local::now().format("%A, %d %B %Y").to_string());

    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            time_str.set(Local::now().format("%H:%M:%S").to_string());
            date_str.set(Local::now().format("%A, %d %B %Y").to_string());
        }
    });

    // Scale fonts proportionally; only grow, never shrink below the default.
    let scale      = content_scale(w, h, 220.0, 140.0, 4.0);
    let time_font  = 36.0 * scale;
    let date_font  = 13.0 * scale;

    rsx! {
        div {
            class: "fs-widget fs-widget--clock",
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: 20px 24px; \
                    display: flex; flex-direction: column; \
                    align-items: center; justify-content: center; gap: 6px; \
                    overflow: hidden;",

            span {
                style: "font-size: {time_font}px; font-weight: 700; \
                        letter-spacing: 2px; font-variant-numeric: tabular-nums; \
                        color: var(--fs-color-primary); white-space: nowrap;",
                "{time_str}"
            }
            span {
                style: "font-size: {date_font}px; color: var(--fs-color-text-muted); \
                        white-space: nowrap; overflow: hidden; text-overflow: ellipsis; \
                        max-width: 100%;",
                "{date_str}"
            }
        }
    }
}

// ── SystemInfoWidget ──────────────────────────────────────────────────────────

/// Snapshot of system information.
#[derive(Clone, Default)]
struct SysInfo {
    hostname:   String,
    uptime:     String,
    mem_used:   String,
    mem_total:  String,
    disk_used:  String,
    disk_total: String,
}

/// A system-info widget showing hostname, uptime, memory and disk.
///
/// Reads `/etc/hostname`, `/proc/uptime`, `/proc/meminfo` and uses `df -h /`
/// for disk information. Refreshes every 10 seconds.
/// Content scales with slot size; rows fill the full widget width.
#[component]
pub fn SystemInfoWidget(w: f64, h: f64) -> Element {
    let mut info = use_signal(SysInfo::default);

    use_future(move || async move {
        loop {
            let snapshot = tokio::task::spawn_blocking(read_sys_info).await.unwrap_or_default();
            info.set(snapshot);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    let i = info.read();

    let scale      = content_scale(w, h, 280.0, 190.0, 3.0);
    let font_size  = (13.0 * scale).clamp(10.0, 32.0);
    let title_size = (12.0 * scale).clamp(9.0, 28.0);
    let gap        = (10.0 * scale).clamp(4.0, 24.0);
    let icon_size  = (16.0 * scale).clamp(12.0, 40.0);

    rsx! {
        div {
            class: "fs-widget fs-widget--sysinfo",
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: 16px 20px; \
                    display: flex; flex-direction: column; justify-content: center; align-items: center; \
                    gap: {gap}px; overflow: hidden;",

            div {
                style: "font-size: {title_size}px; font-weight: 600; \
                        text-transform: uppercase; letter-spacing: 0.08em; \
                        color: var(--fs-color-text-muted); \
                        border-bottom: 1px solid var(--fs-color-border-default); \
                        padding-bottom: 8px; flex-shrink: 0; \
                        width: 100%; text-align: center;",
                "System Info"
            }

            SysRow { icon: "🖥",  label: "Host",   value: i.hostname.clone(),
                     font_size, icon_size }
            SysRow { icon: "⏱",  label: "Uptime", value: i.uptime.clone(),
                     font_size, icon_size }
            SysRow { icon: "🧠",  label: "Memory",
                     value: format!("{} / {}", i.mem_used, i.mem_total),
                     font_size, icon_size }
            SysRow { icon: "💾",  label: "Disk",
                     value: format!("{} / {}", i.disk_used, i.disk_total),
                     font_size, icon_size }
        }
    }
}

// ── SysRow ────────────────────────────────────────────────────────────────────

#[component]
fn SysRow(
    icon:      String,
    label:     String,
    value:     String,
    font_size: f64,
    icon_size: f64,
) -> Element {
    let label_min_w = font_size * 4.5; // ~4.5 chars wide at current font size

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: {font_size * 0.7}px; \
                    font-size: {font_size}px; width: 100%; min-width: 0;",
            span {
                style: "font-size: {icon_size}px; min-width: {icon_size}px; \
                        text-align: center; flex-shrink: 0;",
                "{icon}"
            }
            span {
                style: "color: var(--fs-color-text-muted); \
                        min-width: {label_min_w}px; flex-shrink: 0;",
                "{label}"
            }
            span {
                style: "color: var(--fs-color-text-primary); font-weight: 500; \
                        flex: 1; min-width: 0; \
                        overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                if value.is_empty() { "—" } else { "{value}" }
            }
        }
    }
}

// ── QuickNotesWidget ──────────────────────────────────────────────────────────

/// A simple in-memory text area for quick notes.
///
/// No persistence — notes are cleared on restart. Textarea fills the slot.
#[component]
pub fn QuickNotesWidget(w: f64, h: f64) -> Element {
    let mut text = use_signal(|| String::new());

    let scale     = content_scale(w, h, 300.0, 230.0, 3.0);
    let font_size = (13.0 * scale).clamp(11.0, 32.0);
    // Header + padding ≈ 60px scaled, remainder goes to the textarea.
    let textarea_h = (h - 60.0 * scale).max(60.0);

    rsx! {
        div {
            class: "fs-widget fs-widget--notes",
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: 16px 20px; \
                    display: flex; flex-direction: column; gap: 10px; \
                    overflow: hidden;",

            div {
                style: "font-size: {font_size * 0.9}px; font-weight: 600; \
                        text-transform: uppercase; letter-spacing: 0.08em; \
                        color: var(--fs-color-text-muted); \
                        border-bottom: 1px solid var(--fs-color-border-default); \
                        padding-bottom: 8px; flex-shrink: 0;",
                "Quick Notes"
            }

            textarea {
                style: "background: var(--fs-color-bg-base, #0f172a); \
                        color: var(--fs-color-text-primary); \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: 6px; \
                        padding: 8px 10px; \
                        font-size: {font_size}px; font-family: inherit; \
                        resize: none; \
                        height: {textarea_h}px; width: 100%; \
                        outline: none; box-sizing: border-box; flex-shrink: 0;",
                placeholder: "Type your notes here…",
                value: "{text}",
                oninput: move |e| text.set(e.value()),
            }
        }
    }
}

// ── PlaceholderWidget ─────────────────────────────────────────────────────────

/// Shows a "coming soon" card for widget kinds not yet implemented.
#[component]
pub fn PlaceholderWidget(kind: WidgetKind) -> Element {
    let label = kind.label();
    let icon  = kind.icon().to_string();

    rsx! {
        div {
            class: "fs-widget fs-widget--placeholder",
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: 20px 24px; \
                    display: flex; flex-direction: column; align-items: center; \
                    justify-content: center; gap: 8px; \
                    overflow: hidden; opacity: 0.7;",

            span { style: "font-size: 28px;", "{icon}" }
            span {
                style: "font-size: 13px; font-weight: 600; \
                        color: var(--fs-color-text-primary);",
                "{label}"
            }
            span {
                style: "font-size: 11px; color: var(--fs-color-text-muted);",
                "coming soon"
            }
        }
    }
}

// ── MessagesWidget ────────────────────────────────────────────────────────────

/// A messages widget showing recent messages from the FreeSynergy message bus.
/// Displays an empty state until the message bus (Phase G) is connected.
#[component]
pub fn MessagesWidget(w: f64, h: f64) -> Element {
    let scale      = content_scale(w, h, 320.0, 220.0, 3.0);
    let font_size  = (13.0 * scale).clamp(10.0, 32.0);
    let title_size = (12.0 * scale).clamp(9.0, 28.0);
    let icon_size  = (28.0 * scale).clamp(20.0, 64.0);

    rsx! {
        div {
            class: "fs-widget fs-widget--messages",
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: 16px 20px; \
                    display: flex; flex-direction: column; gap: 10px; \
                    overflow: hidden;",

            // Header row: title + unread badge
            div {
                style: "font-size: {title_size}px; font-weight: 600; \
                        text-transform: uppercase; letter-spacing: 0.08em; \
                        color: var(--fs-color-text-muted); \
                        border-bottom: 1px solid var(--fs-color-border-default); \
                        padding-bottom: 8px; flex-shrink: 0; \
                        display: flex; align-items: center; justify-content: space-between;",
                span { "Messages" }
                span {
                    style: "background: var(--fs-color-primary); color: #fff; \
                            border-radius: 10px; padding: 1px 7px; \
                            font-size: {title_size * 0.9}px; font-weight: 700;",
                    "0"
                }
            }

            // Empty state
            div {
                style: "flex: 1; display: flex; flex-direction: column; \
                        align-items: center; justify-content: center; gap: 8px;",
                span { style: "font-size: {icon_size}px; opacity: 0.4;", "📭" }
                span {
                    style: "font-size: {font_size}px; color: var(--fs-color-text-muted);",
                    "No new messages"
                }
            }
        }
    }
}

// ── MyTasksWidget ─────────────────────────────────────────────────────────────

/// An in-memory task checklist widget.
///
/// Tasks can be added by typing in the input field and pressing Enter.
/// Completed tasks are struck through. State is not persisted across restarts.
#[derive(Clone)]
struct Task {
    id:   u32,
    text: String,
    done: bool,
}

#[component]
pub fn MyTasksWidget(w: f64, h: f64) -> Element {
    let mut tasks:         Signal<Vec<Task>> = use_signal(Vec::new);
    let mut new_task_text: Signal<String>    = use_signal(String::new);
    let mut next_id:       Signal<u32>       = use_signal(|| 0);

    let scale      = content_scale(w, h, 320.0, 220.0, 3.0);
    let font_size  = (13.0 * scale).clamp(10.0, 30.0);
    let title_size = (12.0 * scale).clamp(9.0, 26.0);
    let gap        = (4.0  * scale).clamp(2.0, 10.0);

    let task_list  = tasks.read().clone();
    let done_count = task_list.iter().filter(|t| t.done).count();
    let total      = task_list.len();

    rsx! {
        div {
            class: "fs-widget fs-widget--tasks",
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: 14px 18px; \
                    display: flex; flex-direction: column; gap: {gap * 2.0}px; \
                    overflow: hidden;",

            // Header
            div {
                style: "font-size: {title_size}px; font-weight: 600; \
                        text-transform: uppercase; letter-spacing: 0.08em; \
                        color: var(--fs-color-text-muted); \
                        border-bottom: 1px solid var(--fs-color-border-default); \
                        padding-bottom: 8px; flex-shrink: 0; \
                        display: flex; align-items: center; justify-content: space-between;",
                span { "My Tasks" }
                if total > 0 {
                    span {
                        style: "font-size: {title_size * 0.85}px; \
                                color: var(--fs-color-text-muted);",
                        "{done_count}/{total}"
                    }
                }
            }

            // Task list — scrollable
            div {
                style: "flex: 1; overflow-y: auto; min-height: 0; \
                        display: flex; flex-direction: column; gap: {gap}px;",
                if task_list.is_empty() {
                    div {
                        style: "flex: 1; display: flex; align-items: center; \
                                justify-content: center; \
                                font-size: {font_size}px; \
                                color: var(--fs-color-text-muted); opacity: 0.7;",
                        "No tasks yet"
                    }
                }
                for task in task_list.iter().cloned() {
                    {
                        let done_style = if task.done {
                            "text-decoration: line-through; opacity: 0.5;"
                        } else {
                            ""
                        };
                        rsx! {
                            div {
                                key: "{task.id}",
                                style: "display: flex; align-items: center; gap: 8px; \
                                        flex-shrink: 0; padding: 1px 0;",
                                input {
                                    r#type: "checkbox",
                                    checked: task.done,
                                    style: "width: {font_size}px; height: {font_size}px; \
                                            cursor: pointer; \
                                            accent-color: var(--fs-color-primary); flex-shrink: 0;",
                                    onchange: {
                                        let tid = task.id;
                                        move |_| {
                                            if let Some(t) = tasks.write().iter_mut().find(|t| t.id == tid) {
                                                t.done = !t.done;
                                            }
                                        }
                                    },
                                }
                                span {
                                    style: "font-size: {font_size}px; flex: 1; min-width: 0; \
                                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap; \
                                            color: var(--fs-color-text-primary); {done_style}",
                                    "{task.text}"
                                }
                            }
                        }
                    }
                }
            }

            // Add task row
            div {
                style: "display: flex; gap: 6px; flex-shrink: 0;",
                input {
                    r#type: "text",
                    placeholder: "Add a task…",
                    value: "{new_task_text}",
                    style: "flex: 1; padding: 5px 8px; \
                            background: var(--fs-color-bg-base, #0f172a); \
                            color: var(--fs-color-text-primary); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: 4px; \
                            font-size: {font_size * 0.9}px; font-family: inherit; \
                            outline: none; box-sizing: border-box;",
                    oninput: move |e| new_task_text.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            let text = new_task_text.read().trim().to_string();
                            if !text.is_empty() {
                                let id = *next_id.read();
                                next_id.set(id + 1);
                                tasks.write().push(Task { id, text, done: false });
                                new_task_text.set(String::new());
                            }
                        }
                    },
                }
            }
        }
    }
}

// ── system reads ─────────────────────────────────────────────────────────────

fn read_sys_info() -> SysInfo {
    SysInfo {
        hostname:   read_hostname(),
        uptime:     read_uptime(),
        mem_used:   read_mem_used(),
        mem_total:  read_mem_total(),
        disk_used:  read_disk_used(),
        disk_total: read_disk_total(),
    }
}

fn read_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn read_uptime() -> String {
    let raw = std::fs::read_to_string("/proc/uptime").unwrap_or_default();
    let secs: f64 = raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let secs = secs as u64;
    let days  = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins  = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

fn parse_meminfo_kb(key: &str) -> u64 {
    let raw = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    raw.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn kb_to_display(kb: u64) -> String {
    if kb >= 1_048_576 {
        format!("{:.1}G", kb as f64 / 1_048_576.0)
    } else if kb >= 1024 {
        format!("{:.0}M", kb as f64 / 1024.0)
    } else {
        format!("{kb}K")
    }
}

fn read_mem_total() -> String {
    kb_to_display(parse_meminfo_kb("MemTotal:"))
}

fn read_mem_used() -> String {
    let total     = parse_meminfo_kb("MemTotal:");
    let available = parse_meminfo_kb("MemAvailable:");
    kb_to_display(total.saturating_sub(available))
}

fn read_disk_used() -> String {
    disk_stat(true)
}

fn read_disk_total() -> String {
    disk_stat(false)
}

/// Returns used or total disk space for `/` via `df`.
fn disk_stat(used: bool) -> String {
    let out = std::process::Command::new("df")
        .args(["--output=used,size", "-k", "/"])
        .output();
    let Ok(out) = out else { return "?".into() };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines = text.lines();
    let _ = lines.next(); // header
    let data = lines.next().unwrap_or("");
    let mut parts = data.split_whitespace();
    let used_kb:  u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let total_kb: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    if used { kb_to_display(used_kb) } else { kb_to_display(total_kb) }
}

// ── BotStatusWidget ───────────────────────────────────────────────────────────

/// A compact status widget for one bot instance.
///
/// Shows: name, running/stopped status, last audit action, quick-open button.
/// Clicking the widget opens BotManager filtered to this bot.
#[component]
pub fn BotStatusWidget(bot_name: String, w: f64, h: f64) -> Element {
    let scale = content_scale(w, h, 260.0, 140.0, 1.6);
    let font  = 12.0 * scale;
    let title = 14.0 * scale;

    // In a real implementation this would read from the bot's SQLite DB.
    // For now we show a static status that will be wired up via Bus in Phase P.
    let status_color = "#22c55e";
    let status_text  = "● Running";

    rsx! {
        div {
            style: "width: 100%; height: 100%; box-sizing: border-box; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); \
                    padding: {16.0 * scale}px; \
                    display: flex; flex-direction: column; gap: {8.0 * scale}px; \
                    overflow: hidden; cursor: pointer;",

            // Header row: icon + name + status
            div {
                style: "display: flex; align-items: center; gap: {8.0 * scale}px;",
                span { style: "font-size: {title * 1.4}px;", "🤖" }
                div { style: "flex: 1;",
                    div {
                        style: "font-size: {title}px; font-weight: 700; \
                                color: var(--fs-color-text-primary); \
                                white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{bot_name}"
                    }
                    div {
                        style: "font-size: {font * 0.9}px; color: {status_color};",
                        "{status_text}"
                    }
                }
            }

            // Quick info row
            div {
                style: "font-size: {font}px; color: var(--fs-color-text-muted); \
                        border-top: 1px solid var(--fs-color-border-default); \
                        padding-top: {6.0 * scale}px; display: flex; justify-content: space-between;",
                span { "Open BotManager →" }
            }
        }
    }
}
