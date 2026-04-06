//! `DesktopShell` — iced MVU root application for `FreeSynergy` Desktop.
//!
//! Architecture: Facade (`DesktopShell`) + State Machine (`CornerMenuState` per corner)
//!               + Observer (`CapabilityObserver` for optional services)
//!
//! G1.5 changes:
//!   - Left/right sidebars replaced by four corner menus.
//!   - Wallpaper rendered as the content-area background.
//!   - Titlebar extended: View-Buttons + Tiling-Toggle.
//!   - `CapabilityObserver` hides the AI corner menu when `ai.chat` is absent.
//!   - Desktop starts maximised by default (fullscreen flag in `main.rs`).

#[cfg(feature = "iced")]
use fs_gui_engine_iced::iced::{
    self, time,
    widget::{button, column, container, row, scrollable, stack, svg, text, text_input, Space},
    Alignment, Border, Color, Element, Length, Shadow, Subscription, Task, Vector,
};

use chrono::Local;

/// Convenience: translate a key to an owned `String`.
fn tr(key: &str) -> String {
    fs_i18n::t(key).to_string()
}

/// Convert an f32 pixel value to `Length::Fixed`.
#[cfg(feature = "iced")]
fn pxf(v: f32) -> Length {
    Length::Fixed(v)
}

/// Create an iced SVG handle from a raw SVG string.
///
/// Replaces `currentColor` with a concrete hex value so `resvg` renders it.
#[cfg(feature = "iced")]
fn svg_icon(svg_str: &str, _size: f32, color: &str) -> svg::Handle {
    let data = svg_str
        .replace("stroke=\"currentColor\"", &format!("stroke=\"{color}\""))
        .replace("fill=\"currentColor\"", &format!("fill=\"{color}\""));
    svg::Handle::from_memory(data.into_bytes())
}

// ── Palette ───────────────────────────────────────────────────────────────────

/// Theme-aware color palette for the shell chrome.
#[cfg(feature = "iced")]
struct Palette {
    bg_chrome: Color,
    bg_content: Color,
    #[allow(dead_code)]
    border: Color,
    border_accent: Color,
    icon_color: &'static str,
    muted: Color,
    #[allow(dead_code)]
    active_bg: Color,
    #[allow(dead_code)]
    active_border: Color,
    cyan: Color,
}

#[cfg(feature = "iced")]
impl Palette {
    fn dark() -> Self {
        Self {
            bg_chrome: Color::from_rgba(0.04, 0.06, 0.14, 0.97),
            bg_content: Color::from_rgb(0.05, 0.07, 0.15),
            border: Color::from_rgba(0.58, 0.67, 0.78, 0.10),
            border_accent: Color::from_rgba(0.02, 0.74, 0.84, 0.25),
            icon_color: "#94a3b8",
            muted: Color::from_rgb(0.40, 0.50, 0.60),
            active_bg: Color::from_rgba(0.02, 0.74, 0.84, 0.12),
            active_border: Color::from_rgb(0.02, 0.74, 0.84),
            cyan: Color::from_rgb(0.02, 0.74, 0.84),
        }
    }

    fn light() -> Self {
        Self {
            bg_chrome: Color::from_rgba(0.95, 0.97, 1.00, 0.97),
            bg_content: Color::from_rgb(0.97, 0.98, 1.00),
            border: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            border_accent: Color::from_rgba(0.02, 0.60, 0.75, 0.35),
            icon_color: "#334155",
            muted: Color::from_rgb(0.45, 0.52, 0.60),
            active_bg: Color::from_rgba(0.02, 0.74, 0.84, 0.10),
            active_border: Color::from_rgb(0.02, 0.60, 0.75),
            cyan: Color::from_rgb(0.02, 0.60, 0.75),
        }
    }
}

use fs_render::{new_shared_registry, register_standard_components, SharedComponentRegistry};

use crate::app_lifecycle::AppLifecycleBus;
use crate::capability_observer::CapabilityObserver;
use crate::corner_menus::{AiMenu, HelpMenu, SettingsMenu, TasksMenu};
use crate::header::{default_menu, HeaderState};
use crate::launcher::{AppGroup, LauncherState};
use crate::notification::{NotificationHistory, NotificationManager};
use crate::shell_layout::ShellLayout;
use crate::taskbar::{default_apps, AppEntry};
use crate::wallpaper::Wallpaper;
use crate::window::{AppId, Window, WindowHost, WindowId, WindowManager};

#[cfg(feature = "iced")]
use fs_gui_engine_iced::{
    navigation::{CornerMenuState, MenuConfig, NavMessage},
    render_corner_menu,
};

