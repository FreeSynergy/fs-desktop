/// Desktop — root layout: header + sidebar + content area.
use std::collections::HashMap;
use std::sync::Arc;
use dioxus::prelude::*;
use fsn_i18n;

use fsd_browser::BrowserApp;
use fsd_bots::BotManagerApp;
use fsd_container::Container;
use fsd_lenses::LensesApp;
use fsd_managers::{ManagersApp, IconsManagerPanel};
use fsd_profile::ProfileApp;
use fsd_settings::SettingsApp;
use fsd_store::StoreApp;
use fsd_builder::BuilderApp;
use fsd_tasks::TasksApp;
use fsd_theme_mgr::ThemeManagerApp;

use crate::ai_view::AiApp;
use crate::app_shell::{AppMode, AppShell, GLOBAL_CSS, LayoutA, LayoutC};
use fsn_components::FSN_SIDEBAR_CSS;
use crate::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::help_view::HelpApp;
use crate::header::{Breadcrumb, ShellHeader};
use crate::launcher::{AppLauncher, LauncherState};
use crate::notification::{NotificationHistory, NotificationManager, NotificationStack};
use crate::sidebar::{ShellSidebar, default_sidebar_sections};
use crate::taskbar::{AppEntry, default_apps};
use crate::wallpaper::Wallpaper;
use crate::icons::{ICON_EDIT, ICON_ADD, ICON_SETTINGS, ICON_CHEVRON_UP, ICON_CHEVRON_DOWN};
use crate::widgets::{WidgetKind, WidgetSlot, load_widget_layout, render_widget, save_widget_layout};
use crate::window::{Window, WindowId, WindowManager};
use crate::window_frame::{WindowFrame, MinimizedWindowIcon, FSNOBJ_CSS};

/// Eight invisible fixed-position resize handles around the OS window border.
/// Only compiled in desktop mode (requires tao window API).
#[cfg(feature = "desktop")]
fn os_resize_handles() -> Element {
    use dioxus::desktop::tao::window::ResizeDirection;
    let ctx = dioxus::desktop::window();

    macro_rules! handle {
        ($dir:expr, $style:literal, $cursor:literal) => {{
            let c = ctx.clone();
            rsx! {
                div {
                    style: concat!(
                        "position: fixed; z-index: 999999; pointer-events: all; cursor: ",
                        $cursor, "; ", $style
                    ),
                    onmousedown: move |_| { let _ = c.drag_resize_window($dir); },
                }
            }
        }};
    }

    rsx! {
        // Corners
        {handle!(ResizeDirection::NorthWest, "top: 0; left: 0; width: 8px; height: 8px;", "nwse-resize")}
        {handle!(ResizeDirection::NorthEast, "top: 0; right: 0; width: 8px; height: 8px;", "nesw-resize")}
        {handle!(ResizeDirection::SouthWest, "bottom: 0; left: 0; width: 8px; height: 8px;", "nesw-resize")}
        {handle!(ResizeDirection::SouthEast, "bottom: 0; right: 0; width: 8px; height: 8px;", "nwse-resize")}
        // Edges
        {handle!(ResizeDirection::North, "top: 0; left: 8px; right: 8px; height: 4px;", "ns-resize")}
        {handle!(ResizeDirection::South, "bottom: 0; left: 8px; right: 8px; height: 4px;", "ns-resize")}
        {handle!(ResizeDirection::West,  "left: 0; top: 8px; bottom: 8px; width: 4px;", "ew-resize")}
        {handle!(ResizeDirection::East,  "right: 0; top: 8px; bottom: 8px; width: 4px;", "ew-resize")}
    }
}

#[cfg(not(feature = "desktop"))]
fn os_resize_handles() -> Element {
    rsx! {}
}

