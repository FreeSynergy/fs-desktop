//! `DesktopShell` — iced MVU root application for `FreeSynergy` Desktop.
//!
//! Architecture: Elm (MVU) pattern via iced 0.13.
//!   - `DesktopShell`   — owns shell chrome state + active app routing
//!   - `DesktopMessage` — flat enum wrapping all sub-app messages + shell actions
//!   - `update()`       — state transitions
//!   - `view()`         — shell chrome + active app content placeholder

#[cfg(feature = "iced")]
use fs_gui_engine_iced::iced::{
    self, event, mouse, time,
    widget::{button, column, container, row, scrollable, svg, text, text_input, Space},
    window, Alignment, Border, Color, Element, Length, Shadow, Subscription, Task, Vector,
};

use chrono::Local;

/// Convenience: translate key to owned `String` for use in iced widgets.
fn tr(key: &str) -> String {
    fs_i18n::t(key).to_string()
}

/// Convert an f32 pixel value to `Length::Fixed`.
#[cfg(feature = "iced")]
fn pxf(v: f32) -> Length {
    Length::Fixed(v)
}

/// Create an `iced` SVG handle from a raw SVG string.
///
/// `currentColor` only works in HTML/CSS context — we replace it with a
/// concrete hex color before creating the memory handle so `resvg` renders it.
/// The `_size` hint is informational (the SVG carries its own `width`/`height`).
#[cfg(feature = "iced")]
fn svg_icon(svg_str: &str, _size: f32, color: &str) -> svg::Handle {
    let data = svg_str
        .replace("stroke=\"currentColor\"", &format!("stroke=\"{color}\""))
        .replace("fill=\"currentColor\"", &format!("fill=\"{color}\""));
    svg::Handle::from_memory(data.into_bytes())
}

/// Theme-aware color palette for the shell chrome.
#[cfg(feature = "iced")]
struct Palette {
    /// Primary background (header, sidebars, taskbar).
    bg_chrome: Color,
    /// Content area background.
    bg_content: Color,
    /// Subtle border / divider.
    border: Color,
    /// Accent border (bottom of header, right of sidebar).
    border_accent: Color,
    /// Icon / label foreground color as hex string (for SVG replacement).
    icon_color: &'static str,
    /// Muted text (section labels).
    muted: Color,
    /// Active item highlight background.
    active_bg: Color,
    /// Active item border.
    active_border: Color,
    /// Accent (brand cyan).
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

/// Convenience: translate key with variables to owned `String`.
fn tr_with(key: &str, args: &[(&str, &str)]) -> String {
    fs_i18n::t_with(key, args).to_string()
}

use fs_render::{new_shared_registry, register_standard_components, SharedComponentRegistry};

use crate::app_lifecycle::AppLifecycleBus;
use crate::header::{default_menu, HeaderState};
use crate::help_sidebar::HelpSidebarState;
use crate::launcher::{AppGroup, LauncherState};
use crate::notification::{NotificationHistory, NotificationManager};
use crate::shell_layout::ShellLayout;
use crate::sidebar::{default_pinned_items, default_sidebar_sections, SidebarItem, SidebarSection};
use crate::sidebar_state::{
    MouseProximityObserver, SidebarMode, SidebarSide, SidebarState, SidebarTransition,
};
use crate::taskbar::{default_apps, AppEntry};
use crate::window::{AppId, Window, WindowHost, WindowId, WindowManager};

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
    SidebarSelect(String),
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
    /// Toggle visibility of a shell section (Topbar/Sidebar/Bottombar).
    LayoutToggleSection(fs_render::ShellKind),
    /// Pin/unpin an app (persists to `PackageRegistry`).
    PinApp(String),
    UnpinApp(String),

    // ── Sidebar state machine ─────────────────────────────────────────────────
    /// Cursor moved — x position (window width is stored in state).
    CursorMoved(f32),
    /// Window resized — new logical width.
    WindowResized(f32),
    /// Animation tick for sidebar width lerp (~30 fps).
    SidebarAnimTick,
    /// Toggle pin on the left sidebar (stays open / auto-collapse).
    LeftSidebarTogglePin,
    /// Toggle pin on the right help sidebar.
    RightSidebarTogglePin,
    /// Toggle light / dark mode.
    ToggleTheme,