// ── DesktopMessage ────────────────────────────────────────────────────────────

/// All messages the desktop shell can process.
#[derive(Debug, Clone)]
pub enum DesktopMessage {
    // ── Window management ─────────────────────────────────────────────────────
    OpenApp(AppId),
    CloseWindow(WindowId),
    FocusWindow(WindowId),
    MinimizeWindow(WindowId),

    // ── Shell navigation ──────────────────────────────────────────────────────
    MenuAction(String),
    NotificationDismiss(u64),
    NotificationMarkRead,

    // ── Launcher ─────────────────────────────────────────────────────────────
    LauncherToggle,
    LauncherSearch(String),
    LauncherLaunch(String),
    LauncherClose,
    LauncherPrevPage,
    LauncherNextPage,
    LauncherGotoPage(usize),

    // ── Header ────────────────────────────────────────────────────────────────
    HeaderMenuToggle(usize),
    HeaderMenuClose,
    HeaderAvatarToggle,

    // ── Layout ───────────────────────────────────────────────────────────────
    /// Toggle visibility of a shell section (Topbar / Sidebar / Bottombar).
    LayoutToggleSection(fs_render::ShellKind),

    // ── Corner menus (G1.5) ───────────────────────────────────────────────────
    /// Wraps all `NavMessage` events emitted by the four corner menus.
    CornerMenuNav(NavMessage),

    // ── Titlebar extensions (G1.5) ────────────────────────────────────────────
    /// Toggle the tiling layout for open windows.
    TilingToggle,
    /// Switch to a specific `ProgramView` for the active app.
    ViewSwitch(fs_render::navigation::ProgramView),

    // ── Capability observer (G1.5) ────────────────────────────────────────────
    /// AI service appeared or disappeared in `fs-registry`.
    AiCapabilityChanged(bool),

    // ── Appearance ────────────────────────────────────────────────────────────
    /// Toggle light / dark mode.
    ToggleTheme,

    // ── Clock tick ────────────────────────────────────────────────────────────
    ClockTick,

    // ── No-op / async completion ──────────────────────────────────────────────
    Noop,
}

// ── DesktopShell ──────────────────────────────────────────────────────────────

/// Root desktop application state.
///
/// Owns all shell chrome state and routes to sub-app views.
/// Implements the iced MVU pattern via `update()` and `view()`.
pub struct DesktopShell {
    // ── Window management ─────────────────────────────────────────────────────
    pub windows: WindowManager,
    pub active_app: Option<AppId>,

    // ── Shell layout (Composite) ──────────────────────────────────────────────
    pub shell_layout: ShellLayout,

    // ── Component registry (Phase 3 components) ───────────────────────────────
    pub components: SharedComponentRegistry,

    // ── Shell chrome ─────────────────────────────────────────────────────────
    pub header_state: HeaderState,
    pub taskbar_apps: Vec<AppEntry>,
    pub notifications: NotificationManager,
    pub notification_history: NotificationHistory,
    pub launcher_state: LauncherState,
    pub current_desktop: usize,

    // ── App lifecycle bus (Observer) ──────────────────────────────────────────
    pub lifecycle_bus: AppLifecycleBus,

    // ── Corner menus (G1.5) — four screen corners ─────────────────────────────
    /// Top-left: task/app launcher menu.
    pub corner_tl: CornerMenuState,
    /// Bottom-left: settings menu.
    pub corner_bl: CornerMenuState,
    /// Top-right: help menu.
    pub corner_tr: CornerMenuState,
    /// Bottom-right: AI menu (shown only when capability present).
    pub corner_br: CornerMenuState,

    // ── Capability observer (G1.5) ────────────────────────────────────────────
    pub capability: CapabilityObserver,

    // ── Wallpaper (G1.5) ──────────────────────────────────────────────────────
    pub wallpaper: Wallpaper,

    // ── Tiling (G1.5) ────────────────────────────────────────────────────────
    /// `true` = automatic tiling layout for open windows.
    pub tiling_active: bool,

    // ── Navigation icon size (from settings) ──────────────────────────────────
    pub nav_icon_size: f32,

    // ── Theme ─────────────────────────────────────────────────────────────────
    /// `true` = dark mode (default), `false` = light mode.
    pub dark_mode: bool,

    // ── Clock ─────────────────────────────────────────────────────────────────
    pub clock_time: String,
    pub clock_date: String,
}

