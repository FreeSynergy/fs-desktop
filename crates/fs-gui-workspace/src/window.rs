/// Window system — all views are FsObject windows.
///
/// Design:
/// - `WindowId`       — unique identifier
/// - `WindowSize`     — initial size spec
/// - `WindowButton`   — footer button kinds
/// - `WindowSidebarItem` — sidebar navigation entry
/// - `SidebarSlot`    — Left / Right / Hidden position for a panel
/// - `WindowLayout`   — configurable bar positions (which bar goes where)
/// - `Window`         — runtime state of one open window
/// - `FsWindow`       — trait: anything that can describe itself as a window
/// - `WindowRenderFn` — type-erased zero-arg Dioxus component for the content
/// - `OpenWindow`     — Window + render fn (a live, renderable window)
/// - `WindowHost`     — trait: container that manages child windows
/// - `WindowManager`  — concrete WindowHost (used by Desktop and standalone hosts)
///
/// The Desktop implements WindowHost. Each app implements FsWindow.
/// A standalone app wraps itself in a single-window host — same chrome, same path.
///
/// # Layout model
///
/// Every Window has a `layout: WindowLayout` that controls which side each
/// panel lives on. If a panel slot has no assigned component it becomes
/// invisible automatically — the chrome collapses around what is present.
///
/// The user can change the layout in Desktop settings (e.g. "help left or right?",
/// "main sidebar left or right?"). The layout is stored per-user, not per-window.
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use dioxus::prelude::Element;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

/// Monotonically increasing z-index counter.
/// Every `open_window` and `focus_window` call gets a unique, increasing value,
/// so the most recently opened/focused window always has the highest base z_index.
static WINDOW_Z: AtomicU32 = AtomicU32::new(1);

fn next_z() -> u32 {
    WINDOW_Z.fetch_add(1, Ordering::Relaxed)
}

// ── Identity ──────────────────────────────────────────────────────────────────

/// Unique identifier for an open window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

impl WindowId {
    pub fn next() -> Self { Self(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed)) }
}

// ── Size ──────────────────────────────────────────────────────────────────────

/// Initial size specification for a window.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowSize {
    Fixed      { width: f64, height: f64 },
    Responsive { min_width: f64, max_width: f64 },
    Fullscreen,
}

impl Default for WindowSize {
    fn default() -> Self { Self::Responsive { min_width: 400.0, max_width: 900.0 } }
}

impl WindowSize {
    /// Returns (initial_width, initial_height) in pixels.
    pub fn initial_dimensions(&self) -> (f64, f64) {
        match self {
            Self::Fixed { width, height }             => (*width, *height),
            Self::Responsive { min_width, max_width } => ((min_width + max_width) / 2.0, 600.0),
            Self::Fullscreen                          => (0.0, 0.0),
        }
    }
}

// ── SidebarSlot ───────────────────────────────────────────────────────────────

/// Where a panel is placed inside a Window.
///
/// Used by [`WindowLayout`] to configure which side the main sidebar, the help
/// panel, and other bars live on.  A panel whose slot is `Hidden` (or has no
/// component assigned) is invisible — the chrome collapses automatically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SidebarSlot {
    /// Panel appears on the left side of the content area.
    #[default]
    Left,
    /// Panel appears on the right side of the content area.
    Right,
    /// Panel is not shown.
    Hidden,
}

// ── WindowLayout ──────────────────────────────────────────────────────────────

/// Configurable chrome layout for a Window.
///
/// Stored in user settings; applied globally to all windows.
/// Each panel is optional — if it has no component it collapses to nothing.
///
/// # Defaults
///
/// - Main navigation sidebar: **Left**
/// - Help panel: **Right**
/// - Top tab bar: **visible**
/// - Bottom bar: **hidden** (reserved for future use)
#[derive(Clone, Debug, PartialEq)]
pub struct WindowLayout {
    /// Where the main navigation sidebar lives.
    pub main_sidebar: SidebarSlot,

    /// Where the context-sensitive help panel lives.
    pub help_panel: SidebarSlot,

    /// Whether the top category tab bar is shown.
    pub show_tabbar: bool,

    /// Whether a bottom bar slot is shown (future extension point).
    pub show_bottom: bool,
}

