/// Desktop — root layout: header + sidebar + content area.
use std::collections::HashMap;
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
use fsn_components::FSN_SIDEBAR_CSS;
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
use crate::window_frame::{WindowFrame, MinimizedWindowIcon, FSNOBJ_CSS};

/// Root desktop component.
#[component]
pub fn Desktop() -> Element {
    // Wallpaper CSS is provided as context so child apps (e.g. AppearanceSettings) can update it.
    let wallpaper_bg: Signal<String> = use_context_provider(|| Signal::new(Wallpaper::default().to_css_background()));
    let mut wm              = use_signal(WindowManager::default);
    let mut apps            = use_signal(default_apps);
    let mut launcher        = use_signal(LauncherState::default);
    let mut notifs          = use_signal(NotificationManager::default);
    let mut notif_history   = use_signal(NotificationHistory::default);
    let mut ctx_menu        = use_signal(|| ContextMenuState::default());
    let sidebar_sections: Signal<Vec<SidebarSection>> = use_signal(default_sidebar_sections);
    let mut theme: Signal<String> = use_context_provider(|| Signal::new(crate::db::load_theme_from_db()));
    // B5: Animation, chrome opacity, and component style contexts
    let anim_enabled: Signal<bool>    = use_context_provider(|| Signal::new(true));
    let chrome_opacity: Signal<f64>   = use_context_provider(|| Signal::new(0.80f64));
    let chrome_style: Signal<String>  = use_context_provider(|| Signal::new("kde".to_string()));
    let btn_style: Signal<String>     = use_context_provider(|| Signal::new("rounded".to_string()));
    let sidebar_style: Signal<String> = use_context_provider(|| Signal::new("solid".to_string()));

    // ── Widget layer state ─────────────────────────────────────────────────
    let mut widget_layout   = use_signal(load_widget_layout);
    let mut edit_mode       = use_signal(|| false);
    let mut next_widget_id  = use_signal(|| 100u32);
    let mut picker_open     = use_signal(|| false);

    // Persistent icon positions: window_id → (pos_x, pos_y)
    let mut icon_positions: Signal<HashMap<u64, (f64, f64)>> = use_signal(HashMap::new);

    let bg = wallpaper_bg.read().clone();

    // B5: Build dynamic CSS overrides for animation + chrome opacity.
    let anim_dur = if *anim_enabled.read() { "180ms" } else { "0ms" };
    let win_opacity = *chrome_opacity.read();
    let dynamic_css = format!(
        ":root {{ --fsn-anim-duration: {anim_dur}; --fsn-window-bg: rgba(15,23,42,{win_opacity:.2}); }}"
    );

    // Separate custom-injected CSS (Store themes) from named theme attribute.
    // Convention: theme Signal = "__custom__<css>" for Store themes, plain id for built-in.
    let theme_val = theme.read().clone();
    let (theme_attr, store_theme_css) = if let Some(css) = theme_val.strip_prefix("__custom__") {
        ("".to_string(), css.to_string())
    } else {
        (theme_val, String::new())
    };

    // ── Theme + menu action handler ────────────────────────────────────────
    let menu_action_handler = move |id: String| {
        match id.as_str() {
            "theme-midnight-blue" => { theme.set("midnight-blue".to_string()); crate::db::save_theme_to_db("midnight-blue".to_string()); }
            "launcher"            => launcher.write().toggle(),
            "open-tasks"          => open_app(&mut wm, &mut apps, "tasks"),
            "open-bots"           => open_app(&mut wm, &mut apps, "bots"),
            _ => {}
        }
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
    let in_edit_mode   = *edit_mode.read();
    let is_picker_open = *picker_open.read();
    // In edit mode the window area is hidden (visibility: hidden preserves component state).
    let window_area_visibility = if in_edit_mode { "visibility: hidden;" } else { "" };

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

    // Pre-compute icon positions for minimized windows.
    // Algorithm: fill grid slots left→right; when a row is full, go one row up.
    // Windows that already have a stored drag position keep that position.
    let effective_icon_positions: HashMap<u64, (f64, f64)> = {
        const ICON_W: f64 = 88.0;
        const ICON_H: f64 = 84.0;
        const START_X: f64 = 20.0;
        const START_Y: f64 = 600.0;
        const MAX_COLS: usize = 14; // ~14 × 88 px ≈ 1232 px — fits any HD+ screen

        let stored = icon_positions.read().clone();
        let mut used: Vec<(f64, f64)> = stored.values().cloned().collect();
        let mut result = stored.clone();

        for window in wm.read().windows().iter().filter(|w| w.minimized) {
            if stored.contains_key(&window.id.0) {
                continue; // already has a user-dragged position
            }
            // Find first free slot: right → up
            let pos = 'find: {
                for row in 0usize.. {
                    let y = START_Y - row as f64 * ICON_H;
                    for col in 0..MAX_COLS {
                        let x = START_X + col as f64 * ICON_W;
                        let free = !used.iter().any(|(ux, uy)| {
                            (ux - x).abs() < ICON_W * 0.8 && (uy - y).abs() < ICON_H * 0.8
                        });
                        if free { break 'find (x, y); }
                    }
                }
                (START_X, START_Y)
            };
            used.push(pos);
            result.insert(window.id.0, pos);
        }
        result
    };

    rsx! {
        style { "{GLOBAL_CSS}" }
        style { "{FSNOBJ_CSS}" }
        style { "{FSN_SIDEBAR_CSS}" }
        style { "{dynamic_css}" }
        // Store-theme CSS injection (overrides the built-in midnight-blue defaults)
        if !store_theme_css.is_empty() {
            style { "{store_theme_css}" }
        }

        div {
            id: "fsd-desktop",
            "data-theme": "{theme_attr}",
            "data-chrome-style":  "{chrome_style.read()}",
            "data-btn-style":     "{btn_style.read()}",
            "data-sidebar-style": "{sidebar_style.read()}",
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

            // ── Content area: sidebar (flow) + desktop area ────────────────
            div {
                style: "flex: 1; display: flex; flex-direction: row; overflow: hidden;",

                // ── Shell sidebar (flow, always visible, hover-expand) ──────
                ShellSidebar {
                    sections: sidebar_sections.read().clone(),
                    active_id: active_app_id,
                    on_select: on_sidebar_select,
                }

                // ── Desktop area (home layer + window area) ─────────────────
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

                    // ── Home layer — widgets sit on the desktop background ──
                    div {
                        id: "fsd-home-layer",
                        style: "position: absolute; inset: 0; overflow: hidden; pointer-events: none;",

                        for slot in widget_layout.read().clone().into_iter() {
                            HomeWidgetCard {
                                key: "{slot.id}",
                                slot: slot.clone(),
                                edit_mode: in_edit_mode,
                                on_remove: move |id: u32| {
                                    widget_layout.write().retain(|s| s.id != id);
                                },
                                on_update: move |updated: WidgetSlot| {
                                    if let Some(s) = widget_layout.write().iter_mut().find(|s| s.id == updated.id) {
                                        *s = updated;
                                    }
                                },
                            }
                        }
                    }

                    // ── Window area — pointer-events: none so widgets are reachable.
                    // Hidden (visibility: hidden) in edit mode to let the user work on widgets only.
                    div {
                        id: "fsd-window-area",
                        style: "position: absolute; inset: 0; overflow: hidden; pointer-events: none; {window_area_visibility}",

                        // Render visible (non-minimized) windows
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

                        // Render minimized windows as desktop icons.
                        // Key pattern "min-{id}" avoids collisions with visible-window keys.
                        // effective_icon_positions (pre-computed above) assigns grid slots
                        // right → up for windows without a stored drag position.
                        for window in wm.read().windows().iter().filter(|w| w.minimized).cloned().collect::<Vec<_>>() {
                            MinimizedWindowIcon {
                                key: "min-{window.id.0}",
                                window: window.clone(),
                                pos_x: effective_icon_positions.get(&window.id.0).map(|p| p.0).unwrap_or(20.0),
                                pos_y: effective_icon_positions.get(&window.id.0).map(|p| p.1).unwrap_or(600.0),
                                on_restore: on_focus_window,
                                on_move: {
                                    let wid = window.id.0;
                                    move |(nx, ny): (f64, f64)| {
                                        icon_positions.write().insert(wid, (nx, ny));
                                    }
                                },
                            }
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

                    // ── Edit Desktop button (bottom-right, outside edit mode) ──
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

                    // ── Edit Mode toolbar (bottom bar) ─────────────────────────
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
                                                max-height: 320px; \
                                                overflow-y: auto; \
                                                z-index: 70; \
                                                box-shadow: 0 8px 24px rgba(0,0,0,0.4);",

                                        for kind in WidgetKind::all_with_custom() {
                                            WidgetPickerRow {
                                                kind: kind.clone(),
                                                on_add: move |k: WidgetKind| {
                                                    let id = *next_widget_id.read();
                                                    next_widget_id.set(id + 1);
                                                    let (w, h) = k.default_size();
                                                    let count = widget_layout.read().len();
                                                    let x = 24.0 + (count as f64 % 3.0) * (w + 16.0);
                                                    let y = 24.0 + (count as f64 / 3.0).floor() * (h + 16.0);
                                                    widget_layout.write().push(WidgetSlot { id, kind: k, x, y, w, h });
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
                } // end desktop area
            } // end flex row (sidebar + desktop)
        }
    }
}

// ── HomeWidgetCard ────────────────────────────────────────────────────────────

/// Wraps a widget in a card shell with drag and resize support in edit mode.
#[component]
fn HomeWidgetCard(
    slot: WidgetSlot,
    edit_mode: bool,
    on_remove: EventHandler<u32>,
    on_update: EventHandler<WidgetSlot>,
) -> Element {
    let id   = slot.id;
    let kind = slot.kind.clone();

    // Local position / size — initialised from slot on mount, updated on drag/resize
    let mut pos_x  = use_signal(|| slot.x);
    let mut pos_y  = use_signal(|| slot.y);
    let mut width  = use_signal(|| slot.w);
    let mut height = use_signal(|| slot.h);

    // Drag state
    let mut dragging = use_signal(|| false);
    let mut drag_ox  = use_signal(|| 0.0f64);
    let mut drag_oy  = use_signal(|| 0.0f64);

    // Resize state
    let mut resizing  = use_signal(|| false);
    let mut resize_sx = use_signal(|| 0.0f64);
    let mut resize_sy = use_signal(|| 0.0f64);
    let mut resize_sw = use_signal(|| 0.0f64);
    let mut resize_sh = use_signal(|| 0.0f64);

    let x = *pos_x.read();
    let y = *pos_y.read();
    let w = *width.read();
    let h = *height.read();
    let is_dragging  = *dragging.read();
    let is_resizing  = *resizing.read();

    // Clones for closures
    let kind_render   = kind.clone();
    let kind_label    = kind.label();
    let kind_drag_up  = kind.clone();
    let kind_resize_up = kind.clone();

    let card_style = format!(
        "position: absolute; left: {x}px; top: {y}px; width: {w}px; height: {h}px; \
         display: flex; flex-direction: column; overflow: hidden; \
         pointer-events: all; user-select: none;"
    );

    rsx! {
        div { style: "{card_style}",

            // Drag handle — only in edit mode
            if edit_mode {
                div {
                    style: "height: 26px; \
                            background: var(--fsn-color-bg-elevated, #1e2d45); \
                            border-radius: 8px 8px 0 0; \
                            display: flex; align-items: center; \
                            padding: 0 8px; gap: 6px; \
                            cursor: grab; border-bottom: 1px solid var(--fsn-color-border-default);",
                    onmousedown: move |e: MouseEvent| {
                        let coords = e.client_coordinates();
                        drag_ox.set(coords.x - *pos_x.read());
                        drag_oy.set(coords.y - *pos_y.read());
                        dragging.set(true);
                    },
                    span {
                        style: "font-size: 11px; color: var(--fsn-color-text-muted); \
                                flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        "⠿ {kind_label}"
                    }
                    button {
                        style: "width: 18px; height: 18px; flex-shrink: 0; \
                                background: rgba(239,68,68,0.85); color: #fff; \
                                border: none; border-radius: 50%; font-size: 11px; line-height: 1; \
                                display: flex; align-items: center; justify-content: center; \
                                cursor: pointer; padding: 0;",
                        onmousedown: move |e: MouseEvent| e.stop_propagation(),
                        onclick:     move |_| on_remove.call(id),
                        "✕"
                    }
                }
            }

            // Widget content — pass slot dimensions so widgets can scale their content.
            { render_widget(&kind_render, w, h) }

            // Resize handle (bottom-right corner) — only in edit mode
            if edit_mode {
                div {
                    style: "position: absolute; bottom: 0; right: 0; \
                            width: 20px; height: 20px; cursor: nwse-resize; \
                            display: flex; align-items: center; justify-content: center; \
                            font-size: 11px; color: var(--fsn-color-text-muted); \
                            opacity: 0.7;",
                    onmousedown: move |e: MouseEvent| {
                        e.stop_propagation();
                        let coords = e.client_coordinates();
                        resize_sx.set(coords.x);
                        resize_sy.set(coords.y);
                        resize_sw.set(*width.read());
                        resize_sh.set(*height.read());
                        resizing.set(true);
                    },
                    "◢"
                }
            }
        }

        // Full-screen overlay that captures mouse events while dragging
        if is_dragging {
            div {
                style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; \
                        z-index: 9999; pointer-events: all; cursor: grabbing;",
                onmousemove: move |e: MouseEvent| {
                    let coords = e.client_coordinates();
                    pos_x.set(coords.x - *drag_ox.read());
                    pos_y.set(coords.y - *drag_oy.read());
                },
                onmouseup: move |_| {
                    dragging.set(false);
                    on_update.call(WidgetSlot {
                        id, kind: kind_drag_up.clone(),
                        x: *pos_x.read(), y: *pos_y.read(),
                        w: *width.read(),  h: *height.read(),
                    });
                },
            }
        }

        // Full-screen overlay that captures mouse events while resizing
        if is_resizing {
            div {
                style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; \
                        z-index: 9999; pointer-events: all; cursor: nwse-resize;",
                onmousemove: move |e: MouseEvent| {
                    let coords = e.client_coordinates();
                    let dx = coords.x - *resize_sx.read();
                    let dy = coords.y - *resize_sy.read();
                    width.set((*resize_sw.read() + dx).max(150.0));
                    height.set((*resize_sh.read() + dy).max(80.0));
                },
                onmouseup: move |_| {
                    resizing.set(false);
                    on_update.call(WidgetSlot {
                        id, kind: kind_resize_up.clone(),
                        x: *pos_x.read(), y: *pos_y.read(),
                        w: *width.read(),  h: *height.read(),
                    });
                },
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
    let icon = apps.read().iter().find(|a| a.id == app_id)
        .map(|a| a.icon.clone())
        .unwrap_or_else(|| "🗗".to_string());
    let window = Window::new(title_key).with_icon(icon);
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
