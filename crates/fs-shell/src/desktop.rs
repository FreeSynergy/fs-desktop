/// Desktop — root layout: header + sidebar + content area.
use std::collections::HashMap;
use std::sync::Arc;
use dioxus::prelude::*;
use fs_i18n;

use fs_browser::BrowserApp;
use fs_bots::BotManagerApp;
use fs_container_app::Container;
use fs_lenses::LensesApp;
use fs_managers::{ManagersApp, IconsManagerPanel};
use fs_profile::ProfileApp;
use fs_settings::SettingsApp;
use fs_store_app::StoreApp;
use fs_builder::BuilderApp;
use fs_tasks::TasksApp;
use fs_theme_app::ThemeManagerApp;

use fs_components::AppContext;

use crate::ai_view::AiApp;
use crate::app_shell::{AppMode, AppShell, GLOBAL_CSS, LayoutA, LayoutC};
use fs_components::FS_SIDEBAR_CSS;
use crate::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::help_view::{HelpApp, HelpSidebarPanel};
use crate::header::{Breadcrumb, ShellHeader};
use crate::launcher::{AppLauncher, LauncherState};
use crate::notification::{NotificationHistory, NotificationManager, NotificationStack};
use crate::sidebar::{ShellSidebar, default_sidebar_sections};
use crate::taskbar::{AppEntry, default_apps};
use fs_db_desktop::package_registry::PackageRegistry;
use crate::wallpaper::Wallpaper;
use crate::icons::{ICON_EDIT, ICON_ADD, ICON_SETTINGS, ICON_CHEVRON_UP, ICON_CHEVRON_DOWN};
use crate::widgets::{WidgetKind, WidgetSlot, load_widget_layout, render_widget, save_widget_layout};
use crate::window::{OpenWindow, Window, WindowId, WindowManager, WindowRenderFn};
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
/// 1. Loads built-in generic snippets (EN + DE) from fs-i18n.
/// 2. Each app crate registers its own app-specific strings (store.*, container.*, …).
/// 3. Overlays any user-installed language pack from disk (Store → Inventory).
fn init_i18n() -> String {
    let lang = fs_settings::load_active_language();

    // 1. Generic built-in snippets (actions.*, labels.*, status.*, …)
    let _ = fs_i18n::init_with_builtins(&lang);

    // 2. App-specific strings — each crate owns its own TOML assets
    fs_store_app::register_i18n();
    fs_settings::register_i18n();
    fs_builder::register_i18n();
    fs_browser::register_i18n();
    fs_lenses::register_i18n();
    fs_managers::register_i18n();
    fs_container_app::register_i18n();
    fs_bots::register_i18n();
    fs_theme_app::register_i18n();
    // shell.* + profile.* — registered inline below
    {
        const EN: &str = include_str!("../assets/i18n/en.toml");
        const DE: &str = include_str!("../assets/i18n/de.toml");
        let _ = fs_i18n::add_toml_lang("en", EN);
        let _ = fs_i18n::add_toml_lang("de", DE);
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
            let _ = fs_i18n::add_toml_lang(&lang, &content);
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
            tokio::runtime::Handle::current().block_on(fs_db_desktop::FsdDb::open())
        })
        .expect("Failed to open FreeSynergy databases");
        crate::db::DbContext(Arc::new(db))
    });
    let db = db_ctx.0.clone();

    // Single AppContext — all cross-cutting desktop state in one place.
    // Child components access locale, theme, wallpaper, and appearance settings
    // via use_context::<AppContext>() instead of individual signals.
    let app_ctx = use_context_provider(|| {
        let saved_wallpaper = crate::db::load_wallpaper_css_from_db(&db);
        AppContext {
            locale:         Signal::new(init_i18n()),
            theme:          Signal::new(crate::db::load_theme_from_db(&db)),
            wallpaper:      Signal::new(if saved_wallpaper.is_empty() {
                                Wallpaper::default().to_css_background()
                            } else {
                                saved_wallpaper
                            }),
            anim_enabled:   Signal::new(true),
            chrome_opacity: Signal::new(0.80f64),
            chrome_style:   Signal::new("kde".to_string()),
            btn_style:      Signal::new("rounded".to_string()),
            sidebar_style:  Signal::new("solid".to_string()),
            app_open_req:   Signal::new(None),
        }
    });
    let _active_lang = app_ctx.locale.read().clone();

    // Persist wallpaper whenever it changes.
    {
        let db = db.clone();
        use_effect(move || {
            let css = app_ctx.wallpaper.read().clone();
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
        let _ = fs_store_app::INSTALL_COUNTER.read(); // subscribe to store install/remove events
        default_sidebar_sections()
    });
    // Convenience locals extracted from AppContext for use in this component's closures.
    let mut theme          = app_ctx.theme;
    let anim_enabled       = app_ctx.anim_enabled;
    let chrome_opacity     = app_ctx.chrome_opacity;
    let chrome_style       = app_ctx.chrome_style;
    let btn_style          = app_ctx.btn_style;
    let sidebar_style      = app_ctx.sidebar_style;
    let mut app_open_req   = app_ctx.app_open_req;

    // Notification callbacks for AI health changes (used by HelpSidebarPanel).
    // Declared here so they can capture the `notifs` signal.

    // Browser URL request: Lenses (and Conductor) can push a URL here to open it in the Browser.
    let _browser_url_req: fs_browser::app::BrowserUrlRequest =
        use_context_provider(|| Signal::new(None));

    // ── Virtual desktops ───────────────────────────────────────────────────
    let mut active_desktop: Signal<usize> = use_signal(|| 0usize);
    let desktop_count: usize = 2; // configurable in settings; 2 = default per spec
    // Wipe animation: direction + monotonic key so each switch triggers a fresh animation.
    // "left"  = new tab has higher index (slide new content in from the right)
    // "right" = new tab has lower index  (slide new content in from the left)
    let mut slide_dir: Signal<&'static str> = use_signal(|| "");
    let mut slide_anim_key: Signal<u32> = use_signal(|| 0u32);

    // Handle app-open requests from sub-apps (e.g. Conductor's "Install Service" → Store).
    use_effect(move || {
        let req = app_open_req.read().clone();
        if let Some(app_id) = req {
            open_app(&mut wm, &mut apps, &app_id, *active_desktop.read());
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

    let bg = app_ctx.wallpaper.read().clone();

    // B5: Build dynamic CSS overrides for animation + chrome opacity.
    let anim_dur = if *anim_enabled.read() { "180ms" } else { "0ms" };
    let win_opacity = *chrome_opacity.read();
    let dynamic_css = format!(
        ":root {{ --fs-anim-duration: {anim_dur}; --fs-window-bg: rgba(15,23,42,{win_opacity:.2}); }}"
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
            "open-tasks"          => open_app(&mut wm, &mut apps, "tasks", *active_desktop.read()),
            _ => {}
        }
    };

    // ── Sidebar app select ─────────────────────────────────────────────────
    let on_sidebar_select = move |app_id: String| {
        open_app(&mut wm, &mut apps, &app_id, *active_desktop.read());
        launcher.write().close();
    };

    // ── Launcher callbacks ──────────────────────────────────────────────────
    let on_launcher_launch = move |app_id: String| {
        open_app(&mut wm, &mut apps, &app_id, *active_desktop.read());
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
        style { "{FS_SIDEBAR_CSS}" }
        style { "{dynamic_css}" }
        // Store-theme CSS injection (overrides the built-in midnight-blue defaults)
        if !store_theme_css.is_empty() {
            style { "{store_theme_css}" }
        }

        // OS-level window resize handles (desktop only, invisible 4-8px borders).
        { os_resize_handles() }

        div {
            id: "fs-desktop",
            "data-theme":          "{theme_attr}",
            "data-lang":           "{_active_lang}",
            "data-chrome-style":   "{chrome_style.read()}",
            "data-btn-style":      "{btn_style.read()}",
            "data-sidebar-style":  "{sidebar_style.read()}",
            style: "
                width: 100vw; height: 100vh; overflow: hidden;
                display: flex; flex-direction: column;
                background: var(--fs-bg-base);
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

                // ── Desktop area: tab bar + (home layer + window area) ──────
                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                    // Virtual desktop tab bar (top of desktop area)
                    DesktopTabBar {
                        count: desktop_count,
                        active: *active_desktop.read(),
                        on_switch: move |idx: usize| {
                            let cur = *active_desktop.read();
                            if idx != cur {
                                slide_dir.set(if idx > cur { "left" } else { "right" });
                                let next_key = *slide_anim_key.read() + 1;
                                slide_anim_key.set(next_key);
                                active_desktop.set(idx);
                            }
                        },
                    }

                    {
                        const WIPE_CSS: &str = r#"
@keyframes fsnWipeFromRight {
    0%   { transform: translateX(100%); }
    40%  { transform: translateX(0); }
    60%  { transform: translateX(0); }
    100% { transform: translateX(-100%); }
}
@keyframes fsnWipeFromLeft {
    0%   { transform: translateX(-100%); }
    40%  { transform: translateX(0); }
    60%  { transform: translateX(0); }
    100% { transform: translateX(100%); }
}
.fs-wipe-overlay {
    position: absolute; inset: 0; z-index: 9998; pointer-events: none;
    background: var(--fs-bg-base);
}
.fs-wipe-overlay--left  { animation: fsnWipeFromRight 320ms cubic-bezier(0.4,0,0.2,1) forwards; }
.fs-wipe-overlay--right { animation: fsnWipeFromLeft  320ms cubic-bezier(0.4,0,0.2,1) forwards; }
@media (prefers-reduced-motion: reduce) {
    .fs-wipe-overlay--left, .fs-wipe-overlay--right { animation: none; display: none; }
}
"#;
                        rsx! { style { "{WIPE_CSS}" } }
                    }

                    div {
                    style: "flex: 1; position: relative; overflow: hidden;",
                    oncontextmenu: move |e: MouseEvent| {
                        e.prevent_default();
                        let coords = e.client_coordinates();
                        ctx_menu.set(ContextMenuState::open_at(
                            coords.x,
                            coords.y,
                            vec![
                                ContextMenuItem::new("edit-desktop", fs_i18n::t("shell.desktop.edit")).with_icon(ICON_EDIT),
                                ContextMenuItem::new("add-widget",   fs_i18n::t("shell.desktop.add_widget")).with_icon(ICON_ADD),
                                ContextMenuItem::new("settings",     fs_i18n::t("shell.desktop.settings")).with_icon(ICON_SETTINGS),
                            ],
                        ));
                    },

                    // ── Home layer — widgets sit on the desktop background ──
                    div {
                        id: "fs-home-layer",
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
                        id: "fs-window-area",
                        style: "position: absolute; inset: 0; overflow: hidden; pointer-events: none; {window_area_visibility}",

                        // Render ALL non-minimized windows — use display:none for inactive
                        // desktops so Dioxus keeps component state (pos/dim signals) alive.
                        for window in wm.read().windows().iter().filter(|w| !w.minimized).cloned().collect::<Vec<_>>() {
                            {
                                let on_active = window.desktop_index == *active_desktop.read();
                                let visibility = if on_active { "" } else { "display: none;" };
                                rsx! {
                                    div {
                                        key: "wrap-{window.id.0}",
                                        style: "{visibility}",
                                        WindowFrame {
                                            key: "{window.id.0}",
                                            window: window.clone(),
                                            on_close: on_close_window,
                                            on_focus: on_focus_window,
                                            on_minimize: on_minimize_window,
                                            on_maximize: on_maximize_window,
                                        }
                                    }
                                }
                            }
                        }

                        // Minimized icons: show only those on the active desktop.
                        for window in wm.read().windows().iter()
                            .filter(|w| w.minimized && w.desktop_index == *active_desktop.read())
                            .cloned().collect::<Vec<_>>()
                        {
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
                                "settings"     => open_app(&mut wm, &mut apps, "settings", *active_desktop.read()),
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
                                        background: var(--fs-color-bg-surface); \
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: 8px; \
                                        padding: 6px 14px; \
                                        font-size: 12px; font-family: inherit; \
                                        color: var(--fs-color-text-muted); \
                                        cursor: pointer; opacity: 0.75; \
                                        transition: opacity 150ms;",
                                span { style: "display: flex; align-items: center;", dangerous_inner_html: ICON_EDIT }
                                {fs_i18n::t("shell.desktop.edit")}
                            }
                        }
                    }

                    // ── Edit Mode toolbar (bottom bar) ─────────────────────────
                    if in_edit_mode {
                        div {
                            id: "fs-edit-toolbar",
                            style: "position: absolute; bottom: 0; left: 0; right: 0; z-index: 60; \
                                    background: var(--fs-color-bg-surface); \
                                    border-top: 1px solid var(--fs-color-border-default); \
                                    padding: 10px 20px; \
                                    display: flex; align-items: center; gap: 10px;",

                            // "+ Add Widget" with picker panel
                            div { style: "position: relative;",
                                button {
                                    onclick: on_toggle_picker,
                                    style: "display: flex; align-items: center; gap: 4px; \
                                            background: var(--fs-color-primary, #06b6d4); \
                                            color: #fff; \
                                            border: none; border-radius: 6px; \
                                            padding: 7px 14px; \
                                            font-size: 13px; font-family: inherit; \
                                            cursor: pointer;",
                                    if is_picker_open {
                                        span { style: "display: flex; align-items: center;", dangerous_inner_html: ICON_CHEVRON_UP }
                                        {fs_i18n::t("shell.desktop.add_widget")}
                                    } else {
                                        span { style: "display: flex; align-items: center;", dangerous_inner_html: ICON_CHEVRON_DOWN }
                                        {fs_i18n::t("shell.desktop.add_widget")}
                                    }
                                }

                                // Widget picker panel (floats above toolbar)
                                if is_picker_open {
                                    div {
                                        id: "fs-widget-picker",
                                        style: "position: absolute; bottom: calc(100% + 8px); left: 0; \
                                                background: var(--fs-color-bg-surface); \
                                                border: 1px solid var(--fs-color-border-default); \
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
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: 6px; \
                                        padding: 7px 14px; \
                                        font-size: 13px; font-family: inherit; \
                                        color: var(--fs-color-text-muted); \
                                        cursor: pointer;",
                                {fs_i18n::t("shell.desktop.clear_all")}
                            }

                            // Spacer
                            div { style: "flex: 1;" }

                            // "Done" button
                            button {
                                onclick: on_done_editing,
                                style: "background: var(--fs-color-primary, #06b6d4); \
                                        color: #fff; \
                                        border: none; border-radius: 6px; \
                                        padding: 7px 18px; \
                                        font-size: 13px; font-family: inherit; \
                                        font-weight: 600; \
                                        cursor: pointer;",
                                {fs_i18n::t("shell.desktop.done")}
                            }
                        }
                    }
                    // ── Wipe animation overlay ─────────────────────────────
                    // Keyed by switch counter so each tab-switch triggers a fresh animation.
                    {
                        let dir = *slide_dir.read();
                        let key = *slide_anim_key.read();
                        if !dir.is_empty() {
                            let cls = if dir == "left" {
                                "fs-wipe-overlay fs-wipe-overlay--left"
                            } else {
                                "fs-wipe-overlay fs-wipe-overlay--right"
                            };
                            rsx! {
                                div { key: "{key}", class: "{cls}" }
                            }
                        } else {
                            rsx! {}
                        }
                    }

                    } // end inner relative desktop area
                } // end desktop column (tab bar + inner)

                // ── Help sidebar (right, hover-expandable) ─────────────────
                HelpSidebarPanel {
                    on_ai_offline: Some(EventHandler::new(move |_| {
                        notifs.write().push(
                            crate::notification::NotificationKind::Warning,
                            "AI Assistant offline",
                            Some("The AI service is not responding.".into()),
                        );
                    })),
                    on_ai_online: Some(EventHandler::new(move |_| {
                        notifs.write().push(
                            crate::notification::NotificationKind::Success,
                            "AI Assistant online",
                            Some("The AI assistant is now available.".into()),
                        );
                    })),
                }

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
                            background: var(--fs-color-bg-elevated, #1e2d45); \
                            border-radius: 8px 8px 0 0; \
                            display: flex; align-items: center; \
                            padding: 0 8px; gap: 6px; \
                            cursor: grab; border-bottom: 1px solid var(--fs-color-border-default);",
                    onmousedown: move |e: MouseEvent| {
                        let coords = e.client_coordinates();
                        drag_ox.set(coords.x - *pos_x.read());
                        drag_oy.set(coords.y - *pos_y.read());
                        dragging.set(true);
                    },
                    span {
                        style: "font-size: 11px; color: var(--fs-color-text-muted); \
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
                            font-size: 11px; color: var(--fs-color-text-muted); \
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
                    color: var(--fs-color-text-primary); \
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

fn open_app(wm: &mut Signal<WindowManager>, apps: &mut Signal<Vec<AppEntry>>, app_id: &str, desktop_idx: usize) {
    // Normalize catalog IDs: strip "fs-" prefix so "fs-browser" routes as "browser".
    let app_id = app_id.strip_prefix("fs-").unwrap_or(app_id);

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

    // Icon priority: existing AppEntry → PackageRegistry (project icon) → built-in map → fallback.
    // This ensures the window titlebar always shows the same icon as the left sidebar.
    let icon = apps.read().iter().find(|a| a.id == app_id)
        .map(|a| a.icon.clone())
        .or_else(|| {
            PackageRegistry::load()
                .into_iter()
                .find(|p| p.id == app_id)
                .map(|p| p.icon)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| icon_for_app(app_id));

    let meta   = Window::new(title_key).with_icon(icon.clone()).with_desktop(desktop_idx);
    let win_id = meta.id;
    let render = render_fn_for(app_id);
    wm.write().open(OpenWindow::new(meta, render));

    // Ensure every opened app has an AppEntry so the taskbar can show it.
    let app_exists = apps.read().iter().any(|a| a.id == app_id);
    if app_exists {
        if let Some(app) = apps.write().iter_mut().find(|a| a.id == app_id) {
            app.windows.push(win_id);
        }
    } else {
        apps.write().push(AppEntry {
            id:        app_id.to_string(),
            label_key: app_id_to_label(app_id).to_string(),
            icon,
            icon_url:  None,
            group:     None,
            pinned:    false,
            windows:   vec![win_id],
        });
    }
    tracing::info!("Opened app: {}", app_id);
}

/// Returns the `WindowRenderFn` for `app_id` (the part *after* the `"fs-"` prefix,
/// e.g. `"store"`, `"tasks"`, `"bot-manager"`).
///
/// Each function is a zero-arg Dioxus component that owns its full layout — the
/// same component that would run in a standalone OS window. This replaces the old
/// `AppWindowContent` match block: the render fn is stored once in `OpenWindow`
/// when the window is opened, so dispatch happens at open-time, not on every render.
fn render_fn_for(app_id: &str) -> WindowRenderFn {
    // Per-instance bot icons all open the BotManager view (Phase P: pre-select via Context).
    if app_id.starts_with("bot-") && app_id != "bot-manager" {
        return render_bot_manager;
    }
    match app_id {
        "tasks"                                       => render_tasks,
        "store"                                       => render_store,
        "builder"                                     => render_builder,
        "container" | "manager-container-app"         => render_container,
        "language-manager" | "manager-language"       => render_language_manager,
        "icons-manager"    | "manager-icons"          => render_icons_manager,
        "theme-manager"    | "manager-theme"          => render_theme_manager,
        "bot-manager"      | "manager-bots"           => render_bot_manager,
        "settings"                                    => render_settings,
        "managers"                                    => render_managers,
        "profile"                                     => render_profile,
        "browser"                                     => render_browser,
        "lenses"                                      => render_lenses,
        "ai"                                          => render_ai,
        "help"                                        => render_help,
        _                                             => render_unknown,
    }
}

fn render_tasks()            -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { TasksApp {} } } } }
fn render_store()            -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { StoreApp {} } } } }
fn render_builder()          -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { BuilderApp {} } } } }
fn render_container()        -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { Container {} } } } }
fn render_language_manager() -> Element { rsx! { AppShell { mode: AppMode::Window, fs_settings::LanguageSettings {} } } }
fn render_icons_manager()    -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { IconsManagerPanel {} } } } }
fn render_theme_manager()    -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { ThemeManagerApp {} } } } }
fn render_bot_manager()      -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { BotManagerApp {} } } } }
fn render_settings()         -> Element { rsx! { AppShell { mode: AppMode::Window, SettingsApp {} } } }
fn render_managers()         -> Element { rsx! { AppShell { mode: AppMode::Window, ManagersApp {} } } }
fn render_profile()          -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutC { ProfileApp {} } } } }
fn render_browser()          -> Element { rsx! { AppShell { mode: AppMode::Window, BrowserApp {} } } }
fn render_lenses()           -> Element { rsx! { AppShell { mode: AppMode::Window, LensesApp {} } } }
fn render_ai()               -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { AiApp {} } } } }
fn render_help()             -> Element { rsx! { AppShell { mode: AppMode::Window, LayoutA { HelpApp {} } } } }
fn render_unknown()          -> Element {
    rsx! {
        div {
            style: "color: var(--fs-color-text-muted, #94a3b8); font-size: 13px; \
                    display: flex; align-items: center; justify-content: center; height: 200px;",
            "Unknown app"
        }
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

// ── DesktopTabBar ─────────────────────────────────────────────────────────────

/// Tab bar at the top of the desktop area for switching virtual desktops.
///
/// Per spec konzepte/ui-standards.md:
/// - Tab right (index increases) → current slides LEFT out, new slides from RIGHT
/// - Tab left (index decreases)  → current slides RIGHT out, new slides from LEFT
/// - Default: 2 virtual desktops (configurable in Settings)
#[component]
fn DesktopTabBar(count: usize, active: usize, on_switch: EventHandler<usize>) -> Element {
    // Track previous active index to determine slide direction for animation.
    let mut prev = use_signal(|| active);

    // CSS for the tab bar + slide animation.
    const TAB_BAR_CSS: &str = r#"
.fs-desktop-tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--fs-color-bg-sidebar, #0a0f1a);
    border-bottom: 1px solid var(--fs-color-border-default, #1e293b);
    flex-shrink: 0;
    height: 32px;
}
.fs-desktop-tab {
    padding: 2px 16px;
    border-radius: 6px 6px 0 0;
    border: none;
    background: transparent;
    color: var(--fs-color-text-muted, #64748b);
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    transition: background 120ms, color 120ms;
    white-space: nowrap;
}
.fs-desktop-tab:hover {
    background: var(--fs-color-bg-elevated, #1e293b);
    color: var(--fs-color-text-secondary, #94a3b8);
}
.fs-desktop-tab--active {
    background: var(--fs-color-bg-elevated, #1e293b);
    color: var(--fs-color-primary, #06b6d4);
    font-weight: 600;
    border-bottom: 2px solid var(--fs-color-primary, #06b6d4);
}
"#;

    rsx! {
        style { "{TAB_BAR_CSS}" }
        div {
            class: "fs-desktop-tabs",
            for i in 0..count {
                {
                    let is_active = i == active;
                    let tab_class = if is_active {
                        "fs-desktop-tab fs-desktop-tab--active"
                    } else {
                        "fs-desktop-tab"
                    };
                    rsx! {
                        button {
                            key: "{i}",
                            class: "{tab_class}",
                            onclick: move |_| {
                                prev.set(active);
                                on_switch.call(i);
                            },
                            "Desktop {i + 1}"
                        }
                    }
                }
            }
        }
    }
}