impl Default for WindowLayout {
    fn default() -> Self { Self::standard() }
}

impl WindowLayout {
    /// Standard layout: main sidebar left, help panel right, tabbar visible.
    pub fn standard() -> Self {
        Self {
            main_sidebar: SidebarSlot::Left,
            help_panel:   SidebarSlot::Right,
            show_tabbar:  true,
            show_bottom:  false,
        }
    }

    /// Mirrored layout: main sidebar right, help panel left.
    pub fn mirrored() -> Self {
        Self {
            main_sidebar: SidebarSlot::Right,
            help_panel:   SidebarSlot::Left,
            show_tabbar:  true,
            show_bottom:  false,
        }
    }

    /// Minimal layout: both sidebars hidden, no tab bar.
    pub fn minimal() -> Self {
        Self {
            main_sidebar: SidebarSlot::Hidden,
            help_panel:   SidebarSlot::Hidden,
            show_tabbar:  false,
            show_bottom:  false,
        }
    }

    /// Returns `true` when a left-side panel will be rendered.
    pub fn has_left_panel(&self) -> bool {
        self.main_sidebar == SidebarSlot::Left || self.help_panel == SidebarSlot::Left
    }

    /// Returns `true` when a right-side panel will be rendered.
    pub fn has_right_panel(&self) -> bool {
        self.main_sidebar == SidebarSlot::Right || self.help_panel == SidebarSlot::Right
    }
}

// ── Buttons ───────────────────────────────────────────────────────────────────

/// Standard footer buttons.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowButton {
    Ok,
    Cancel,
    Apply,
    Custom { label_key: String, action_id: String },
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

/// One entry in a window's left-side navigation sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowSidebarItem {
    pub id:    String,
    pub icon:  String,
    pub label: String,
}

impl WindowSidebarItem {
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), icon: icon.into(), label: label.into() }
    }
}

// ── FsWindow trait ────────────────────────────────────────────────────────────

/// Trait for anything that can describe itself as a window.
///
/// Implement this on app state structs, dialog descriptors, and the Desktop
/// itself — everything is a window, everything plays by the same rules.
///
/// Only `title_key` must be provided; all other methods have sane defaults.
/// Use `into_window()` to materialise the description into a live `Window`.
pub trait FsWindow {
    fn title_key(&self) -> &str;

    fn icon(&self) -> &str                          { "🗗" }
    fn default_size(&self) -> WindowSize            { WindowSize::default() }
    fn sidebar_items(&self) -> Vec<WindowSidebarItem> { vec![] }
    fn footer_buttons(&self) -> Vec<WindowButton>   { vec![WindowButton::Ok, WindowButton::Cancel] }
    fn help_topic(&self) -> Option<&str>            { None }
    fn closable(&self) -> bool                      { true }
    fn scrollable(&self) -> bool                    { true }
    fn desktop_index(&self) -> usize                { 0 }

    /// Override the default [`WindowLayout`] for this window type.
    ///
    /// The default is [`WindowLayout::standard()`] (main sidebar left, help panel right).
    /// Override this for apps that want a different panel arrangement by default,
    /// but note that the user's global layout preference takes precedence.
    fn default_layout(&self) -> WindowLayout { WindowLayout::standard() }

    /// Build a `Window` (runtime state) from this description.
    fn into_window(self) -> Window where Self: Sized {
        Window {
            id:                  WindowId::next(),
            title_key:           self.title_key().to_string(),
            icon:                self.icon().to_string(),
            size:                self.default_size(),
            sidebar_items:       self.sidebar_items(),
            buttons:             self.footer_buttons(),
            help_topic:          self.help_topic().map(str::to_string),
            closable:            self.closable(),
            scrollable:          self.scrollable(),
            layout:              self.default_layout(),
            z_index:             0,
            visible:             true,
            minimized:           false,
            maximized:           false,
            active_sidebar_id:   None,
            active_tab:          None,
            has_unsaved_changes: false,
            desktop_index:       self.desktop_index(),
        }
    }
}

// ── Window (runtime state) ────────────────────────────────────────────────────