/// Initialize the global i18n instance and load all app-specific language strings.
///
/// Called once at Desktop startup:
/// 1. Loads built-in generic snippets (EN + DE) from fsn-i18n.
/// 2. Each app crate registers its own app-specific strings (store.*, container.*, …).
/// 3. Overlays any user-installed language pack from disk (Store → Inventory).
fn init_i18n() -> String {
    let lang = fsd_settings::load_active_language();

    // 1. Generic built-in snippets (actions.*, labels.*, status.*, …)
    let _ = fsn_i18n::init_with_builtins(&lang);

    // 2. App-specific strings — each crate owns its own TOML assets
    fsd_store::register_i18n();
    fsd_settings::register_i18n();
    fsd_builder::register_i18n();
    fsd_browser::register_i18n();
    fsd_lenses::register_i18n();
    fsd_managers::register_i18n();
    fsd_container::register_i18n();
    fsd_bots::register_i18n();
    fsd_theme_mgr::register_i18n();
    // shell.* + profile.* — registered inline below
    {
        const EN: &str = include_str!("../assets/i18n/en.toml");
        const DE: &str = include_str!("../assets/i18n/de.toml");
        let _ = fsn_i18n::add_toml_lang("en", EN);
        let _ = fsn_i18n::add_toml_lang("de", DE);
    }

    // 3. Overlay user-installed language pack from ~/.local/share/fsn/i18n/{lang}/ui.toml
    //    (future: will read from Inventory DB instead of files)
    if lang != "en" {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let pack = std::path::PathBuf::from(home)
            .join(".local/share/fsn/i18n")
            .join(&lang)
            .join("ui.toml");
        if let Ok(content) = std::fs::read_to_string(&pack) {
            let _ = fsn_i18n::add_toml_lang(&lang, &content);
        }
    }

    lang
}