    // ── Help sidebar ──────────────────────────────────────────────────────────
    /// User typed into the AI input field.
    HelpAiInputChanged(String),
    /// User submitted the AI query.
    HelpAiSend,

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

    // ── Component registry (Phase 3 components) ────────────────────────────────
    pub components: SharedComponentRegistry,

    // ── Shell chrome ─────────────────────────────────────────────────────────
    pub header_state: HeaderState,
    pub taskbar_apps: Vec<AppEntry>,
    pub notifications: NotificationManager,
    pub notification_history: NotificationHistory,
    pub sidebar_sections: Vec<SidebarSection>,
    pub pinned_items: Vec<SidebarItem>,
    pub launcher_state: LauncherState,
    pub current_desktop: usize,

    // ── App lifecycle bus (Observer) ──────────────────────────────────────────
    pub lifecycle_bus: AppLifecycleBus,

    // ── Sidebar state machines ────────────────────────────────────────────────
    /// Visual state of the left (taskbar) sidebar.
    pub left_sidebar_state: SidebarState,
    /// Mode of the left sidebar (Auto | Pinned).
    pub left_sidebar_mode: SidebarMode,
    /// Visual state of the right (help) sidebar.
    pub help_sidebar: HelpSidebarState,
    /// Proximity observer for the left sidebar.
    pub left_proximity: MouseProximityObserver,
    /// Proximity observer for the right sidebar.
    pub right_proximity: MouseProximityObserver,

    // ── Sidebar animation (smooth width lerp) ─────────────────────────────────
    /// Animated pixel width of the left sidebar (interpolates toward target).
    pub left_anim_width: f32,
    /// Animated pixel width of the right sidebar.
    pub right_anim_width: f32,

    // ── Window / layout tracking ──────────────────────────────────────────────
    /// Current logical window width — updated via `WindowResized` events.
    pub window_width: f32,

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
        let left_size = layout
            .sidebars_on_side(SidebarSide::Left)
            .first()
            .map_or(220, |s| s.size);
        let right_size = layout
            .sidebars_on_side(SidebarSide::Right)
            .first()
            .map_or(320, |s| s.size);

        #[allow(clippy::cast_precision_loss)]
        let left_start = SidebarState::COLLAPSED_WIDTH as f32;
        #[allow(clippy::cast_precision_loss)]
        let right_start = SidebarState::COLLAPSED_WIDTH as f32;

