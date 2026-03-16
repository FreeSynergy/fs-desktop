/// Desktop — root layout: header + content area with auto-hide overlay sidebar.
use std::time::Duration;
use dioxus::prelude::*;

use fsd_bots::BotManagerApp;
use fsd_conductor::ConductorApp;
use fsd_profile::ProfileApp;
use fsd_settings::SettingsApp;
use fsd_store::StoreApp;
use fsd_studio::StudioApp;
use fsd_tasks::TasksApp;

use crate::ai_view::AiApp;
use crate::app_shell::{AppMode, AppShell, GLOBAL_CSS, LayoutA, LayoutC};
use crate::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::help_view::HelpApp;
use crate::header::{Breadcrumb, ShellHeader};
use crate::launcher::{AppLauncher, LauncherState};
use crate::notification::{NotificationHistory, NotificationManager, NotificationStack};
use crate::sidebar::{ShellSidebar, SidebarSection, default_sidebar_sections};
use crate::taskbar::{AppEntry, default_apps};
use crate::wallpaper::Wallpaper;
use crate::widgets::{WidgetKind, WidgetSlot, load_widget_layout, render_widget, save_widget_layout};
use crate::window::{Window, WindowId, WindowManager};
use crate::window_frame::WindowFrame;

/// Root desktop component.
#[component]
pub fn Desktop() -> Element {
    let wallpaper           = use_signal(Wallpaper::default);
    let mut wm              = use_signal(WindowManager::default);
    let mut apps            = use_signal(default_apps);
    let mut launcher        = use_signal(LauncherState::default);
    let mut notifs          = use_signal(NotificationManager::default);
    let mut notif_history   = use_signal(NotificationHistory::default);
    let mut ctx_menu        = use_signal(|| ContextMenuState::default());
    let sidebar_sections: Signal<Vec<SidebarSection>> = use_signal(default_sidebar_sections);
    let mut theme: Signal<String> = use_context_provider(|| Signal::new("midnight-blue".to_string()));

    // ── Widget layer state ─────────────────────────────────────────────────
    let mut widget_layout   = use_signal(load_widget_layout);
    let mut edit_mode       = use_signal(|| false);
    let mut next_widget_id  = use_signal(|| 100u32);
    let mut picker_open     = use_signal(|| false);

    // Sidebar auto-hide: visible = shown (translateX(0)), hidden = off-screen (translateX(-240px))
    let mut sidebar_visible  = use_signal(|| true);
    let mut sidebar_hide_gen = use_signal(|| 0u32);

    let bg = wallpaper.read().to_css_background();

    // ── Theme + menu action handler ────────────────────────────────────────
    let menu_action_handler = move |id: String| {
        match id.as_str() {
            "theme-midnight-blue" => theme.set("midnight-blue".to_string()),
            "theme-cloud-white"   => theme.set("cloud-white".to_string()),
            "theme-cupertino"     => theme.set("cupertino".to_string()),
            "theme-nordic"        => theme.set("nordic".to_string()),
            "theme-rose-pine"     => theme.set("rose-pine".to_string()),
            "launcher"            => launcher.write().toggle(),
            "open-tasks"          => open_app(&mut wm, &mut apps, "tasks"),
            "open-bots"           => open_app(&mut wm, &mut apps, "bots"),
            _ => {}
        }
    };

    // ── Sidebar auto-hide logic ────────────────────────────────────────────
    let on_sidebar_enter = move |_: MouseEvent| {
        let gen = *sidebar_hide_gen.read() + 1;
        *sidebar_hide_gen.write() = gen;
        *sidebar_visible.write() = true;
    };
    let on_sidebar_leave = move |_: MouseEvent| {
        let gen = *sidebar_hide_gen.read() + 1;
        *sidebar_hide_gen.write() = gen;
        spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if *sidebar_hide_gen.read() == gen {
                *sidebar_visible.write() = false;
            }
        });
    };
    let on_edge_enter = move |_: MouseEvent| {
        let gen = *sidebar_hide_gen.read() + 1;
        *sidebar_hide_gen.write() = gen;
        *sidebar_visible.write() = true;
    };

    // ── Sidebar app select ─────────────────────────────────────────────────
    let on_sidebar_select = move |app_id: String| {
        open_app(&mut wm, &mut apps, &app_id);
        launcher.write().close();
    };

    // ── Launcher callbacks ──────────────────────────────────────────────────
    let on_launcher_launch = move |app_id: String| {
        open_app(&mut wm, &mut apps, &app_id);
        launcher.write().close();
    };
    let on_launcher_query = move |q: String| { launcher.write().query = q; };
    let on_launcher_close = move |_: ()| { launcher.write().close(); };

    // ── Window manager callbacks ────────────────────────────────────────────
    let on_close_window = move |id: WindowId| {
        wm.write().close(id);
        for app in apps.write().iter_mut() {
            app.windows.retain(|&wid| wid != id);
        }
    };
    let on_focus_window    = move |id: WindowId| { wm.write().focus(id); };
    let on_minimize_window = move |id: WindowId| { wm.write().minimize(id); };
    let on_maximize_window = move |id: WindowId| { wm.write().maximize(id); };

    // ── Notification dismiss ────────────────────────────────────────────────
    let on_dismiss_notif = move |id: u64| { notifs.write().dismiss(id); };

    // ── Widget edit mode callbacks ──────────────────────────────────────────

    // Enter edit mode.
    let on_edit_desktop = move |_: MouseEvent| {
        edit_mode.set(true);
        picker_open.set(false);
    };

    // Exit edit mode and persist the current layout.
    let on_done_editing = move |_: MouseEvent| {
        edit_mode.set(false);
        picker_open.set(false);
        save_widget_layout(&widget_layout.read());
    };

    // Clear all widgets.
    let on_clear_all = move |_: MouseEvent| {
        widget_layout.write().clear();
        picker_open.set(false);
    };

    // Toggle the widget picker panel.
    let on_toggle_picker = move |_: MouseEvent| {
        let open = *picker_open.read();
        picker_open.set(!open);
    };

    // ── Derived state ───────────────────────────────────────────────────────
    let launcher_state = launcher.read().clone();
    let notif_items    = notifs.read().items().to_vec();
    let app_list       = apps.read().clone();
    let visible        = *sidebar_visible.read();
    let sidebar_transform = if visible { "translateX(0)" } else { "translateX(-240px)" };
    let in_edit_mode   = *edit_mode.read();
    let is_picker_open = *picker_open.read();

    let active_app_id = wm.read()
        .windows()
        .iter()
        .filter(|w| !w.minimized)
        .max_by_key(|w| w.z_index)
        .and_then(|w| w.title_key.strip_prefix("app-").map(String::from))
        .unwrap_or_default();

    let breadcrumbs = wm.read()
        .windows()
        .iter()
        .filter(|w| !w.minimized)
        .max_by_key(|w| w.z_index)
        .map(|w| vec![Breadcrumb::new(app_id_to_label(w.title_key.trim_start_matches("app-")))])
        .unwrap_or_else(|| vec![Breadcrumb::new("Desktop")]);

    rsx! {
        style { "{GLOBAL_CSS}" }

        div {
            id: "fsd-desktop",
            "data-theme": "{theme}",
            style: "
                width: 100vw; height: 100vh; overflow: hidden;
                display: flex; flex-direction: column;
                background: var(--fsn-bg-base);
                {bg}
            ",

            // ── Header ─────────────────────────────────────────────────────
            div { style: "flex-shrink: 0;",
                ShellHeader {
                    breadcrumbs,
                    user_name: "Admin".to_string(),
                    user_avatar: None,
                    on_menu_action: Some(EventHandler::new(menu_action_handler)),
                    history: notif_history.read().clone(),
                    on_mark_read: Some(EventHandler::new(move |_| notif_history.write().mark_all_read())),
                }
            }

            // ── Content area (home layer + window area + sidebar) ───────────
            div {
                style: "flex: 1; position: relative; overflow: hidden;",
                oncontextmenu: move |e: MouseEvent| {
                    e.prevent_default();
                    let coords = e.client_coordinates();
                    ctx_menu.set(ContextMenuState::open_at(
                        coords.x,
                        coords.y,
                        vec![
                            ContextMenuItem::new("edit-desktop", "Edit Desktop").with_icon("✏"),
                            ContextMenuItem::new("add-widget",   "Add Widget").with_icon("＋"),
                            ContextMenuItem::new("settings",     "Settings").with_icon("⚙"),
                        ],
                    ));
                },

                // ── Home layer — widgets sit on the desktop background ──────
                div {
                    id: "fsd-home-layer",
                    style: "position: absolute; inset: 0; padding: 24px; \
                            display: flex; flex-wrap: wrap; \
                            gap: 16px; align-items: flex-start; align-content: flex-start; \
                            overflow: hidden; pointer-events: none;",

                    for slot in widget_layout.read().clone().into_iter() {
                        HomeWidgetCard {
                            key: "{slot.id}",
                            slot: slot.clone(),
                            edit_mode: in_edit_mode,
                            on_remove: move |id: u32| {
                                widget_layout.write().retain(|s| s.id != id);
                            },
                        }
                    }
                }

                // ── Window area (full size — sidebar overlays on top) ───────
                div {
                    id: "fsd-window-area",
                    style: "position: absolute; inset: 0; overflow: hidden;",
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

                // ── Sidebar — absolute overlay with auto-hide animation ──────
                div {
                    style: "position: absolute; top: 0; left: 0; height: 100%; z-index: 50; \
                            width: 240px; \
                            transform: {sidebar_transform}; \
                            transition: transform 300ms ease;",
                    onmouseenter: on_sidebar_enter,
                    onmouseleave: on_sidebar_leave,
                    ShellSidebar {
                        sections: sidebar_sections.read().clone(),
                        active_id: active_app_id,
                        on_select: on_sidebar_select,
                    }
                }

                // Edge trigger strip — shows sidebar when mouse approaches the left edge
                if !visible {
                    div {
                        style: "position: absolute; top: 0; left: 0; width: 8px; height: 100%; \
                                z-index: 49; cursor: default;",
                        onmouseenter: on_edge_enter,
                    }
                }

                // App Launcher overlay
                if launcher_state.open {
                    AppLauncher {
                        apps: app_list,
                        query: launcher_state.query.clone(),
                        on_query_change: on_launcher_query,
                        on_launch: on_launcher_launch,
                        on_close: on_launcher_close,
                    }
                }

                // Notification stack
                NotificationStack {
                    notifications: notif_items,
                    on_dismiss: on_dismiss_notif,
                }

                // Context menu
                ContextMenu {
                    state: ctx_menu.read().clone(),
                    on_action: move |id: String| {
                        match id.as_str() {
                            "edit-desktop" => edit_mode.set(true),
                            "settings"     => open_app(&mut wm, &mut apps, "settings"),
                            _ => {}
                        }
                        ctx_menu.set(ContextMenuState::default());
                    },
                    on_close: move |_| ctx_menu.set(ContextMenuState::default()),
                }

                // ── Edit Desktop button (bottom-right, outside edit mode) ────
                if !in_edit_mode {
                    div {
                        style: "position: absolute; bottom: 16px; right: 16px; z-index: 60;",
                        button {
                            onclick: on_edit_desktop,
                            style: "background: var(--fsn-color-bg-surface); \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: 8px; \
                                    padding: 6px 14px; \
                                    font-size: 12px; font-family: inherit; \
                                    color: var(--fsn-color-text-muted); \
                                    cursor: pointer; opacity: 0.75; \
                                    transition: opacity 150ms;",
                            "✏ Edit Desktop"
                        }
                    }
                }

                // ── Edit Mode toolbar (bottom bar) ───────────────────────────
                if in_edit_mode {
                    div {
                        id: "fsd-edit-toolbar",
                        style: "position: absolute; bottom: 0; left: 0; right: 0; z-index: 60; \
                                background: var(--fsn-color-bg-surface); \
                                border-top: 1px solid var(--fsn-color-border-default); \
                                padding: 10px 20px; \
                                display: flex; align-items: center; gap: 10px;",

                        // "+ Add Widget" with picker panel
                        div { style: "position: relative;",
                            button {
                                onclick: on_toggle_picker,
                                style: "background: var(--fsn-color-primary, #06b6d4); \
                                        color: #fff; \
                                        border: none; border-radius: 6px; \
                                        padding: 7px 14px; \
                                        font-size: 13px; font-family: inherit; \
                                        cursor: pointer;",
                                if is_picker_open { "▲ Add Widget" } else { "▼ Add Widget" }
                            }

                            // Widget picker panel (floats above toolbar)
                            if is_picker_open {
                                div {
                                    id: "fsd-widget-picker",
                                    style: "position: absolute; bottom: calc(100% + 8px); left: 0; \
                                            background: var(--fsn-color-bg-surface); \
                                            border: 1px solid var(--fsn-color-border-default); \
                                            border-radius: 10px; \
                                            padding: 6px 0; \
                                            min-width: 220px; \
                                            z-index: 70; \
                                            box-shadow: 0 8px 24px rgba(0,0,0,0.4);",

                                    for kind in WidgetKind::all() {
                                        WidgetPickerRow {
                                            kind: kind.clone(),
                                            on_add: move |k: WidgetKind| {
                                                let id = *next_widget_id.read();
                                                next_widget_id.set(id + 1);
                                                widget_layout.write().push(WidgetSlot { id, kind: k });
                                                picker_open.set(false);
                                            },
                                        }
                                    }
                                }
                            }
                        }

                        // "Clear All" button
                        button {
                            onclick: on_clear_all,
                            style: "background: transparent; \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: 6px; \
                                    padding: 7px 14px; \
                                    font-size: 13px; font-family: inherit; \
                                    color: var(--fsn-color-text-muted); \
                                    cursor: pointer;",
                            "Clear All"
                        }

                        // Spacer
                        div { style: "flex: 1;" }

                        // "Done" button
                        button {
                            onclick: on_done_editing,
                            style: "background: var(--fsn-color-primary, #06b6d4); \
                                    color: #fff; \
                                    border: none; border-radius: 6px; \
                                    padding: 7px 18px; \
                                    font-size: 13px; font-family: inherit; \
                                    font-weight: 600; \
                                    cursor: pointer;",
                            "✓ Done"
                        }
                    }
                }
            }
        }
    }
}

