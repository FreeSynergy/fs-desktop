/// Desktop — root layout with CSS Grid: header · sidebar · window area · taskbar.
use dioxus::prelude::*;

use fsd_conductor::ConductorApp;
use fsd_profile::ProfileApp;
use fsd_settings::{DesktopConfig, SettingsApp, SidebarPosition};
use fsd_store::StoreApp;
use fsd_studio::StudioApp;

use crate::ai_view::AiApp;
use crate::app_shell::{AppMode, AppShell, GLOBAL_CSS, LayoutA, LayoutC};
use crate::help_view::HelpApp;
use crate::header::{Breadcrumb, ShellHeader};
use crate::launcher::{AppLauncher, LauncherState};
use crate::notification::{NotificationManager, NotificationStack};
use crate::sidebar::{ShellSidebar, SidebarSection, default_sidebar_sections};
use crate::taskbar::{default_apps, AppEntry, Taskbar};
use crate::wallpaper::Wallpaper;
use crate::window::{Window, WindowId, WindowManager};
use crate::window_frame::WindowFrame;

/// Root desktop component.
#[component]
pub fn Desktop() -> Element {
    let cfg                 = use_signal(DesktopConfig::load);
    let wallpaper           = use_signal(Wallpaper::default);
    let mut wm              = use_signal(WindowManager::default);
    let mut apps            = use_signal(default_apps);
    let mut launcher        = use_signal(LauncherState::default);
    let mut notifs          = use_signal(NotificationManager::default);
    let mut sidebar_collapsed = use_signal(|| cfg.read().sidebar.default_collapsed);
    let sidebar_sections: Signal<Vec<SidebarSection>> = use_signal(default_sidebar_sections);

    let bg = wallpaper.read().to_css_background();

    // ── Sidebar collapsed toggle ─────────────────────────────────────────────
    let on_sidebar_toggle = move |_: ()| {
        let v = *sidebar_collapsed.read();
        *sidebar_collapsed.write() = !v;
    };

    // ── Sidebar app select ───────────────────────────────────────────────────
    let on_sidebar_select = move |app_id: String| {
        open_app(&mut wm, &mut apps, &app_id);
        launcher.write().close();
    };

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
    let on_launcher_query = move |q: String| { launcher.write().query = q; };
    let on_launcher_close = move |_: ()| { launcher.write().close(); };

    // ── Window manager callbacks ─────────────────────────────────────────────
    let on_close_window = move |id: WindowId| {
        wm.write().close(id);
        for app in apps.write().iter_mut() {
            app.windows.retain(|&wid| wid != id);
        }
    };
    let on_focus_window    = move |id: WindowId| { wm.write().focus(id); };
    let on_minimize_window = move |id: WindowId| { wm.write().minimize(id); };
    let on_maximize_window = move |id: WindowId| { wm.write().maximize(id); };

    // ── Notification dismiss ─────────────────────────────────────────────────
    let on_dismiss_notif = move |id: u64| { notifs.write().dismiss(id); };

    // ── Derived state ────────────────────────────────────────────────────────
    let launcher_state = launcher.read().clone();
    let notif_items    = notifs.read().items().to_vec();
    let app_list       = apps.read().clone();
    let collapsed      = *sidebar_collapsed.read();
    let sidebar_cfg    = cfg.read().sidebar.clone();
    let expanded_width = format!("{}px", sidebar_cfg.width);
    let col_width      = if collapsed { "48px" } else { expanded_width.as_str() };

    // Grid layout adapts to sidebar position
    let (grid_areas, grid_cols, grid_rows) = match sidebar_cfg.position {
        SidebarPosition::Left => (
            "'header header' 'sidebar main' 'taskbar taskbar'".to_string(),
            format!("{col_width} 1fr"),
            "60px 1fr 48px".to_string(),
        ),
        SidebarPosition::Right => (
            "'header header' 'main sidebar' 'taskbar taskbar'".to_string(),
            format!("1fr {col_width}"),
            "60px 1fr 48px".to_string(),
        ),
        SidebarPosition::Top => (
            "'header' 'sidebar' 'main' 'taskbar'".to_string(),
            "1fr".to_string(),
            format!("60px {col_width} 1fr 48px"),
        ),
        SidebarPosition::Bottom => (
            "'header' 'main' 'sidebar' 'taskbar'".to_string(),
            "1fr".to_string(),
            format!("60px 1fr {col_width} 48px"),
        ),
    };

    // Active sidebar item from the focused window
    let active_app_id = wm.read()
        .windows()
        .iter()
        .filter(|w| !w.minimized)
        .max_by_key(|w| w.z_index)
        .and_then(|w| w.title_key.strip_prefix("app-").map(String::from))
        .unwrap_or_default();

    // Breadcrumbs from focused window
    let breadcrumbs = wm.read()
        .windows()
        .iter()
        .filter(|w| !w.minimized)
        .max_by_key(|w| w.z_index)
        .map(|w| {
            let label = w.title_key.trim_start_matches("app-");
            let label = match label {
                "conductor" => "Conductor",
                "store"     => "Store",
                "studio"    => "Studio",
                "settings"  => "Settings",
                "profile"   => "Profile",
                "ai"        => "AI Assistant",
                "help"      => "Help",
                other       => other,
            };
            vec![Breadcrumb::new(label)]
        })
        .unwrap_or_else(|| vec![Breadcrumb::new("Desktop")]);

    rsx! {
        // Inject Midnight Blue theme variables at the root — ensures CSS vars are
        // available before any component renders (no FOUC for background colour).
        style { "{GLOBAL_CSS}" }

        div {
            id: "fsd-desktop",
            style: "
                width: 100vw; height: 100vh; overflow: hidden;
                display: grid;
                grid-template-areas: {grid_areas};
                grid-template-rows: {grid_rows};
                grid-template-columns: {grid_cols};
                background: var(--fsn-bg-base);
                {bg}
            ",

            // ── Header ───────────────────────────────────────────────────────
            div { style: "grid-area: header;",
                ShellHeader {
                    breadcrumbs,
                    user_name: "Admin".to_string(),
                    user_avatar: None,
                }
            }

            // ── Sidebar ───────────────────────────────────────────────────────
            div { style: "grid-area: sidebar; overflow: hidden;",
                ShellSidebar {
                    sections: sidebar_sections.read().clone(),
                    active_id: active_app_id,
                    collapsed,
                    on_select: on_sidebar_select,
                    on_toggle: on_sidebar_toggle,
                }
            }

            // ── Window area ───────────────────────────────────────────────────
            div {
                id: "fsd-window-area",
                style: "grid-area: main; position: relative; overflow: hidden;",

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

            // ── Taskbar ───────────────────────────────────────────────────────
            div { style: "grid-area: taskbar;",
                Taskbar {
                    apps: app_list.clone(),
                    on_launch: on_taskbar_launch,
                }
            }

            // ── App Launcher overlay ──────────────────────────────────────────
            if launcher_state.open {
                AppLauncher {
                    apps: app_list,
                    query: launcher_state.query.clone(),
                    on_query_change: on_launcher_query,
                    on_launch: on_launcher_launch,
                    on_close: on_launcher_close,
                }
            }

            // ── Notification stack ────────────────────────────────────────────
            NotificationStack {
                notifications: notif_items,
                on_dismiss: on_dismiss_notif,
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn open_app(wm: &mut Signal<WindowManager>, apps: &mut Signal<Vec<AppEntry>>, app_id: &str) {
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

    if let Some(app) = apps.write().iter_mut().find(|a| a.id == app_id) {
        app.windows.push(win_id);
    }
    tracing::info!("Opened app: {}", app_id);
}

/// Wraps each app in the appropriate layout (A / B / C).
#[component]
fn AppWindowContent(title_key: String) -> Element {
    match title_key.as_str() {
        "app-conductor" => rsx! {
            AppShell { mode: AppMode::Window,
                ConductorApp {}
            }
        },
        "app-store" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { StoreApp {} }
            }
        },
        "app-studio" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { StudioApp {} }
            }
        },
        "app-settings" => rsx! {
            AppShell { mode: AppMode::Window,
                SettingsApp {}
            }
        },
        "app-profile" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutC { ProfileApp {} }
            }
        },
        "app-ai" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { AiApp {} }
            }
        },
        "app-help" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { HelpApp {} }
            }
        },
        _ => rsx! {
            div {
                style: "color: var(--fsn-color-text-muted, #94a3b8); font-size: 13px; \
                        display: flex; align-items: center; justify-content: center; height: 200px;",
                "Unknown app: {title_key}"
            }
        },
    }
}
