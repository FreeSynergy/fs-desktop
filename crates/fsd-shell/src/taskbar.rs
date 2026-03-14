/// Taskbar — KDE-like panel always visible at the bottom of the screen.
use dioxus::prelude::*;

use crate::window::{WindowId, WindowManager};

/// A registered application that can appear in the taskbar.
#[derive(Clone, Debug, PartialEq)]
pub struct AppEntry {
    /// Unique identifier (e.g. "conductor", "store").
    pub id: String,
    /// i18n key for the display name.
    pub label_key: String,
    /// Icon name or SVG path.
    pub icon: String,
    /// Whether this app is pinned to the taskbar permanently.
    pub pinned: bool,
    /// Open window IDs belonging to this app (empty = not running).
    pub windows: Vec<WindowId>,
}

impl AppEntry {
    pub fn is_running(&self) -> bool {
        !self.windows.is_empty()
    }
}

/// Builds the default pinned apps list.
pub fn default_apps() -> Vec<AppEntry> {
    vec![
        AppEntry {
            id: "conductor".into(),
            label_key: "app-conductor".into(),
            icon: "⚙".into(),
            pinned: true,
            windows: vec![],
        },
        AppEntry {
            id: "store".into(),
            label_key: "app-store".into(),
            icon: "📦".into(),
            pinned: true,
            windows: vec![],
        },
        AppEntry {
            id: "studio".into(),
            label_key: "app-studio".into(),
            icon: "🔧".into(),
            pinned: true,
            windows: vec![],
        },
        AppEntry {
            id: "settings".into(),
            label_key: "app-settings".into(),
            icon: "⚙".into(),
            pinned: true,
            windows: vec![],
        },
    ]
}

/// Taskbar component — renders the bottom panel.
#[component]
pub fn Taskbar(apps: Vec<AppEntry>, on_launch: EventHandler<String>) -> Element {
    rsx! {
        div {
            class: "fsd-taskbar",
            style: "display: flex; align-items: center; height: 48px; background: var(--fsn-color-bg-sidebar); padding: 0 8px; gap: 4px;",

            // App launcher button
            button {
                class: "fsd-taskbar__launcher",
                style: "font-size: 20px; background: none; border: none; cursor: pointer; color: var(--fsn-color-text-inverse);",
                title: "App Launcher",
                onclick: move |_| on_launch.call("launcher".into()),
                "⊞"
            }

            // Divider
            div { style: "width: 1px; height: 32px; background: var(--fsn-color-border-default); margin: 0 4px;" }

            // Pinned + running apps
            for app in &apps {
                TaskbarApp {
                    key: "{app.id}",
                    app: app.clone(),
                    on_click: {
                        let id = app.id.clone();
                        move |_| on_launch.call(id.clone())
                    }
                }
            }

            // Spacer
            div { style: "flex: 1;" }

            // System tray area
            SystemTray {}

            // Clock
            Clock {}
        }
    }
}

/// A single app button in the taskbar.
#[component]
fn TaskbarApp(app: AppEntry, on_click: EventHandler<MouseEvent>) -> Element {
    let running = app.is_running();
    rsx! {
        button {
            class: if running { "fsd-taskbar__app fsd-taskbar__app--running" } else { "fsd-taskbar__app" },
            style: "display: flex; flex-direction: column; align-items: center; background: none; border: none; cursor: pointer; padding: 4px 8px; color: var(--fsn-color-text-inverse); position: relative;",
            title: "{app.label_key}",
            onclick: on_click,

            span { style: "font-size: 18px;", "{app.icon}" }

            // Running indicator dot
            if running {
                div {
                    style: "position: absolute; bottom: 2px; width: 4px; height: 4px; border-radius: 50%; background: var(--fsn-color-primary);"
                }
            }
        }
    }
}

/// System tray — shows sync status, notifications, network.
#[component]
fn SystemTray() -> Element {
    rsx! {
        div {
            class: "fsd-tray",
            style: "display: flex; align-items: center; gap: 8px; padding: 0 8px; color: var(--fsn-color-text-inverse); font-size: 14px;",
            span { title: "Sync OK", "⟳" }
            span { title: "Network", "⬡" }
            span { title: "Notifications", "🔔" }
        }
    }
}

/// Clock display in the taskbar.
#[component]
fn Clock() -> Element {
    // TODO: use use_future + chrono for live clock updates
    rsx! {
        div {
            class: "fsd-clock",
            style: "color: var(--fsn-color-text-inverse); font-size: 13px; padding: 0 12px; min-width: 60px; text-align: center;",
            "12:00"
        }
    }
}