/// Root desktop component.
#[component]
pub fn Desktop() -> Element {
    // Pre-register all built-in apps in PackageRegistry (idempotent).
    // Must run before default_sidebar_sections() reads from the registry.
    crate::builtin_apps::ensure_registered();

    // Open all databases once at startup and expose as a shared context.
    // All db operations in this component (and children via crate::db) use this
    // single Arc instead of opening a fresh connection pool per operation.
    let db_ctx = use_context_provider(|| {
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fsd_db::FsdDb::open())
        })
        .expect("Failed to open FreeSynergy databases");
        crate::db::DbContext(Arc::new(db))
    });
    let db = db_ctx.0.clone();

    // Initialize i18n once and expose the active language as a reactive context.
    // fsd-settings writes to this via LangContext when the user switches languages.
    // Reading the signal here subscribes Desktop to re-render on language change,
    // which causes all inline RSX (header, sidebar, taskbar, shell text) to update.
    let lang_ctx = use_context_provider(|| {
        fsd_settings::LangContext(Signal::new(init_i18n()))
    });
    let _active_lang = lang_ctx.0.read().clone();

    // Wallpaper CSS is provided as context so child apps (e.g. AppearanceSettings) can update it.
    let wallpaper_bg: Signal<String> = use_context_provider(|| {
        let saved = crate::db::load_wallpaper_css_from_db(&db);
        Signal::new(if saved.is_empty() { Wallpaper::default().to_css_background() } else { saved })
    });
    // Persist wallpaper whenever it changes.
    {
        let db = db.clone();
        use_effect(move || {
            let css = wallpaper_bg.read().clone();
            crate::db::save_wallpaper_css_to_db(db.clone(), css);
        });
    }

    let mut wm              = use_signal(WindowManager::default);
    let mut apps            = use_signal(default_apps);
    let mut launcher        = use_signal(LauncherState::default);
    let mut notifs          = use_signal(NotificationManager::default);
    let mut notif_history   = use_signal(NotificationHistory::default);
    let mut ctx_menu        = use_signal(|| ContextMenuState::default());
    // Sidebar refresh counter — kept for manual refresh calls from sub-apps.
    let sidebar_refresh: Signal<u32> = use_context_provider(|| Signal::new(0u32));
    let sidebar_sections = use_memo(move || {
        let _ = sidebar_refresh.read(); // subscribe to manual refreshes
        let _ = fsd_store::INSTALL_COUNTER.read(); // subscribe to store install/remove events
        default_sidebar_sections()
    });
    let mut theme: Signal<String> = use_context_provider(|| Signal::new(crate::db::load_theme_from_db(&db)));
    // B5: Animation, chrome opacity, and component style contexts
    let anim_enabled: Signal<bool>    = use_context_provider(|| Signal::new(true));
    let chrome_opacity: Signal<f64>   = use_context_provider(|| Signal::new(0.80f64));
    let chrome_style: Signal<String>  = use_context_provider(|| Signal::new("kde".to_string()));
    let btn_style: Signal<String>     = use_context_provider(|| Signal::new("rounded".to_string()));
    let sidebar_style: Signal<String> = use_context_provider(|| Signal::new("solid".to_string()));
    // Any sub-app can request opening another app by setting this to Some(app_id).
    let mut app_open_req: Signal<Option<String>> = use_context_provider(|| Signal::new(None));

    // Browser URL request: Lenses (and Conductor) can push a URL here to open it in the Browser.
    let _browser_url_req: fsd_browser::app::BrowserUrlRequest =
        use_context_provider(|| Signal::new(None));

    // Handle app-open requests from sub-apps (e.g. Conductor's "Install Service" → Store).
    use_effect(move || {
        let req = app_open_req.read().clone();
        if let Some(app_id) = req {
            open_app(&mut wm, &mut apps, &app_id);
            *app_open_req.write() = None;
        }
    });

    // ── Widget layer state ─────────────────────────────────────────────────
    let db_widgets = db.clone();
    let mut widget_layout   = use_signal(move || load_widget_layout(&db_widgets));
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
    let db_menu = db.clone();
    let menu_action_handler = move |id: String| {
        match id.as_str() {
            "theme-midnight-blue" => { theme.set("midnight-blue".to_string()); crate::db::save_theme_to_db(db_menu.clone(), "midnight-blue".to_string()); }
            "launcher"            => launcher.write().toggle(),
            "open-tasks"          => open_app(&mut wm, &mut apps, "tasks"),
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
    let db_done = db.clone();
    let on_done_editing = move |_: MouseEvent| {
        edit_mode.set(false);
        picker_open.set(false);
        save_widget_layout(db_done.clone(), &widget_layout.read());
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

        // OS-level window resize handles (desktop only, invisible 4-8px borders).
        { os_resize_handles() }

        div {
            id: "fsd-desktop",
            "data-theme":          "{theme_attr}",
            "data-lang":           "{_active_lang}",
            "data-chrome-style":   "{chrome_style.read()}",
            "data-btn-style":      "{btn_style.read()}",
            "data-sidebar-style":  "{sidebar_style.read()}",
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
                                ContextMenuItem::new("edit-desktop", fsn_i18n::t("shell.desktop.edit")).with_icon(ICON_EDIT),
                                ContextMenuItem::new("add-widget",   fsn_i18n::t("shell.desktop.add_widget")).with_icon(ICON_ADD),
                                ContextMenuItem::new("settings",     fsn_i18n::t("shell.desktop.settings")).with_icon(ICON_SETTINGS),
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
                                style: "display: flex; align-items: center; gap: 6px; \
                                        background: var(--fsn-color-bg-surface); \
                                        border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: 8px; \
                                        padding: 6px 14px; \
                                        font-size: 12px; font-family: inherit; \
                                        color: var(--fsn-color-text-muted); \
                                        cursor: pointer; opacity: 0.75; \
                                        transition: opacity 150ms;",
                                span { style: "display: flex; align-items: center;", dangerous_inner_html: ICON_EDIT }
                                {fsn_i18n::t("shell.desktop.edit")}
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
                                    style: "display: flex; align-items: center; gap: 4px; \
                                            background: var(--fsn-color-primary, #06b6d4); \
                                            color: #fff; \
                                            border: none; border-radius: 6px; \
                                            padding: 7px 14px; \
                                            font-size: 13px; font-family: inherit; \
                                            cursor: pointer;",
                                    if is_picker_open {
                                        span { style: "display: flex; align-items: center;", dangerous_inner_html: ICON_CHEVRON_UP }
                                        {fsn_i18n::t("shell.desktop.add_widget")}
                                    } else {
                                        span { style: "display: flex; align-items: center;", dangerous_inner_html: ICON_CHEVRON_DOWN }
                                        {fsn_i18n::t("shell.desktop.add_widget")}
                                    }
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
                                {fsn_i18n::t("shell.desktop.clear_all")}
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
                                {fsn_i18n::t("shell.desktop.done")}
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
                                border: none; border-radius: 50%; \
                                display: flex; align-items: center; justify-content: center; \
                                cursor: pointer; padding: 0;",
                        onmousedown: move |e: MouseEvent| e.stop_propagation(),
                        onclick:     move |_| on_remove.call(id),
                        span { dangerous_inner_html: crate::icons::ICON_CLOSE }
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

/// Returns the SVG icon (or emoji fallback) for a known built-in app ID.
fn icon_for_app(app_id: &str) -> String {
    use crate::icons::{
        ICON_STORE, ICON_SETTINGS, ICON_CONTAINER, ICON_BOTS,
        ICON_LANGUAGE, ICON_THEME, ICON_ICONS, ICON_MANAGERS,
    };
    match app_id {
        "store"            => ICON_STORE.to_string(),
        "settings"         => ICON_SETTINGS.to_string(),
        "container"        => ICON_CONTAINER.to_string(),
        "bot-manager"      => ICON_BOTS.to_string(),
        "language-manager" => ICON_LANGUAGE.to_string(),
        "theme-manager"    => ICON_THEME.to_string(),
        "icons-manager"    => ICON_ICONS.to_string(),
        "managers"         => ICON_MANAGERS.to_string(),
        _                  => "🗗".to_string(),
    }
}

fn open_app(wm: &mut Signal<WindowManager>, apps: &mut Signal<Vec<AppEntry>>, app_id: &str) {
    // Normalize catalog IDs: strip "fsn-" prefix so "fsn-browser" routes as "browser".
    let app_id = app_id.strip_prefix("fsn-").unwrap_or(app_id);

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
    // Use the app entry icon first; fall back to the built-in icon map so every
    // app gets its own icon in the titlebar instead of the generic "🗗" fallback.
    let icon = apps.read().iter().find(|a| a.id == app_id)
        .map(|a| a.icon.clone())
        .unwrap_or_else(|| icon_for_app(app_id));
    let window = Window::new(title_key).with_icon(icon);
    let win_id = window.id;
    wm.write().open(window);

    if let Some(app) = apps.write().iter_mut().find(|a| a.id == app_id) {
        app.windows.push(win_id);
    }
    tracing::info!("Opened app: {}", app_id);
}

/// Wraps each app in the appropriate layout (A / B / C).
/// Apps that manage their own internal sidebar (container-app, theme, bots) use LayoutA
/// so the full area is handed to them without an extra wrapper split.
#[component]
fn AppWindowContent(title_key: String) -> Element {
    match title_key.as_str() {
        "app-tasks" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { TasksApp {} }
            }
        },
        "app-store" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { StoreApp {} }
            }
        },
        "app-builder" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { BuilderApp {} }
            }
        },
        "app-container" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { Container {} }
            }
        },
        "app-language-manager" | "app-manager-language" => rsx! {
            AppShell { mode: AppMode::Window,
                fsd_settings::LanguageSettings {}
            }
        },
        "app-icons-manager" | "app-manager-icons" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { IconsManagerPanel {} }
            }
        },
        "app-theme-manager" | "app-manager-theme" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { ThemeManagerApp {} }
            }
        },
        "app-manager-container-app" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { Container {} }
            }
        },
        "app-bot-manager" | "app-manager-bots" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { BotManagerApp {} }
            }
        },
        // Per-instance bot icons (app-bot-<name>) — all open the BotManager.
        // In Phase P the BotManager will accept a pre-selected bot name via Context.
        t if t.starts_with("app-bot-") => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutA { BotManagerApp {} }
            }
        },
        "app-settings" => rsx! {
            AppShell { mode: AppMode::Window,
                SettingsApp {}
            }
        },
        "app-managers" => rsx! {
            AppShell { mode: AppMode::Window,
                ManagersApp {}
            }
        },
        "app-profile" => rsx! {
            AppShell { mode: AppMode::Window,
                LayoutC { ProfileApp {} }
            }
        },
        "app-browser" => rsx! {
            AppShell { mode: AppMode::Window,
                BrowserApp {}
            }
        },
        "app-lenses" => rsx! {
            AppShell { mode: AppMode::Window,
                LensesApp {}
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
        "tasks"         => "Tasks",
        "store"         => "Store",
        "builder"       => "Builder",
        "browser"       => "Browser",
        "lenses"        => "Lenses",
        "settings"      => "Settings",
        "managers"      => "Managers",
        "profile"       => "Profile",
        "ai"            => "AI Assistant",
        "help"          => "Help",
        "container"           => "Container Manager",
        "theme-manager"       => "Theme Manager",
        "bot-manager"         => "Bot Manager",
        "language-manager"    => "Language Manager",
        "icons-manager"       => "Icons Manager",
        "manager-language"    => "Language Manager",
        "manager-theme"       => "Theme Manager",
        "manager-icons"       => "Icons Manager",
        "manager-container-app" => "Container App Manager",
        "manager-bots"        => "Bots Manager",
        other           => other,
    }
}