        Self {
            windows: WindowManager::default(),
            active_app: None,
            shell_layout: layout,
            components,
            header_state: HeaderState::new(std::env::var("USER").unwrap_or_else(|_| "User".into())),
            taskbar_apps: default_apps(),
            notifications: NotificationManager::default(),
            notification_history: NotificationHistory::default(),
            sidebar_sections: default_sidebar_sections(),
            pinned_items: default_pinned_items(),
            launcher_state: LauncherState::default(),
            current_desktop: 0,
            lifecycle_bus: AppLifecycleBus::with_defaults("desktop"),
            left_sidebar_state: SidebarState::Collapsed,
            left_sidebar_mode: SidebarMode::Auto,
            help_sidebar: HelpSidebarState::default(),
            left_proximity: MouseProximityObserver::new(SidebarSide::Left, left_size),
            right_proximity: MouseProximityObserver::new(SidebarSide::Right, right_size),
            left_anim_width: left_start,
            right_anim_width: right_start,
            window_width: 1280.0,
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
                self.lifecycle_bus
                    .app_opened(app_id.name().to_lowercase().as_str());
                self.active_app = Some(app_id);
                let meta = Window::new(app_id.name()).with_icon(app_id.icon().to_string());
                let open = crate::window::OpenWindow::new(meta, app_id);
                self.windows.open_window(open);
            }
            DesktopMessage::CloseWindow(id) => {
                // Emit closed event for the window being removed.
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
            DesktopMessage::SidebarSelect(id) => {
                self.handle_sidebar_select(&id);
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
                self.handle_sidebar_select(&id);
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
                // Rebuild sidebar sections from registry after layout change.
                self.sidebar_sections = default_sidebar_sections();
                self.pinned_items = default_pinned_items();
            }
            DesktopMessage::PinApp(id) => {
                use fs_db_desktop::package_registry::PackageRegistry;
                let _ = PackageRegistry::set_pinned(&id, true);
                self.lifecycle_bus.app_pinned(&id);
                self.sidebar_sections = default_sidebar_sections();
                self.pinned_items = default_pinned_items();
            }
            DesktopMessage::UnpinApp(id) => {
                use fs_db_desktop::package_registry::PackageRegistry;
                let _ = PackageRegistry::set_pinned(&id, false);
                self.lifecycle_bus.app_unpinned(&id);
                self.sidebar_sections = default_sidebar_sections();
                self.pinned_items = default_pinned_items();
            }

            // ── Sidebar state machine ─────────────────────────────────────────
            DesktopMessage::CursorMoved(x) => {
                // Left sidebar — collapse/expand jumps directly to Open (animation handles visual)
                if let Some(t) =
                    self.left_proximity
                        .check(x, self.window_width, self.left_sidebar_state)
                {
                    self.left_sidebar_state = match t {
                        SidebarTransition::StartExpand | SidebarTransition::Open => {
                            SidebarState::Open
                        }
                        SidebarTransition::Collapse => SidebarState::Collapsed,
                    };
                }
                // Right sidebar
                if let Some(t) =
                    self.right_proximity
                        .check(x, self.window_width, self.help_sidebar.state)
                {
                    self.help_sidebar.state = match t {
                        SidebarTransition::StartExpand | SidebarTransition::Open => {
                            SidebarState::Open
                        }
                        SidebarTransition::Collapse => SidebarState::Collapsed,
                    };
                }
            }
            DesktopMessage::WindowResized(w) => {
                self.window_width = w;
            }
            DesktopMessage::SidebarAnimTick => {
                let left_target = self.left_sidebar_target();
                let right_target = self.right_sidebar_target();
                // Lerp factor: ~18% per tick at 30 fps ≈ smooth ~0.5 s expand
                self.left_anim_width += (left_target - self.left_anim_width) * 0.22;
                self.right_anim_width += (right_target - self.right_anim_width) * 0.22;
                // Snap to target when close enough to avoid infinite tiny diffs
                if (self.left_anim_width - left_target).abs() < 0.5 {
                    self.left_anim_width = left_target;
                }
                if (self.right_anim_width - right_target).abs() < 0.5 {
                    self.right_anim_width = right_target;
                }
            }
            DesktopMessage::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
            }
            DesktopMessage::LeftSidebarTogglePin => {
                self.left_sidebar_mode = match self.left_sidebar_mode {
                    SidebarMode::Auto => {
                        self.left_sidebar_state = SidebarState::Open;
                        self.left_proximity.mode = SidebarMode::Pinned;
                        SidebarMode::Pinned
                    }
                    SidebarMode::Pinned => {
                        self.left_proximity.mode = SidebarMode::Auto;
                        SidebarMode::Auto
                    }
                };
            }
            DesktopMessage::RightSidebarTogglePin => {
                self.help_sidebar.mode = match self.help_sidebar.mode {
                    SidebarMode::Auto => {
                        self.help_sidebar.state = SidebarState::Open;
                        self.right_proximity.mode = SidebarMode::Pinned;
                        SidebarMode::Pinned
                    }
                    SidebarMode::Pinned => {
                        self.right_proximity.mode = SidebarMode::Auto;
                        SidebarMode::Auto
                    }
                };
            }

            // ── Help sidebar ──────────────────────────────────────────────────
            DesktopMessage::HelpAiInputChanged(text) => {
                self.help_sidebar.ai_input = text;
            }
            DesktopMessage::HelpAiSend => {
                let query = std::mem::take(&mut self.help_sidebar.ai_input);
                if !query.is_empty() {
                    use crate::help_sidebar::{AiHelpSource, HelpSource};
                    self.help_sidebar.content = AiHelpSource.resolve(&query);
                }
            }

            // ── Clock tick ────────────────────────────────────────────────────
            DesktopMessage::ClockTick => {
                self.clock_time = Local::now().format("%H:%M").to_string();
                self.clock_date = Local::now().format("%d.%m.%Y").to_string();
            }

