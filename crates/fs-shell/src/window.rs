/// Window system — all views are FsObject windows.
///
/// Design:
/// - `WindowId`       — unique identifier
/// - `WindowSize`     — initial size spec
/// - `WindowButton`   — footer button kinds
/// - `WindowSidebarItem` — sidebar navigation entry
/// - `Window`         — runtime state of one open window
/// - `FsWindow`       — trait: anything that can describe itself as a window
/// - `WindowRenderFn` — type-erased zero-arg Dioxus component for the content
/// - `OpenWindow`     — Window + render fn (a live, renderable window)
/// - `WindowHost`     — trait: container that manages child windows
/// - `WindowManager`  — concrete WindowHost (used by Desktop and standalone hosts)
///
/// The Desktop implements WindowHost. Each app implements FsWindow.
/// A standalone app wraps itself in a single-window host — same chrome, same path.
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::Element;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

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
            z_index:             0,
            visible:             true,
            minimized:           false,
            maximized:           false,
            active_sidebar_id:   None,
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
    // ── runtime ──────────────────────────────────────────────────────────────
    pub z_index:             u32,
    pub visible:             bool,
    pub minimized:           bool,
    pub maximized:           bool,
    pub active_sidebar_id:   Option<String>,
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
            z_index:             0,
            visible:             true,
            minimized:           false,
            maximized:           false,
            active_sidebar_id:   None,
            has_unsaved_changes: false,
            desktop_index:       0,
        }
    }

    pub fn with_size(mut self, size: WindowSize) -> Self           { self.size = size; self }
    pub fn with_buttons(mut self, b: Vec<WindowButton>) -> Self    { self.buttons = b; self }
    pub fn with_help(mut self, t: impl Into<String>) -> Self       { self.help_topic = Some(t.into()); self }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self    { self.icon = icon.into(); self }
    pub fn with_sidebar(mut self, items: Vec<WindowSidebarItem>) -> Self { self.sidebar_items = items; self }
    pub fn with_desktop(mut self, index: usize) -> Self            { self.desktop_index = index; self }
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
        let z = self.windows.len() as u32;
        let mut w = window;
        w.meta.z_index = z;
        self.windows.push(w);
    }

    fn close_window(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
    }

    fn focus_window(&mut self, id: WindowId) {
        let max_z = self.windows.len() as u32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.meta.z_index  = max_z;
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
