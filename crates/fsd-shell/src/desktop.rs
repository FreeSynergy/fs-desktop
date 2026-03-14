/// Desktop — root layout: wallpaper + window area + taskbar + launcher overlay + notifications.
use dioxus::prelude::*;

use crate::launcher::{AppLauncher, LauncherState};
use crate::notification::{NotificationKind, NotificationManager, NotificationStack};
use crate::taskbar::{default_apps, AppEntry, Taskbar};
use crate::wallpaper::Wallpaper;
use crate::window::{Window, WindowId, WindowManager};
use crate::window_frame::WindowFrame;

/// Root desktop component.
#[component]
pub fn Desktop() -> Element {
    let wallpaper       = use_signal(Wallpaper::default);
    let mut wm          = use_signal(WindowManager::default);
    let mut apps        = use_signal(default_apps);
    let mut launcher    = use_signal(LauncherState::default);
    let mut notifs      = use_signal(NotificationManager::default);

    let bg = wallpaper.read().to_css_background();

    // ── Taskbar launch callback ──────────────────────────────────────────────
    let on_taskbar_launch = move |app_id: String| {
        if app_id == "launcher" {
            launcher.write().toggle();
            return;
        }
        open_app(&mut wm, &apps.read(), &app_id);
        launcher.write().close();
    };

    // ── Launcher callbacks ───────────────────────────────────────────────────
    let on_launcher_launch = move |app_id: String| {
        open_app(&mut wm, &apps.read(), &app_id);
        launcher.write().close();
    };

    let on_launcher_query = move |q: String| {
        launcher.write().query = q;
    };

    let on_launcher_close = move |_: ()| {
        launcher.write().close();
    };

    // ── Window manager callbacks ─────────────────────────────────────────────
    let on_close_window = move |id: WindowId| {
        wm.write().close(id);
    };

    let on_focus_window = move |id: WindowId| {
        wm.write().focus(id);
    };

    // ── Notification dismiss ─────────────────────────────────────────────────
    let on_dismiss_notif = move |id: u64| {
        notifs.write().dismiss(id);
    };

    let launcher_state = launcher.read().clone();
    let notif_items    = notifs.read().items().to_vec();
    let app_list       = apps.read().clone();

    rsx! {
        div {
            id: "fsd-desktop",
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; overflow: hidden; {bg}",

            // ── Window area ──────────────────────────────────────────────────
            div {
                id: "fsd-window-area",
                style: "flex: 1; position: relative; overflow: hidden;",

                for window in wm.read().windows().to_vec() {
                    WindowFrame {
                        key: "{window.id.0}",
                        window: window.clone(),
                        on_close: on_close_window,
                        on_focus: on_focus_window,

                        // App-specific content rendered by label_key convention
                        AppWindowContent { title_key: window.title_key.clone() }
                    }
                }
            }

            // ── Taskbar ──────────────────────────────────────────────────────
            Taskbar {
                apps: app_list.clone(),
                on_launch: on_taskbar_launch,
            }

            // ── App Launcher overlay ─────────────────────────────────────────
            if launcher_state.open {
                AppLauncher {
                    apps: app_list,
                    query: launcher_state.query.clone(),
                    on_query_change: on_launcher_query,
                    on_launch: on_launcher_launch,
                    on_close: on_launcher_close,
                }
            }

            // ── Notification stack ───────────────────────────────────────────
            NotificationStack {
                notifications: notif_items,
                on_dismiss: on_dismiss_notif,
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Opens or focuses an app window by its ID.
fn open_app(wm: &mut Signal<WindowManager>, apps: &[AppEntry], app_id: &str) {
    // If already open, just focus the first matching window
    if let Some(app) = apps.iter().find(|a| a.id == app_id) {
        if let Some(&first_id) = app.windows.first() {
            wm.write().focus(first_id);
            return;
        }
    }

    let title_key = format!("app-{}", app_id);
    let window = Window::new(title_key);
    wm.write().open(window);

    tracing::info!("Opened app: {}", app_id);
}

/// Placeholder content component — each fsd-* crate will eventually inject its own.
#[component]
fn AppWindowContent(title_key: String) -> Element {
    rsx! {
        div {
            style: "color: var(--fsn-color-text-muted, #94a3b8); font-size: 13px; \
                    display: flex; align-items: center; justify-content: center; \
                    height: 200px;",
            "Loading "{title_key}"…"
        }
    }
}
