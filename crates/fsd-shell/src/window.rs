/// Window system — all views are FsnObject windows.
///
/// Design:
/// - `Window` — data struct for a single open window
/// - `WindowManager` — manages all open windows, ordering, visibility
/// - `WindowSidebarItem` — a sidebar navigation item (icon + label)
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an open window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

impl WindowId {
    pub fn next() -> Self {
        Self(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// The initial size of a window.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowSize {
    Fixed { width: f64, height: f64 },
    Responsive { min_width: f64, max_width: f64 },
    Fullscreen,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self::Responsive { min_width: 400.0, max_width: 900.0 }
    }
}

impl WindowSize {
    /// Returns (initial_width, initial_height) in pixels.
    pub fn initial_dimensions(&self) -> (f64, f64) {
        match self {
            Self::Fixed { width, height } => (*width, *height),
            Self::Responsive { min_width, max_width } => ((*min_width + *max_width) / 2.0, 600.0),
            Self::Fullscreen => (0.0, 0.0),
        }
    }
}

/// Standard window footer buttons.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowButton {
    Ok,
    Cancel,
    Apply,
    Custom { label_key: String, action_id: String },
}

/// A sidebar navigation item within a window.
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

/// Trait that all window contents implement.
pub trait WindowContent: Send + Sync + 'static {
    fn title_key(&self) -> &str;
    fn help_topic(&self) -> Option<&str> { None }
}

/// An open window in the desktop environment.
#[derive(Clone, PartialEq)]
pub struct Window {
    pub id:                  WindowId,
    pub title_key:           String,
    pub closable:            bool,
    pub buttons:             Vec<WindowButton>,
    pub size:                WindowSize,
    pub scrollable:          bool,
    pub help_topic:          Option<String>,
    pub z_index:             u32,
    pub visible:             bool,
    /// Minimized: hidden from window area, shown as icon on desktop.
    pub minimized:           bool,
    /// Maximized: fills full window area.
    pub maximized:           bool,
    /// Sidebar items — if empty, no sidebar is rendered.
    pub sidebar_items:       Vec<WindowSidebarItem>,
    /// Currently active sidebar item id.
    pub active_sidebar_id:   Option<String>,
    /// Whether this window has unsaved changes (triggers confirm dialog on close).
    pub has_unsaved_changes: bool,
    /// Icon shown when minimized (emoji or text).
    pub icon:                String,
    /// Virtual desktop this window lives on (0-indexed).
    pub desktop_index:       usize,
}

impl Window {
    pub fn new(title_key: impl Into<String>) -> Self {
        Self {
            id:                  WindowId::next(),
            title_key:           title_key.into(),
            closable:            true,
            buttons:             vec![WindowButton::Ok, WindowButton::Cancel],
            size:                WindowSize::default(),
            scrollable:          true,
            help_topic:          None,
            z_index:             0,
            visible:             true,
            minimized:           false,
            maximized:           false,
            sidebar_items:       vec![],
            active_sidebar_id:   None,
            has_unsaved_changes: false,
            icon:                "🗗".to_string(),
            desktop_index:       0,
        }
    }

    pub fn with_size(mut self, size: WindowSize) -> Self { self.size = size; self }
    pub fn with_buttons(mut self, buttons: Vec<WindowButton>) -> Self { self.buttons = buttons; self }
    pub fn with_help(mut self, topic: impl Into<String>) -> Self { self.help_topic = Some(topic.into()); self }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self { self.icon = icon.into(); self }
    pub fn with_sidebar(mut self, items: Vec<WindowSidebarItem>) -> Self { self.sidebar_items = items; self }
    pub fn with_desktop(mut self, index: usize) -> Self { self.desktop_index = index; self }
}

/// Manages all open windows — ordering and visibility.
#[derive(Default, Clone)]
pub struct WindowManager {
    windows: Vec<Window>,
}

impl WindowManager {
    pub fn open(&mut self, window: Window) {
        let z = self.windows.len() as u32;
        let mut w = window;
        w.z_index = z;
        self.windows.push(w);
    }

    pub fn close(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
    }

    /// Bring window to front. Also restores a minimized window.
    pub fn focus(&mut self, id: WindowId) {
        let max_z = self.windows.len() as u32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.z_index = max_z;
            w.minimized = false;
        }
    }

    /// Minimize window to desktop icon.
    pub fn minimize(&mut self, id: WindowId) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.minimized = true;
            w.maximized = false;
        }
    }

    /// Toggle maximized state.
    pub fn maximize(&mut self, id: WindowId) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.maximized = !w.maximized;
            if w.maximized { w.minimized = false; }
        }
    }

    /// Set active sidebar item for a window.
    pub fn set_sidebar_active(&mut self, id: WindowId, item_id: String) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.active_sidebar_id = Some(item_id);
        }
    }

    /// Mark a window as having unsaved changes.
    pub fn set_unsaved(&mut self, id: WindowId, unsaved: bool) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.has_unsaved_changes = unsaved;
        }
    }

    pub fn windows(&self) -> &[Window] { &self.windows }
    pub fn is_open(&self, id: WindowId) -> bool { self.windows.iter().any(|w| w.id == id) }
    pub fn minimized_windows(&self) -> Vec<&Window> { self.windows.iter().filter(|w| w.minimized).collect() }
    pub fn visible_windows(&self) -> Vec<&Window> { self.windows.iter().filter(|w| !w.minimized).collect() }
}