/// Runtime state of one open window.
///
/// Static properties (title, icon, sidebar, …) come from `FsWindow::into_window()`.
/// Dynamic state (z-order, position, minimized, …) is mutated at runtime by `WindowHost`.
///
/// # Title bar structure (always)
///
/// ```text
/// ┌─────────────────────────────────────────────────┐
/// │ [icon]      Package Name (centered)    [─][□][×] │
/// ├──────────┬──────────────────────────┬────────────┤
/// │ main     │   [tab bar (optional)]   │  help      │
/// │ sidebar  │   content area           │  panel     │
/// │ (left or │                          │  (left or  │
/// │  right)  │                          │   right)   │
/// └──────────┴──────────────────────────┴────────────┘
/// ```
///
/// Panels that have no component assigned become invisible automatically.
#[derive(Clone, PartialEq)]
pub struct Window {
    pub id:                  WindowId,
    pub title_key:           String,
    pub icon:                String,
    pub size:                WindowSize,
    pub sidebar_items:       Vec<WindowSidebarItem>,
    pub buttons:             Vec<WindowButton>,
    pub help_topic:          Option<String>,
    pub closable:            bool,
    pub scrollable:          bool,
    /// Configurable panel layout — which side each bar lives on.
    pub layout:              WindowLayout,
    // ── runtime ──────────────────────────────────────────────────────────────
    pub z_index:             u32,
    pub visible:             bool,
    pub minimized:           bool,
    pub maximized:           bool,
    pub active_sidebar_id:   Option<String>,
    /// Active tab in the top tab bar (if shown).
    pub active_tab:          Option<String>,
    pub has_unsaved_changes: bool,
    /// Virtual desktop this window lives on (0-indexed).
    pub desktop_index:       usize,
}

impl Window {
    /// Quick constructor — all runtime state initialised to sane defaults.
    pub fn new(title_key: impl Into<String>) -> Self {
        Self {
            id:                  WindowId::next(),
            title_key:           title_key.into(),
            icon:                "🗗".to_string(),
            size:                WindowSize::default(),
            sidebar_items:       vec![],
            buttons:             vec![WindowButton::Ok, WindowButton::Cancel],
            help_topic:          None,
            closable:            true,
            scrollable:          true,
            layout:              WindowLayout::standard(),
            z_index:             0,
            visible:             true,
            minimized:           false,
            maximized:           false,
            active_sidebar_id:   None,
            active_tab:          None,
            has_unsaved_changes: false,
            desktop_index:       0,
        }
    }

    pub fn with_size(mut self, size: WindowSize) -> Self              { self.size = size; self }
    pub fn with_buttons(mut self, b: Vec<WindowButton>) -> Self       { self.buttons = b; self }
    pub fn with_help(mut self, t: impl Into<String>) -> Self          { self.help_topic = Some(t.into()); self }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self       { self.icon = icon.into(); self }
    pub fn with_sidebar(mut self, items: Vec<WindowSidebarItem>) -> Self { self.sidebar_items = items; self }
    pub fn with_desktop(mut self, index: usize) -> Self               { self.desktop_index = index; self }
    /// Override the default chrome layout for this window.
    pub fn with_layout(mut self, layout: WindowLayout) -> Self        { self.layout = layout; self }
}

// ── OpenWindow ────────────────────────────────────────────────────────────────

/// A zero-arg Dioxus component function used as the window's content renderer.
///
/// Must be a plain `fn() -> Element` (not a closure) — function pointers are
/// `Copy + PartialEq + 'static`, which lets Dioxus memoize `WindowContent`.
pub type WindowRenderFn = fn() -> Element;

/// A live, renderable window: runtime metadata + type-erased render function.
///
/// `Deref<Target = Window>` lets all existing callers access metadata fields
/// directly without wrapping every access in `.meta.xxx`.
///
/// # Design
/// The Desktop stores `Vec<OpenWindow>` in its `WindowManager`.
/// When running standalone, a single-window host does the same with one entry.
/// Both share the same `WindowFrame` rendering path — from one mould.
#[derive(Clone)]
pub struct OpenWindow {
    pub meta:   Window,
    pub render: WindowRenderFn,
}

impl PartialEq for OpenWindow {
    fn eq(&self, other: &Self) -> bool {
        self.meta == other.meta
            && core::ptr::fn_addr_eq(self.render as fn() -> _, other.render as fn() -> _)
    }
}