// ── HomeWidgetCard ────────────────────────────────────────────────────────────

/// Wraps a widget in a card shell. In edit mode shows a remove button and
/// a "move" cursor as a drag hint.
#[component]
fn HomeWidgetCard(
    slot: WidgetSlot,
    edit_mode: bool,
    on_remove: EventHandler<u32>,
) -> Element {
    let id    = slot.id;
    let kind  = slot.kind.clone();

    let card_style = if edit_mode {
        "position: relative; pointer-events: all; cursor: move;"
    } else {
        "position: relative; pointer-events: all;"
    };

    rsx! {
        div {
            style: "{card_style}",

            // The actual widget
            { render_widget(&kind) }

            // Remove button — only visible in edit mode
            if edit_mode {
                button {
                    onclick: move |_| on_remove.call(id),
                    style: "position: absolute; top: 6px; right: 6px; \
                            width: 22px; height: 22px; \
                            background: rgba(239, 68, 68, 0.85); \
                            color: #fff; \
                            border: none; border-radius: 50%; \
                            font-size: 13px; line-height: 1; \
                            display: flex; align-items: center; justify-content: center; \
                            cursor: pointer; z-index: 10; \
                            padding: 0;",
                    "✕"
                }
            }
        }
    }
}

// ── WidgetPickerRow ───────────────────────────────────────────────────────────