impl Default for DesktopShell {
    fn default() -> Self {
        crate::builtin_apps::ensure_registered();

        let components = new_shared_registry();
        {
            let mut reg = components.lock().unwrap();
            register_standard_components(&mut reg);
        }

        let layout = ShellLayout::load();

        Self {
            windows: WindowManager::default(),
            active_app: None,
            shell_layout: layout,
            components,
            header_state: HeaderState::new(std::env::var("USER").unwrap_or_else(|_| "User".into())),
            taskbar_apps: default_apps(),
            notifications: NotificationManager::default(),
            notification_history: NotificationHistory::default(),
            launcher_state: LauncherState::default(),
            current_desktop: 0,
            lifecycle_bus: AppLifecycleBus::with_defaults("desktop"),
            corner_tl: CornerMenuState::default(),
            corner_bl: CornerMenuState::default(),
            corner_tr: CornerMenuState::default(),
            corner_br: CornerMenuState::default(),
            capability: CapabilityObserver::default(),
            wallpaper: Wallpaper::default(),
            tiling_active: false,
            nav_icon_size: 32.0,
            dark_mode: true,
            clock_time: Local::now().format("%H:%M").to_string(),
            clock_date: Local::now().format("%d.%m.%Y").to_string(),
        }
    }
}

// ── update() ─────────────────────────────────────────────────────────────────

#[cfg(feature = "iced")]
impl DesktopShell {
    /// Process a `DesktopMessage` and return any async tasks.
    pub fn update(&mut self, msg: DesktopMessage) -> Task<DesktopMessage> {
        match msg {
            // ── Window management ─────────────────────────────────────────────
            DesktopMessage::OpenApp(app_id) => {
                Self::spawn_app(app_id);
                self.lifecycle_bus
                    .app_opened(app_id.name().to_lowercase().as_str());
                self.active_app = Some(app_id);
                let meta = Window::new(app_id.name()).with_icon(app_id.icon().to_string());
                let open = crate::window::OpenWindow::new(meta, app_id);
                self.windows.open_window(open);
            }
            DesktopMessage::CloseWindow(id) => {
                if let Some(win) = self.windows.open_windows().iter().find(|w| w.id == id) {
                    self.lifecycle_bus
                        .app_closed(win.app.name().to_lowercase().as_str());
                }
                self.windows.close_window(id);
                if self.windows.open_windows().is_empty() {
                    self.active_app = None;
                }
            }
            DesktopMessage::FocusWindow(id) => {
                self.windows.focus_window(id);
            }
            DesktopMessage::MinimizeWindow(id) => {
                self.windows.minimize_window(id);
            }

            // ── Shell navigation ──────────────────────────────────────────────
            DesktopMessage::MenuAction(id) => {
                self.handle_menu_action(&id);
            }
            DesktopMessage::NotificationDismiss(id) => {
                self.notifications.dismiss(id);
            }
            DesktopMessage::NotificationMarkRead => {
                self.notification_history.mark_all_read();
            }

            // ── Launcher ─────────────────────────────────────────────────────
            DesktopMessage::LauncherToggle => {
                self.launcher_state.toggle();
            }
            DesktopMessage::LauncherSearch(q) => {
                self.launcher_state.set_query(q);
            }
            DesktopMessage::LauncherLaunch(id) => {
                self.launcher_state.close();
                self.handle_corner_action(&id);
            }
            DesktopMessage::LauncherClose => {
                self.launcher_state.close();
            }
            DesktopMessage::LauncherPrevPage => {
                let groups = AppGroup::filtered(&self.taskbar_apps, &self.launcher_state.query);
                let total = AppGroup::total_pages(&groups);
                self.launcher_state.prev_page(total);
            }
            DesktopMessage::LauncherNextPage => {
                let groups = AppGroup::filtered(&self.taskbar_apps, &self.launcher_state.query);
                let total = AppGroup::total_pages(&groups);
                self.launcher_state.next_page(total);
            }
            DesktopMessage::LauncherGotoPage(idx) => {
                self.launcher_state.goto_page(idx);
            }

            // ── Header ────────────────────────────────────────────────────────
            DesktopMessage::HeaderMenuToggle(idx) => {
                self.header_state.open_menu = if self.header_state.open_menu == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
            }
            DesktopMessage::HeaderMenuClose => {
                self.header_state.open_menu = None;
            }
            DesktopMessage::HeaderAvatarToggle => {
                self.header_state.avatar_menu_open = !self.header_state.avatar_menu_open;
            }

            // ── Layout ───────────────────────────────────────────────────────
            DesktopMessage::LayoutToggleSection(kind) => {
                self.shell_layout.toggle_visibility(&kind);
                self.shell_layout.save();
            }

            // ── Corner menus (G1.5) ───────────────────────────────────────────
            DesktopMessage::CornerMenuNav(nav_msg) => {
                use fs_gui_engine_iced::update_corner_menu;
                use fs_render::navigation::Corner;
                update_corner_menu(&mut self.corner_tl, Corner::TopLeft, &nav_msg);
                update_corner_menu(&mut self.corner_bl, Corner::BottomLeft, &nav_msg);
                update_corner_menu(&mut self.corner_tr, Corner::TopRight, &nav_msg);
                update_corner_menu(&mut self.corner_br, Corner::BottomRight, &nav_msg);
                // Dispatch corner actions to app / help / settings logic.
                if let NavMessage::CornerMenuAction(_, action) = &nav_msg {
                    self.handle_corner_action(action);
                }
            }

            // ── Titlebar extensions ────────────────────────────────────────────
            DesktopMessage::TilingToggle => {
                self.tiling_active = !self.tiling_active;
            }
            DesktopMessage::ViewSwitch(_view) => {
                // G1.4: ProgramViewProvider per-app — wired when G1.4 lands.
            }

            // ── Capability observer ───────────────────────────────────────────
            DesktopMessage::AiCapabilityChanged(available) => {
                self.capability.set_ai_chat(available);
            }

            // ── Appearance ────────────────────────────────────────────────────
            DesktopMessage::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
            }

            // ── Clock ─────────────────────────────────────────────────────────
            DesktopMessage::ClockTick => {
                self.clock_time = Local::now().format("%H:%M").to_string();
                self.clock_date = Local::now().format("%d.%m.%Y").to_string();
            }

            DesktopMessage::Noop => {}
        }