            DesktopMessage::Noop => {}
        }

        // When the active app changes, update help context.
        if let Some(app) = self.active_app {
            self.help_sidebar
                .on_active_window_changed(Some(app.name().to_lowercase().as_str()));
        }

        Task::none()
    }

    // ── Sidebar animation helpers ─────────────────────────────────────────────

    /// Target pixel width for the left sidebar (what the animation lerps toward).
    #[allow(clippy::cast_precision_loss)]
    fn left_sidebar_target(&self) -> f32 {
        let full = self
            .shell_layout
            .sidebars_on_side(SidebarSide::Left)
            .first()
            .map_or(220, |s| s.size) as f32;
        match self.left_sidebar_state {
            SidebarState::Open | SidebarState::Expanding => full,
            SidebarState::Collapsed => SidebarState::COLLAPSED_WIDTH as f32,
        }
    }

    /// Target pixel width for the right sidebar.
    #[allow(clippy::cast_precision_loss)]
    fn right_sidebar_target(&self) -> f32 {
        let full = self
            .shell_layout
            .sidebars_on_side(SidebarSide::Right)
            .first()
            .map_or(320, |s| s.size) as f32;
        match self.help_sidebar.state {
            SidebarState::Open | SidebarState::Expanding => full,
            SidebarState::Collapsed => SidebarState::COLLAPSED_WIDTH as f32,
        }
    }

    // ── Subscription ──────────────────────────────────────────────────────────

    /// iced subscription: mouse cursor, window resize, animation tick, clock.
    pub fn subscription(&self) -> Subscription<DesktopMessage> {
        use std::time::Duration;

        // Mouse cursor → sidebar proximity check.
        let mouse_sub = event::listen_with(|evt, _status, _id| match evt {
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(DesktopMessage::CursorMoved(position.x))
            }
            iced::Event::Window(window::Event::Resized(size)) => {
                Some(DesktopMessage::WindowResized(size.width))
            }
            _ => None,
        });

        // ~30 fps animation tick for smooth sidebar width lerp.
        let anim_tick =
            time::every(Duration::from_millis(33)).map(|_| DesktopMessage::SidebarAnimTick);

        // Clock update every 30 seconds.
        let clock_tick = time::every(Duration::from_secs(30)).map(|_| DesktopMessage::ClockTick);

        Subscription::batch([mouse_sub, anim_tick, clock_tick])
    }

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

    fn handle_sidebar_select(&mut self, id: &str) {
        let app_id = match id {
            "browser" => Some(AppId::Browser),
            "settings" => Some(AppId::Settings),
            "profile" => Some(AppId::Profile),
            "store" => Some(AppId::Store),
            "lenses" => Some(AppId::Lenses),
            "builder" => Some(AppId::Builder),
            "tasks" => Some(AppId::Tasks),
            "bots" => Some(AppId::Bots),
            "ai" => Some(AppId::Ai),
            "container-app" => Some(AppId::Container),
            "managers" | "managers-folder" => Some(AppId::Managers),
            "help" => Some(AppId::Help),
            _ => None,
        };
        if let Some(app) = app_id {
            self.active_app = Some(app);
        }
    }
}

// ── view() ────────────────────────────────────────────────────────────────────

#[cfg(feature = "iced")]
impl DesktopShell {
    /// Active color palette (dark or light mode).
    fn palette(&self) -> Palette {
        if self.dark_mode {
            Palette::dark()
        } else {
            Palette::light()
        }
    }

    /// Chrome container style (header / sidebars / taskbar).
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
    #[must_use]
    pub fn view(&self) -> Element<'_, DesktopMessage> {
        if self.launcher_state.open {
            return self.view_launcher();
        }

        let header = self.view_header();
        let left_sidebar = self.view_left_sidebar();
        let right_sidebar = self.view_right_sidebar();
        let content = self.view_content();
        let taskbar = self.view_taskbar();

        let main_row = row![left_sidebar, content, right_sidebar]
            .spacing(0)
            .height(Length::Fill);

