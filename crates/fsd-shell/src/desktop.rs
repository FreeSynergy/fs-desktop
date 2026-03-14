/// Desktop — root layout: wallpaper + window area + taskbar + launcher overlay + notifications.
use dioxus::prelude::*;

use fsd_conductor::ConductorApp;
use fsd_profile::ProfileApp;
use fsd_settings::SettingsApp;
use fsd_store::StoreApp;
use fsd_studio::StudioApp;

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
        open_app(&mut wm, &mut apps, &app_id);
        launcher.write().close();
    };

    // ── Launcher callbacks ───────────────────────────────────────────────────
    let on_launcher_launch = move |app_id: String| {
        open_app(&mut wm, &mut apps, &app_id);
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
        // Remove from app tracking so running-indicator clears
        for app in apps.write().iter_mut() {
            app.windows.retain(|&wid| wid != id);
        }
    };

    let on_focus_window = move |id: WindowId| {
        wm.write().focus(id);
    };

    let on_minimize_window = move |id: WindowId| {
        wm.write().minimize(id);
    };

    let on_maximize_window = move |id: WindowId| {
        wm.write().maximize(id);
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

                for window in wm.read().windows().iter().filter(|w| !w.minimized).cloned().collect::<Vec<_>>() {
                    WindowFrame {
                        key: "{window.id.0}",
                        window: window.clone(),
                        on_close: on_close_window,
                        on_focus: on_focus_window,
                        on_minimize: on_minimize_window,
                        on_maximize: on_maximize_window,

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

/// Opens or focuses an app window. Restores a minimized window via focus().
fn open_app(wm: &mut Signal<WindowManager>, apps: &mut Signal<Vec<AppEntry>>, app_id: &str) {
    // If already open (or minimized), focus it (focus() also restores minimized)
    let existing_id = apps
        .read()
        .iter()
        .find(|a| a.id == app_id)
        .and_then(|a| a.windows.first().copied());

    if let Some(win_id) = existing_id {
        wm.write().focus(win_id);
        return;
    }

    let title_key = format!("app-{}", app_id);
    let window = Window::new(title_key);
    let win_id = window.id;
    wm.write().open(window);

    // Track window ID so the taskbar running-indicator dot appears
    if let Some(app) = apps.write().iter_mut().find(|a| a.id == app_id) {
        app.windows.push(win_id);
    }

    tracing::info!("Opened app: {}", app_id);
}

/// Dispatches the correct app component based on the window's title_key.
#[component]
fn AppWindowContent(title_key: String) -> Element {
    match title_key.as_str() {
        "app-conductor" => rsx! { ConductorApp {} },
        "app-store"     => rsx! { StoreApp {} },
        "app-studio"    => rsx! { StudioApp {} },
        "app-settings"  => rsx! { SettingsApp {} },
        "app-profile"   => rsx! { ProfileApp {} },
        _ => rsx! {
            div {
                style: "color: var(--fsn-color-text-muted, #94a3b8); font-size: 13px; \
                        display: flex; align-items: center; justify-content: center; height: 200px;",
                "Unknown app: {title_key}"
            }
        },
    }
}
