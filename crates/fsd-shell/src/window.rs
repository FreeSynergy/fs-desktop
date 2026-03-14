/// Window system — all dialogs and views are Window objects.
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an open window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

impl WindowId {
    pub fn next() -> Self {
        Self(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// The size of a window.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowSize {
    Fixed { width: u32, height: u32 },
    Responsive { min_width: u32, max_width: u32 },
    Fullscreen,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self::Responsive { min_width: 400, max_width: 900 }
    }
}

/// Standard window buttons.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowButton {
    /// Confirm and close.
    Ok,
    /// Cancel and close.
    Cancel,
    /// Apply without closing.
    Apply,
    /// Custom action button.
    Custom { label_key: String, action_id: String },
}

/// Trait that all window contents implement.
pub trait WindowContent: Send + Sync + 'static {
    fn title_key(&self) -> &str;
    fn help_topic(&self) -> Option<&str> {
        None
    }
}

/// An open window in the desktop environment.
#[derive(Clone, PartialEq)]
pub struct Window {
    pub id: WindowId,
    pub title_key: String,
    pub closable: bool,
    pub buttons: Vec<WindowButton>,
    pub size: WindowSize,
    pub scrollable: bool,
    pub help_topic: Option<String>,
    pub z_index: u32,
    pub visible: bool,
    /// Hidden to taskbar — not rendered, but not closed.
    pub minimized: bool,
    /// Fills the full window area.
    pub maximized: bool,
}

impl Window {
    pub fn new(title_key: impl Into<String>) -> Self {
        Self {
            id: WindowId::next(),
            title_key: title_key.into(),
            closable: true,
            buttons: vec![WindowButton::Ok, WindowButton::Cancel],
            size: WindowSize::default(),
            scrollable: true,
            help_topic: None,
            z_index: 0,
            visible: true,
            minimized: false,
            maximized: false,
        }
    }

    pub fn with_size(mut self, size: WindowSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_buttons(mut self, buttons: Vec<WindowButton>) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn with_help(mut self, topic: impl Into<String>) -> Self {
        self.help_topic = Some(topic.into());
        self
    }
}

/// Manages all open windows — their ordering and visibility.
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

    /// Minimize window to taskbar (hidden but not closed).
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
            if w.maximized {
                w.minimized = false;
            }
        }
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn is_open(&self, id: WindowId) -> bool {
        self.windows.iter().any(|w| w.id == id)
    }
}
