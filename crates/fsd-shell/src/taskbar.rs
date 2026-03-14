/// Taskbar — KDE-like panel always visible at the bottom of the screen.
use chrono::Local;
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

            TaskbarLauncherBtn {
                on_click: move |_| on_launch.call("launcher".into())
            }
            TaskbarSeparator {}
            TaskbarApps {
                apps: apps.clone(),
                on_launch: {
                    let on_launch = on_launch.clone();
                    move |id| on_launch.call(id)
                }
            }
            div { style: "flex: 1;" }
            SystemTray {}
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

/// Clock display in the taskbar — updates every second.
#[component]
fn Clock() -> Element {
    let mut time_str = use_signal(|| Local::now().format("%H:%M").to_string());
    let mut date_str = use_signal(|| Local::now().format("%d.%m.%Y").to_string());

    // Refresh every second
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            time_str.set(Local::now().format("%H:%M").to_string());
            date_str.set(Local::now().format("%d.%m.%Y").to_string());
        }
    });

    rsx! {
        div {
            class: "fsd-clock",
            style: "display: flex; flex-direction: column; align-items: center; \
                    color: var(--fsn-color-text-inverse, #e2e8f0); padding: 0 12px; min-width: 72px;",
            span {
                style: "font-size: 13px; font-weight: 600; line-height: 1.2;",
                "{time_str}"
            }
            span {
                style: "font-size: 10px; color: var(--fsn-color-text-muted, #94a3b8); line-height: 1.2;",
                "{date_str}"
            }
        }
    }
}

/// Launcher button slot — opens the App Launcher overlay.
#[component]
pub fn TaskbarLauncherBtn(on_click: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "fsd-taskbar__launcher",
            style: "font-size: 20px; background: none; border: none; cursor: pointer; \
                    color: var(--fsn-color-text-inverse); padding: 4px 8px;",
            title: "App Launcher",
            onclick: on_click,
            "⊞"
        }
    }
}

/// Visual separator between taskbar slots.
#[component]
pub fn TaskbarSeparator() -> Element {
    rsx! {
        div { style: "width: 1px; height: 32px; background: var(--fsn-color-border-default); margin: 0 4px;" }
    }
}

/// The running apps slot — all pinned + open apps.
#[component]
pub fn TaskbarApps(apps: Vec<AppEntry>, on_launch: EventHandler<String>) -> Element {
    rsx! {
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
    }
}