        let shell = column![header, main_row, taskbar]
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill);

        let p = self.palette();
        container(shell)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_content)),
                ..container::Style::default()
            })
            .into()
    }

    fn view_header(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();

        let brand = text("FreeSynergy").size(15).color(p.cyan);

        let by_kal = text(" by KalEl").size(11).color(p.muted);

        let brand_row = row![brand, by_kal].align_y(Alignment::Center);

        let menu_bar = Self::view_menu_bar();

        // Theme toggle button
        let theme_icon = if self.dark_mode { "☀" } else { "🌙" };
        let theme_btn = button(text(theme_icon).size(14))
            .on_press(DesktopMessage::ToggleTheme)
            .padding([4, 8]);

        let clock = column![
            text(&self.clock_time).size(13),
            text(&self.clock_date).size(10).color(p.muted),
        ]
        .align_x(Alignment::Center);

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
            theme_btn,
            Space::with_width(4),
            bell_btn,
            Space::with_width(8),
            avatar_btn,
            Space::with_width(12),
            clock,
            Space::with_width(8),
        ]
        .align_y(Alignment::Center)
        .spacing(4)
        .padding([0, 8])
        .height(60);

        container(header_row)
            .width(Length::Fill)
            .height(60)
            .style(move |_| Self::chrome_style(&p, p.border_accent))
            .into()
    }

    fn view_menu_bar() -> Element<'static, DesktopMessage> {
        let menus = default_menu();
        // Collect owned labels first to avoid borrow-of-temporary issues.
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

    // ── Left sidebar (Taskbar) ────────────────────────────────────────────────

    fn view_left_sidebar(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();
        // Use animated width (smooth lerp) — show labels when more than halfway open
        let anim_w = self.left_anim_width;
        let full_width = self
            .shell_layout
            .sidebars_on_side(SidebarSide::Left)
            .first()
            .map_or(220, |s| s.size);
        #[allow(clippy::cast_precision_loss)]
        let show_labels = anim_w > (full_width as f32) * 0.5;
        #[allow(clippy::cast_precision_loss)]
        let collapsed_w = SidebarState::COLLAPSED_WIDTH as f32;

        // Pin toggle button
        let pin_icon = match self.left_sidebar_mode {
            SidebarMode::Pinned => "📌",
            SidebarMode::Auto => "📍",
        };
        let pin_btn: Element<'_, DesktopMessage> = if show_labels {
            button(text(pin_icon).size(11))
                .on_press(DesktopMessage::LeftSidebarTogglePin)
                .padding([2, 6])
                .into()
        } else {
            Space::with_height(0).into()
        };

        // Launcher button
        let launcher_inner: Element<'_, DesktopMessage> = if show_labels {
            row![
                text("⊞").size(16),
                Space::with_width(6),
                text(tr("shell-launcher-title")).size(13),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            container(text("⊞").size(18)).center_x(Length::Fill).into()
        };
        let launcher_btn = button(launcher_inner)
            .on_press(DesktopMessage::LauncherToggle)
            .width(Length::Fill)
            .padding([8, if show_labels { 12 } else { 0 }]);

        let header_row = row![launcher_btn, pin_btn]
            .spacing(0)
            .align_y(Alignment::Center);

        let mut items_col: Vec<Element<'_, DesktopMessage>> = vec![header_row.into()];

        if show_labels && !self.sidebar_sections.is_empty() {
            items_col.push(
                text(tr("taskbar-installed-section"))
                    .size(9)
                    .color(p.muted)
                    .into(),
            );
        }

        for section in &self.sidebar_sections {
            if show_labels {
                if let Some(title) = &section.title {
                    items_col.push(text(title).size(9).color(p.muted).into());
                }
            }
            for item in &section.items {
                items_col.push(self.view_sidebar_item(item, show_labels));
            }
        }

        let scrollable_section =
            scrollable(column(items_col).spacing(1).padding([8, 4])).height(Length::Fill);

        // Separator + pinned
        let separator = container(Space::with_height(1))
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.border)),
                ..container::Style::default()
            });

        let mut pinned_col: Vec<Element<'_, DesktopMessage>> = vec![separator.into()];

        if show_labels && self.pinned_items.iter().any(|p| p.id != "settings") {
            pinned_col.push(
                text(tr("taskbar-pinned-section"))
                    .size(9)
                    .color(p.muted)
                    .into(),
            );
        }

        for item in &self.pinned_items {
            if item.id == "settings" {
                continue;
            }
            pinned_col.push(self.view_sidebar_item(item, show_labels));
        }

        // Settings icon at bottom
        let settings_handle = svg_icon(crate::icons::ICON_SETTINGS, 16.0, p.icon_color);
        let settings_inner: Element<'_, DesktopMessage> = if show_labels {
            row![
                svg(settings_handle).width(16).height(16),
                Space::with_width(8),
                text(tr("taskbar-settings-label")).size(13),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            container(svg(settings_handle).width(18).height(18))
                .center_x(Length::Fill)
                .into()
        };
        let settings_btn = button(settings_inner)
            .on_press(DesktopMessage::SidebarSelect("settings".into()))
            .width(Length::Fill)
            .padding([6, if show_labels { 12 } else { 0 }]);
        pinned_col.push(settings_btn.into());

        let pinned_section = column(pinned_col).spacing(1).padding([4, 4]);

        let sidebar_inner = column![scrollable_section, pinned_section].height(Length::Fill);

        container(sidebar_inner)
            .width(pxf(anim_w.max(collapsed_w)))
            .height(Length::Fill)
            .style(move |_| Self::chrome_style(&p, p.border_accent))
            .into()
    }

    // ── Right sidebar (Help + AI) ─────────────────────────────────────────────

    fn view_right_sidebar(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();
        let anim_w = self.right_anim_width;
        let full_width = self
            .shell_layout
            .sidebars_on_side(SidebarSide::Right)
            .first()
            .map_or(320, |s| s.size);
        #[allow(clippy::cast_precision_loss)]
        let show_content = anim_w > (full_width as f32) * 0.5;
        #[allow(clippy::cast_precision_loss)]
        let collapsed_w = SidebarState::COLLAPSED_WIDTH as f32;

        // Collapsed strip: just a "?" button
        if !show_content {
            let help_handle = svg_icon(crate::icons::ICON_HELP, 18.0, p.icon_color);
            let help_btn = button(
                container(svg(help_handle).width(18).height(18))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .on_press(DesktopMessage::RightSidebarTogglePin)
            .width(Length::Fill)
            .padding([10, 0]);

            return container(help_btn)
                .width(pxf(anim_w.max(collapsed_w)))
                .height(Length::Fill)
                .style(move |_| Self::chrome_style(&p, p.border_accent))
                .into();
        }

        // Expanded: title + content + optional AI input
        let pin_icon = match self.help_sidebar.mode {
            SidebarMode::Pinned => "📌",
            SidebarMode::Auto => "📍",
        };
        let pin_btn = button(text(pin_icon).size(11))
            .on_press(DesktopMessage::RightSidebarTogglePin)
            .padding([2, 6]);

        let title_row = row![
            text(tr("help-sidebar-title")).size(13).color(p.cyan),
            Space::with_width(Length::Fill),
            pin_btn,
        ]
        .align_y(Alignment::Center)
        .padding([4, 8]);

        let content_area = self.view_help_content();

        let mut col_items: Vec<Element<'_, DesktopMessage>> = vec![title_row.into(), content_area];

        // AI input bar (only when AI capability is present)
        if self.help_sidebar.ai_available {
            let ai_input = text_input(&tr("help-ai-placeholder"), &self.help_sidebar.ai_input)
                .on_input(DesktopMessage::HelpAiInputChanged)
                .on_submit(DesktopMessage::HelpAiSend)
                .padding([6, 8])
                .size(12)
                .width(Length::Fill);

            let send_btn = button(text(tr("help-ai-send")).size(12))
                .on_press(DesktopMessage::HelpAiSend)
                .padding([6, 10]);

            let ai_bar = row![ai_input, send_btn]
                .spacing(4)
                .padding([4, 8])
                .align_y(Alignment::Center);

            col_items.push(
                container(ai_bar)
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.08, 0.08, 0.12, 0.96,
                        ))),
                        ..container::Style::default()
                    })
                    .into(),
            );
        }

        container(column(col_items).spacing(0).height(Length::Fill))
            .width(pxf(anim_w.max(collapsed_w)))
            .height(Length::Fill)
            .style(move |_| Self::chrome_style(&p, p.border_accent))
            .into()
    }

    fn view_help_content(&self) -> Element<'_, DesktopMessage> {
        use crate::help_sidebar::HelpContent;
        let p = self.palette();

        let inner: Element<'_, DesktopMessage> = match &self.help_sidebar.content {
            HelpContent::Topic {
                title_key,
                content_key,
                links,
            } => {
                let mut items: Vec<Element<'_, DesktopMessage>> = vec![
                    text(fs_i18n::t(title_key).to_string())
                        .size(14)
                        .color(p.cyan)
                        .into(),
                    Space::with_height(8).into(),
                    text(fs_i18n::t(content_key).to_string()).size(12).into(),
                ];
                if !links.is_empty() {
                    items.push(Space::with_height(8).into());
                    for link in links {
                        items.push(text(link).size(11).color(p.cyan).into());
                    }
                }
                column(items).spacing(2).into()
            }
            HelpContent::AiResponse(resp) => column![
                text(tr("help-ai-response-label")).size(11).color(p.muted),
                Space::with_height(4),
                text(resp.clone()).size(12),
            ]
            .spacing(2)
            .into(),
            HelpContent::None => text(tr("help-no-content")).size(12).color(p.muted).into(),
        };

        scrollable(container(inner).padding([8, 12]).width(Length::Fill))
            .height(Length::Fill)
            .into()
    }

    /// Render one sidebar item with SVG icon.
    ///
    /// `show_labels` — when `false` only the icon is shown (collapsed/animating state).
    fn view_sidebar_item<'a>(
        &'a self,
        item: &'a SidebarItem,
        show_labels: bool,
    ) -> Element<'a, DesktopMessage> {
        let p = self.palette();
        let is_active = self
            .active_app
            .is_some_and(|a| a.name().to_lowercase() == item.id);

        let is_pinned = self.pinned_items.iter().any(|p| p.id == item.id);
        let id = item.id.clone();

        // Build icon: SVG if the string looks like SVG, else Unicode text fallback.
        let icon_el: Element<'_, DesktopMessage> = if item.icon.starts_with('<') {
            let handle = svg_icon(&item.icon, 16.0, p.icon_color);
            svg(handle).width(16).height(16).into()
        } else {
            text(item.icon.clone()).size(16).into()
        };

        let btn_content: Element<'_, DesktopMessage> = if show_labels {
            row![
                icon_el,
                Space::with_width(8),
                text(item.label.clone()).size(13),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            container(icon_el).center_x(Length::Fill).into()
        };

        let app_btn = button(btn_content)
            .on_press(DesktopMessage::SidebarSelect(id.clone()))
            .width(Length::Fill)
            .padding([6, if show_labels { 12 } else { 0 }]);

        let pin_btn: Element<'_, DesktopMessage> = if show_labels && item.id != "settings" {
            let (pin_icon, pin_msg): (&str, DesktopMessage) = if is_pinned {
                ("📌", DesktopMessage::UnpinApp(id.clone()))
            } else {
                ("📍", DesktopMessage::PinApp(id.clone()))
            };
            button(text(pin_icon).size(11))
                .on_press(pin_msg)
                .padding([6, 4])
                .into()
        } else {
            Space::with_width(0).into()
        };

        let item_row = row![app_btn, pin_btn].spacing(0).align_y(Alignment::Center);

        if is_active {
            container(item_row)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.active_bg)),
                    border: Border {
                        color: p.active_border,
                        width: 2.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.02, 0.74, 0.84, 0.20),
                        offset: Vector::new(0.0, 0.0),
                        blur_radius: 8.0,
                    },
                    ..container::Style::default()
                })
                .into()
        } else {
            item_row.into()
        }
    }

    fn view_content(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();

        let content: Element<'_, DesktopMessage> = match self.active_app {
            Some(app_id) => {
                let icon_el: Element<'_, DesktopMessage> = if app_id.icon().starts_with('<') {
                    let handle = svg_icon(app_id.icon(), 48.0, p.icon_color);
                    svg(handle).width(48).height(48).into()
                } else {
                    text(app_id.icon()).size(48).into()
                };
                container(
                    column![
                        icon_el,
                        Space::with_height(16),
                        text(app_id.name()).size(20).color(p.cyan),
                        Space::with_height(8),
                        text(tr("shell-app-opening")).size(14).color(p.muted),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(8),
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
                            text("⊞").size(16),
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

    fn view_taskbar(&self) -> Element<'_, DesktopMessage> {
        let p = self.palette();

        let launcher_btn = button(text("⊞").size(20))
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
                .on_press(DesktopMessage::SidebarSelect(id))
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

    fn view_launcher(&self) -> Element<'_, DesktopMessage> {
        let query = self.launcher_state.query.clone();
        let groups = AppGroup::filtered(&self.taskbar_apps, &query);
        let total_pages = AppGroup::total_pages(&groups);
        let cur_page = self.launcher_state.page.min(total_pages - 1);
        // Clone the page slice to avoid borrow-of-local issues when building elements.
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
                    text(tr_with(
                        "shell-launcher-no-apps",
                        &[("query", query.as_str())],
                    ))
                    .size(14)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.7)),
                )
                .center_x(Length::Fill)
                .padding(48)
                .into(),
            );
        } else {
            for group in page_groups {
                // Move group fields to avoid borrow-of-local-variable issues.
                let group_label = group.label.clone();
                let label = text(group_label)
                    .size(11)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.6));

                let tiles: Vec<Element<'_, DesktopMessage>> = group
                    .apps
                    .into_iter()
                    .map(|app| {
                        let id = app.id.clone();
                        let icon = if app.icon.starts_with('<') {
                            "●".to_string()
                        } else {
                            app.icon.clone()
                        };
                        let label_key = app.label_key.clone();
                        button(
                            column![text(icon).size(28), text(label_key).size(11),]
                                .align_x(Alignment::Center)
                                .spacing(4),
                        )
                        .on_press(DesktopMessage::LauncherLaunch(id))
                        .padding([12, 8])
                        .width(100)
                        .into()
                    })
                    .collect();

                let tile_row = row(tiles).spacing(8).wrap();

                group_items.push(
                    column![label, Space::with_height(4), tile_row]
                        .spacing(0)
                        .padding([8, 12])
                        .into(),
                );
            }
        }

        let mut pagination: Vec<Element<'_, DesktopMessage>> = vec![];
        if total_pages > 1 {
            let prev_btn = button(text("◄").size(13))
                .on_press(DesktopMessage::LauncherPrevPage)
                .padding([2, 10]);
            let next_btn = button(text("►").size(13))
                .on_press(DesktopMessage::LauncherNextPage)
                .padding([2, 10]);
            let page_label = text(format!("{} / {}", cur_page + 1, total_pages)).size(12);

            pagination.push(prev_btn.into());
            pagination.push(page_label.into());
            for i in 0..total_pages {
                let dot = button(text(if i == cur_page { "●" } else { "○" }).size(8))
                    .on_press(DesktopMessage::LauncherGotoPage(i))
                    .padding([2, 4]);
                pagination.push(dot.into());
            }
            pagination.push(next_btn.into());
        }

        let close_btn = button(text(tr("actions.close")).size(13))
            .on_press(DesktopMessage::LauncherClose)
            .padding([6, 16]);

        let panel = column![
            text("FreeSynergy")
                .size(18)
                .color(iced::Color::from_rgb(0.02, 0.74, 0.84)),
            Space::with_height(16),
            search,
            Space::with_height(12),
            scrollable(column(group_items).spacing(8)).height(400),
            Space::with_height(8),
            row(pagination).spacing(4).align_y(Alignment::Center),
            Space::with_height(8),
            close_btn,
        ]
        .spacing(0)
        .padding(24)
        .width(700)
        .max_width(800);

        container(panel)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.85,
                ))),
                ..container::Style::default()
            })
            .into()
    }
}

// ── Non-iced stub ─────────────────────────────────────────────────────────────

#[cfg(not(feature = "iced"))]
impl DesktopShell {
    pub fn update(&mut self, _msg: DesktopMessage) {}
    pub fn view(&self) {}
}