impl OpenWindow {
    pub fn new(meta: Window, render: WindowRenderFn) -> Self {
        Self { meta, render }
    }

    /// Build from a `FsWindow` description + render function.
    pub fn from_fs<W: FsWindow>(desc: W, render: WindowRenderFn) -> Self {
        Self { meta: desc.into_window(), render }
    }
}

impl std::ops::Deref for OpenWindow {
    type Target = Window;
    fn deref(&self) -> &Window { &self.meta }
}

impl std::ops::DerefMut for OpenWindow {
    fn deref_mut(&mut self) -> &mut Window { &mut self.meta }
}

// ── WindowHost trait ──────────────────────────────────────────────────────────

/// Implemented by containers that manage a collection of open windows.
///
/// Both the `WindowManager` (used by the Desktop) and a minimal single-app
/// standalone host implement this trait — the rendering path is identical.
pub trait WindowHost {
    fn open_window(&mut self, window: OpenWindow);
    fn close_window(&mut self, id: WindowId);
    fn focus_window(&mut self, id: WindowId);
    fn minimize_window(&mut self, id: WindowId);
    fn maximize_window(&mut self, id: WindowId);
    fn set_sidebar_active(&mut self, id: WindowId, item_id: String);
    fn set_unsaved(&mut self, id: WindowId, unsaved: bool);
    fn open_windows(&self) -> &[OpenWindow];
    fn is_window_open(&self, id: WindowId) -> bool;
}

// ── WindowManager ─────────────────────────────────────────────────────────────

/// Concrete `WindowHost` — manages all open windows, z-ordering, and visibility.
#[derive(Default, Clone)]
pub struct WindowManager {
    windows: Vec<OpenWindow>,
}

impl WindowHost for WindowManager {
    fn open_window(&mut self, window: OpenWindow) {
        let mut w = window;
        w.meta.z_index = next_z();
        self.windows.push(w);
    }

    fn close_window(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
    }

    fn focus_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.meta.z_index   = next_z();
            w.meta.minimized = false;
        }
    }

    fn minimize_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.meta.minimized = true;
            w.meta.maximized = false;
        }
    }

    fn maximize_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.meta.maximized = !w.meta.maximized;
            if w.meta.maximized { w.meta.minimized = false; }
        }
    }

    fn set_sidebar_active(&mut self, id: WindowId, item_id: String) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.meta.active_sidebar_id = Some(item_id);
        }
    }

    fn set_unsaved(&mut self, id: WindowId, unsaved: bool) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.meta.has_unsaved_changes = unsaved;
        }
    }

    fn open_windows(&self) -> &[OpenWindow] { &self.windows }

    fn is_window_open(&self, id: WindowId) -> bool {
        self.windows.iter().any(|w| w.id == id)
    }
}

// Backward-compatible shims — existing call sites continue to compile unchanged.
impl WindowManager {
    pub fn open(&mut self, w: OpenWindow)               { self.open_window(w) }
    pub fn close(&mut self, id: WindowId)               { self.close_window(id) }
    pub fn focus(&mut self, id: WindowId)               { self.focus_window(id) }
    pub fn minimize(&mut self, id: WindowId)            { self.minimize_window(id) }
    pub fn maximize(&mut self, id: WindowId)            { self.maximize_window(id) }
    pub fn set_sidebar_active_compat(&mut self, id: WindowId, item_id: String) {
        self.set_sidebar_active(id, item_id)
    }
    pub fn windows(&self) -> &[OpenWindow]              { self.open_windows() }
    pub fn is_open(&self, id: WindowId) -> bool         { self.is_window_open(id) }
    pub fn minimized_windows(&self) -> Vec<&OpenWindow> { self.windows.iter().filter(|w| w.minimized).collect() }
    pub fn visible_windows(&self) -> Vec<&OpenWindow>   { self.windows.iter().filter(|w| !w.minimized).collect() }
}

// ── Legacy ────────────────────────────────────────────────────────────────────

/// Legacy alias — use `FsWindow` for new code.
pub trait WindowContent: Send + Sync + 'static {
    fn title_key(&self) -> &str;
    fn help_topic(&self) -> Option<&str> { None }
}