        // Sync help context when active app changes.
        if let Some(app) = self.active_app {
            // Help context is handled by the help corner menu (G1.7 wiring).
            let _ = app;
        }

        Task::none()
    }

    // ── Action dispatch ───────────────────────────────────────────────────────

    fn handle_menu_action(&mut self, id: &str) {
        self.header_state.open_menu = None;
        match id {
            "launcher" => self.launcher_state.toggle(),
            "settings" => {
                self.active_app = Some(AppId::Settings);
            }
            _ => {}
        }
    }

    /// Handle an action string dispatched from any corner menu or launcher.
    ///
    /// Action format: `"<category>:<id>"`, e.g. `"open:browser"`.
    fn handle_corner_action(&mut self, action: &str) {
        let app_id = match action {
            "open:launcher" => {
                self.launcher_state.toggle();
                return;
            }
            "open:browser" => Some(AppId::Browser),
            "open:store" => Some(AppId::Store),
            "open:lenses" => Some(AppId::Lenses),
            "open:tasks" => Some(AppId::Tasks),
            "open:bots" => Some(AppId::Bots),
            "open:managers" => Some(AppId::Managers),
            "open:profile" => Some(AppId::Profile),
            "open:ai" | "ai:chat" | "ai:suggest" => Some(AppId::Ai),
            "open:container" => Some(AppId::Container),
            "settings:appearance" | "settings:language" | "settings:desktop" => {
                Some(AppId::Settings)
            }
            "help:general" | "help:focus" | "help:docs" => Some(AppId::Help),
            _ => None,
        };
        if let Some(app) = app_id {
            if app != AppId::Help {
                Self::spawn_app(app);
            }
            self.active_app = Some(app);
            self.lifecycle_bus
                .app_opened(app.name().to_lowercase().as_str());
        }
    }

    /// Launch an external app binary as a detached child process.
    fn spawn_app(app_id: AppId) {
        let binary = match app_id {
            AppId::Browser => "fs-browser",
            AppId::Settings => "fs-settings",
            AppId::Profile => "fs-profile",
            AppId::Store => "fs-store",
            AppId::Lenses => "fs-lenses",
            AppId::Builder => "fs-builder",
            AppId::Tasks => "fs-tasks",
            AppId::Bots => "fs-bots",
            AppId::Ai => "fs-ai",
            AppId::Container => "fs-container",
            AppId::Managers => "fs-managers",
            AppId::Help => return,
        };
        let _ = std::process::Command::new(binary).spawn();
    }

    // ── Subscription ──────────────────────────────────────────────────────────

    /// iced subscription: clock update.
    pub fn subscription(&self) -> Subscription<DesktopMessage> {
        use std::time::Duration;
        time::every(Duration::from_secs(30)).map(|_| DesktopMessage::ClockTick)
    }
}

// ── view() ────────────────────────────────────────────────────────────────────