/// A single row in the widget picker panel.
#[component]
fn WidgetPickerRow(kind: WidgetKind, on_add: EventHandler<WidgetKind>) -> Element {
    let icon  = kind.icon();
    let label = kind.label();
    let k     = kind.clone();

    rsx! {
        div {
            onclick: move |_| on_add.call(k.clone()),
            style: "display: flex; align-items: center; gap: 12px; \
                    padding: 9px 16px; \
                    cursor: pointer; \
                    font-size: 13px; \
                    color: var(--fsn-color-text-primary); \
                    transition: background 100ms;",
            onmouseenter: |e: MouseEvent| {
                // Simple hover via JS-less approach — opacity handled by CSS on hover
                let _ = e;
            },
            span { style: "font-size: 18px; min-width: 24px; text-align: center;", "{icon}" }
            span { "{label}" }
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
        "app-tasks" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { TasksApp {} }
            }
        },
        "app-bots" => rsx! {
            AppShell { mode: AppMode::Window,
                BotManagerApp {}
            }
        },
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

/// Map an app id (the part after `"app-"`) to a human-readable breadcrumb label.
fn app_id_to_label(id: &str) -> &str {
    match id {
        "tasks"     => "Tasks",
        "bots"      => "Bots",
        "conductor" => "Conductor",
        "store"     => "Store",
        "studio"    => "Studio",
        "settings"  => "Settings",
        "profile"   => "Profile",
        "ai"        => "AI Assistant",
        "help"      => "Help",
        other       => other,
    }
}