#[cfg(feature = "iced")]
impl DesktopShell {
    fn palette(&self) -> Palette {
        if self.dark_mode {
            Palette::dark()
        } else {
            Palette::light()
        }
    }

    fn chrome_style(p: &Palette, border_side_color: Color) -> container::Style {
        container::Style {
            background: Some(iced::Background::Color(p.bg_chrome)),
            border: Border {
                color: border_side_color,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 12.0,
            },
            ..container::Style::default()
        }
    }

    /// Render the full desktop shell.
    ///
    /// Layout (G1.5):
    /// ```text
    /// ┌────────────────────────────┐
    /// │         header (60px)      │
    /// ├────────────────────────────┤
    /// │                            │
    /// │    content (wallpaper)     │
    /// │                            │
    /// ├────────────────────────────┤
    /// │         taskbar (48px)     │
    /// └────────────────────────────┘
    ///  ↑ corner menus overlaid via stack
    /// ```
    #[must_use]
    pub fn view(&self) -> Element<'_, DesktopMessage> {
        if self.launcher_state.open {
            return self.view_launcher();
        }

        let p = self.palette();

        let header = self.view_header();
        let content = self.view_content();
        let taskbar = self.view_taskbar();

        let shell = column![header, content, taskbar]
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill);

        let shell_bg_color = self.wallpaper_background_color(&p);
        let shell_el: Element<'_, DesktopMessage> = container(shell)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(shell_bg_color)),
                ..container::Style::default()
            })
            .into();

        // Overlay the four corner menus.
        let overlays = self.view_corner_overlays();

        let mut layers: Vec<Element<'_, DesktopMessage>> = vec![shell_el];
        layers.extend(overlays);
        stack(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // ── Wallpaper ─────────────────────────────────────────────────────────────

    fn wallpaper_background_color(&self, p: &Palette) -> Color {
        use crate::wallpaper::WallpaperSource;
        match &self.wallpaper.source {
            WallpaperSource::Color { hex } => parse_hex_color(hex).unwrap_or(p.bg_content),
            _ => p.bg_content,
        }
    }

    // ── Corner menus overlay ──────────────────────────────────────────────────

    /// Build the four corner menu overlays (each fills the full viewport).
    fn view_corner_overlays(&self) -> Vec<Element<'_, DesktopMessage>> {
        use fs_render::navigation::Corner;

        let config = MenuConfig {
            icon_size: self.nav_icon_size,
            max_icon_size: self.nav_icon_size * 1.5,
            ..MenuConfig::default()
        };

        let mut overlays = Vec::new();

        // Top-left: tasks / app launcher
        let tl_desc = TasksMenu::default_entries();
        let tl_el = render_corner_menu(&tl_desc, &self.corner_tl, &config)
            .map(DesktopMessage::CornerMenuNav);
        overlays.push(Self::corner_overlay(tl_el, Corner::TopLeft));

        // Bottom-left: settings
        let bl_desc = SettingsMenu::default_entries();
        let bl_el = render_corner_menu(&bl_desc, &self.corner_bl, &config)
            .map(DesktopMessage::CornerMenuNav);
        overlays.push(Self::corner_overlay(bl_el, Corner::BottomLeft));

        // Top-right: help
        let help_desc = HelpMenu::default_entries();
        let help_el = render_corner_menu(&help_desc, &self.corner_tr, &config)
            .map(DesktopMessage::CornerMenuNav);
        overlays.push(Self::corner_overlay(help_el, Corner::TopRight));

        // Bottom-right: AI (only when capability present)
        if self.capability.ai_chat_available() {
            let ai_desc = AiMenu::default_entries();
            let ai_el = render_corner_menu(&ai_desc, &self.corner_br, &config)
                .map(DesktopMessage::CornerMenuNav);
            overlays.push(Self::corner_overlay(ai_el, Corner::BottomRight));
        }

        overlays
    }

    /// Wrap a corner menu element in a full-viewport container aligned to `corner`.
    fn corner_overlay(
        el: Element<'_, DesktopMessage>,
        corner: fs_render::navigation::Corner,
    ) -> Element<'_, DesktopMessage> {
        use fs_render::navigation::Corner as C;
        let (h, v) = match corner {
            C::TopLeft => (Alignment::Start, Alignment::Start),
            C::TopRight => (Alignment::End, Alignment::Start),
            C::BottomLeft => (Alignment::Start, Alignment::End),
            C::BottomRight => (Alignment::End, Alignment::End),
        };
        container(el)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(h)
            .align_y(v)
            .into()
    }

    // ── Header ────────────────────────────────────────────────────────────────

    fn view_header(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();

        let brand = text("FreeSynergy").size(15).color(p.cyan);
        let by_kal = text(" by KalEl").size(11).color(p.muted);
        let brand_row = row![brand, by_kal].align_y(Alignment::Center);

        let menu_bar = Self::view_menu_bar();

        // Theme toggle
        let theme_svg = if self.dark_mode {
            svg_icon(crate::icons::ICON_SUN, 16.0, p.icon_color)
        } else {
            svg_icon(crate::icons::ICON_MOON, 16.0, p.icon_color)
        };
        let theme_btn = button(svg(theme_svg).width(16).height(16))
            .on_press(DesktopMessage::ToggleTheme)
            .padding([4, 8]);

        // Tiling toggle (G1.5)
        let tiling_label = if self.tiling_active {
            text("⊞").size(16).color(p.cyan)
        } else {
            text("⊞").size(16).color(p.muted)
        };
        let tiling_btn = button(tiling_label)
            .on_press(DesktopMessage::TilingToggle)
            .padding([4, 8]);

        // View-buttons (G1.5): only when an app is active
        let view_btns = self.view_program_view_buttons(&p);

        // Clock
        let clock = column![
            text(&self.clock_time).size(13),
            text(&self.clock_date).size(10).color(p.muted),
        ]
        .align_x(Alignment::Center);

        // Bell / notifications
        let notif_count = self.notification_history.unread_count();
        let bell_handle = svg_icon(crate::icons::ICON_BELL, 18.0, p.icon_color);
        let bell_label: Element<'_, DesktopMessage> = if notif_count > 0 {
            row![
                svg(bell_handle).width(18).height(18),
                text(format!(" {notif_count}")).size(11).color(p.cyan),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            svg(bell_handle).width(18).height(18).into()
        };
        let bell_btn = button(bell_label)
            .on_press(DesktopMessage::NotificationMarkRead)
            .padding([4, 8]);

        // Avatar
        let avatar_initial = self
            .header_state
            .user_name
            .chars()
            .next()
            .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string());
        let avatar_btn = button(
            container(text(avatar_initial).size(12))
                .width(28)
                .height(28)
                .center_x(28)
                .center_y(28)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.cyan)),
                    border: Border {
                        radius: 14.0.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }),
        )
        .on_press(DesktopMessage::HeaderAvatarToggle)
        .padding(0);

        let header_row = row![
            brand_row,
            Space::with_width(12),
            menu_bar,
            Space::with_width(Length::Fill),
        ]
        .align_y(Alignment::Center)
        .spacing(4)
        .padding([0, 8]);

        // Append view buttons + titlebar controls
        let mut controls: Vec<Element<'_, DesktopMessage>> = vec![header_row.into()];
        controls.extend(view_btns);
        controls.push(Space::with_width(8).into());
        controls.push(tiling_btn.into());
        controls.push(Space::with_width(4).into());
        controls.push(theme_btn.into());
        controls.push(Space::with_width(4).into());
        controls.push(bell_btn.into());
        controls.push(Space::with_width(8).into());
        controls.push(avatar_btn.into());
        controls.push(Space::with_width(12).into());
        controls.push(clock.into());
        controls.push(Space::with_width(8).into());

        let header_inner = row(controls)
            .align_y(Alignment::Center)
            .spacing(0)
            .height(60);

        container(header_inner)
            .width(Length::Fill)
            .height(60)
            .style(move |_| Self::chrome_style(&p, p.border_accent))
            .into()
    }

    /// View-buttons: one button per supported `ProgramView` of the active app (G1.5).
    ///
    /// For G1.5 the available views are always `Start + Info + Manual + Settings`.
    /// When G1.4 (`ProgramViewProvider` per-app) lands the set will be dynamic.
    fn view_program_view_buttons(&self, p: &Palette) -> Vec<Element<'_, DesktopMessage>> {
        use fs_render::navigation::ProgramView;
        if self.active_app.is_none() {
            return vec![];
        }

        let views = [
            (ProgramView::Start, tr("desktop-titlebar-view-start")),
            (ProgramView::Info, tr("desktop-titlebar-view-info")),
            (ProgramView::Manual, tr("desktop-titlebar-view-manual")),
            (
                ProgramView::SettingsConfig,
                tr("desktop-titlebar-view-settings"),
            ),
        ];

        views
            .into_iter()
            .map(|(view, label)| {
                button(text(label).size(11).color(p.muted))
                    .on_press(DesktopMessage::ViewSwitch(view))
                    .padding([3, 6])
                    .into()
            })
            .collect()
    }

    fn view_menu_bar() -> Element<'static, DesktopMessage> {
        let menus = default_menu();
        let labels: Vec<(usize, String)> = menus
            .iter()
            .enumerate()
            .map(|(idx, m)| (idx, m.label.clone()))
            .collect();
        let buttons: Vec<Element<'_, DesktopMessage>> = labels
            .into_iter()
            .map(|(idx, label)| {
                button(text(label).size(13))
                    .on_press(DesktopMessage::HeaderMenuToggle(idx))
                    .padding([4, 10])
                    .into()
            })
            .collect();
        row(buttons).spacing(2).into()
    }

    // ── Content area ──────────────────────────────────────────────────────────

    fn view_content(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();

        let content: Element<'_, DesktopMessage> = match self.active_app {
            Some(app_id) => {
                let handle = svg_icon(app_id.icon(), 48.0, p.icon_color);
                let icon_el: Element<'_, DesktopMessage> = svg(handle).width(48).height(48).into();
                container(
                    column![
                        icon_el,
                        Space::with_height(16),
                        text(app_id.name()).size(20).color(p.cyan),
                        Space::with_height(8),
                        text(tr("shell-app-launched")).size(14).color(p.muted),
                        Space::with_height(16),
                        button(text(tr("shell-app-relaunch")).size(13))
                            .on_press(DesktopMessage::OpenApp(app_id))
                            .padding([8, 20]),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(4),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            }
            None => container(
                column![
                    text("FreeSynergy").size(40).color(p.cyan),
                    text("by KalEl").size(14).color(p.muted),
                    Space::with_height(32),
                    text(tr("shell-home-hint")).size(14).color(p.muted),
                    Space::with_height(20),
                    button(
                        row![
                            svg(svg_icon(crate::icons::ICON_LAUNCHER, 16.0, "#06b6d4"))
                                .width(16)
                                .height(16),
                            Space::with_width(6),
                            text(tr("shell-launcher-open")).size(14),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .on_press(DesktopMessage::LauncherToggle)
                    .padding([10, 24]),
                ]
                .align_x(Alignment::Center)
                .spacing(4),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_content)),
                ..container::Style::default()
            })
            .into()
    }

    // ── Taskbar ───────────────────────────────────────────────────────────────

    fn view_taskbar(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();

        let launcher_btn = button(
            svg(svg_icon(crate::icons::ICON_LAUNCHER, 20.0, p.icon_color))
                .width(20)
                .height(20),
        )
        .on_press(DesktopMessage::LauncherToggle)
        .padding([4, 10]);

        let mut app_btns: Vec<Element<'_, DesktopMessage>> = vec![launcher_btn.into()];

        for app in &self.taskbar_apps {
            let id = app.id.clone();
            let icon_el: Element<'_, DesktopMessage> = if app.icon.starts_with('<') {
                let handle = svg_icon(&app.icon, 20.0, p.icon_color);
                svg(handle).width(20).height(20).into()
            } else {
                text(app.icon.clone()).size(18).into()
            };
            let btn = button(icon_el)
                .on_press(DesktopMessage::LauncherLaunch(id))
                .padding([4, 8]);
            app_btns.push(btn.into());
        }

        app_btns.push(Space::with_width(Length::Fill).into());

        let clock = column![
            text(&self.clock_time).size(13),
            text(&self.clock_date).size(10).color(p.muted),
        ]
        .align_x(Alignment::Center)
        .padding([0, 12]);
        app_btns.push(clock.into());

        let taskbar_row = row(app_btns)
            .align_y(Alignment::Center)
            .spacing(4)
            .padding([0, 8])
            .height(48);

        container(taskbar_row)
            .width(Length::Fill)
            .height(48)
            .style(move |_| Self::chrome_style(&p, p.border_accent))
            .into()
    }

    // ── Launcher overlay ──────────────────────────────────────────────────────

    fn view_launcher(&self) -> Element<'_, DesktopMessage> {
        use fs_i18n::t_with;

        let query = self.launcher_state.query.clone();
        let groups = AppGroup::filtered(&self.taskbar_apps, &query);
        let total_pages = AppGroup::total_pages(&groups);
        let cur_page = self.launcher_state.page.min(total_pages - 1);
        let page_groups: Vec<AppGroup> = AppGroup::page_slice(&groups, cur_page).to_vec();

        let search_placeholder = tr("shell-launcher-search-placeholder");
        let search = text_input(&search_placeholder, &query)
            .on_input(DesktopMessage::LauncherSearch)
            .padding([10, 14])
            .size(15)
            .width(Length::Fill);

        let mut group_items: Vec<Element<'_, DesktopMessage>> = vec![];

        if groups.is_empty() {
            group_items.push(
                container(
                    text(
                        t_with("shell-launcher-no-apps", &[("query", query.as_str())]).to_string(),
                    )
                    .size(14)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.7)),
                )
                .center_x(Length::Fill)
                .padding([20, 0])
                .into(),
            );
        }

        for group in &page_groups {
            let group_label = text(group.label.clone())
                .size(11)
                .color(iced::Color::from_rgb(0.5, 0.6, 0.7));
            group_items.push(group_label.into());

            let mut row_items: Vec<Element<'_, DesktopMessage>> = vec![];
            for entry in &group.apps {
                let icon_el: Element<'_, DesktopMessage> = if entry.icon.starts_with('<') {
                    let handle = svg_icon(&entry.icon, 32.0, "#94a3b8");
                    svg(handle).width(32).height(32).into()
                } else {
                    text(entry.icon.clone()).size(28).into()
                };
                let id_clone = entry.id.clone();
                let app_btn = button(
                    column![
                        icon_el,
                        text(fs_i18n::t(&entry.label_key).to_string()).size(11),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(4),
                )
                .on_press(DesktopMessage::LauncherLaunch(id_clone))
                .padding([8, 12]);
                row_items.push(app_btn.into());
            }
            group_items.push(row(row_items).spacing(8).into());
            group_items.push(Space::with_height(12).into());
        }

        // Pagination
        if total_pages > 1 {
            let page_label = text(
                fs_i18n::t_with(
                    "shell-launcher-page",
                    &[
                        ("n", &(cur_page + 1).to_string()),
                        ("total", &total_pages.to_string()),
                    ],
                )
                .to_string(),
            )
            .size(12)
            .color(iced::Color::from_rgb(0.5, 0.6, 0.7));

            let prev_btn = button(text("←").size(14))
                .on_press(DesktopMessage::LauncherPrevPage)
                .padding([4, 10]);
            let next_btn = button(text("→").size(14))
                .on_press(DesktopMessage::LauncherNextPage)
                .padding([4, 10]);
            let page_row = row![
                prev_btn,
                Space::with_width(8),
                page_label,
                Space::with_width(8),
                next_btn
            ]
            .align_y(Alignment::Center);
            group_items.push(
                container(page_row)
                    .center_x(Length::Fill)
                    .padding([8, 0])
                    .into(),
            );
        }

        let close_btn = button(text("✕").size(16))
            .on_press(DesktopMessage::LauncherClose)
            .padding([6, 10]);

        let launcher_content = column![
            row![
                text(tr("shell-launcher-title")).size(16),
                Space::with_width(Length::Fill),
                close_btn,
            ]
            .align_y(Alignment::Center)
            .padding([0, 4]),
            Space::with_height(8),
            search,
            Space::with_height(12),
            scrollable(column(group_items).spacing(4)).height(Length::Fill),
        ]
        .padding([16, 16])
        .spacing(0);

        container(launcher_content)
            .width(pxf(540.0))
            .height(pxf(480.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.04, 0.06, 0.14, 0.97,
                ))),
                border: Border {
                    color: Color::from_rgba(0.02, 0.74, 0.84, 0.30),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.50),
                    offset: Vector::new(0.0, 8.0),
                    blur_radius: 32.0,
                },
                ..container::Style::default()
            })
            .into()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a CSS hex color string (`"#rrggbb"` or `"#rgb"`) into an iced `Color`.
///
/// Returns `None` on parse failure so callers can fall back to a default.
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };
    #[allow(clippy::cast_lossless)]
    Some(Color::from_rgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    ))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_six_digits() {
        let c = parse_hex_color("#0bbed4").unwrap();
        assert!((c.r - 11.0 / 255.0).abs() < 0.01);
        assert!((c.g - 190.0 / 255.0).abs() < 0.01);
        assert!((c.b - 212.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_three_digits() {
        let c = parse_hex_color("#0bd").unwrap();
        assert!((c.r - 0.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_invalid_returns_none() {
        assert!(parse_hex_color("#xyz").is_none());
        assert!(parse_hex_color("#12").is_none());
    }

    #[test]
    fn desktop_shell_default_dark_mode() {
        let shell = DesktopShell::default();
        assert!(shell.dark_mode);
    }

    #[test]
    fn desktop_shell_default_no_active_app() {
        let shell = DesktopShell::default();
        assert!(shell.active_app.is_none());
    }

    #[test]
    fn desktop_shell_default_tiling_off() {
        let shell = DesktopShell::default();
        assert!(!shell.tiling_active);
    }
}
