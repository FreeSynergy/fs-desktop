// sidebar_state.rs — SidebarState: State Machine for collapsible sidebars.
//
// Design Pattern: State Machine
//   SidebarState:   Collapsed | Expanding | Open
//   SidebarMode:    Auto (collapses when unfocused) | Pinned (stays open)
//   SidebarSide:    Left | Right — used for positioning in the shell layout
//
// Transitions:
//   Collapsed  →[cursor near edge]→  Expanding  →[animation done]→  Open
//   Open       →[cursor far + Auto]→  Collapsed
//   Open       →[Pin button]→  Pinned (stays Open regardless of cursor)
//   Pinned     →[Pin button again]→  Auto (collapses on cursor leaving)
//
// Cursor proximity is measured against a configurable threshold (pixels from edge).
// The shell emits `SidebarCursorMoved` messages with the x coordinate so the
// state machine can decide transitions without depending on iced internals.

use serde::{Deserialize, Serialize};

// ── SidebarState ──────────────────────────────────────────────────────────────

/// Visual + interaction state of a sidebar panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SidebarState {
    /// Only the icon strip is visible (width = `COLLAPSED_WIDTH` px).
    #[default]
    Collapsed,
    /// Cursor near edge — sidebar is animating toward full width.
    Expanding,
    /// Fully open — icons + labels visible (width = configured size).
    Open,
}

impl SidebarState {
    /// Width in logical pixels when collapsed (icon-strip only).
    pub const COLLAPSED_WIDTH: u32 = 48;

    /// Pixel distance from the sidebar edge that triggers expansion.
    pub const PROXIMITY_THRESHOLD: f32 = 16.0;

    /// Returns `true` if labels should be shown (sidebar is fully open).
    #[must_use]
    pub fn show_labels(self) -> bool {
        self == Self::Open
    }

    /// Returns `true` if the sidebar strip is visible at all.
    #[must_use]
    pub fn is_visible(self) -> bool {
        true // icon strip is always visible
    }

    /// Pixel width to use for rendering.
    #[must_use]
    pub fn render_width(self, full_width: u32) -> u32 {
        match self {
            Self::Collapsed => Self::COLLAPSED_WIDTH,
            Self::Expanding => full_width / 2, // mid-animation approximation
            Self::Open => full_width,
        }
    }
}

// ── SidebarMode ───────────────────────────────────────────────────────────────

/// Whether a sidebar auto-collapses on cursor leave or stays pinned open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SidebarMode {
    /// Sidebar collapses when cursor moves away.
    #[default]
    Auto,
    /// Sidebar stays open regardless of cursor position.
    Pinned,
}

// ── SidebarSide ───────────────────────────────────────────────────────────────

/// On which side of the main content area a sidebar is placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarSide {
    /// Sidebar appears on the left of the main content.
    #[default]
    Left,
    /// Sidebar appears on the right of the main content.
    Right,
}

// ── MouseProximityObserver ────────────────────────────────────────────────────

/// Observes cursor X position and fires transition events for a sidebar.
///
/// The shell calls `check(cursor_x, window_width)` on every cursor-moved event.
/// It returns `Some(SidebarTransition)` when a state change should occur.
pub struct MouseProximityObserver {
    /// Which side this observer monitors.
    pub side: SidebarSide,
    /// Current mode (Auto | Pinned).
    pub mode: SidebarMode,
    /// Configured full sidebar width.
    pub full_width: u32,
}

/// Requested state transition from the proximity observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTransition {
    /// Cursor entered proximity zone — start expanding.
    StartExpand,
    /// Sidebar should become fully open.
    Open,
    /// Cursor left proximity zone — collapse.
    Collapse,
}

impl MouseProximityObserver {
    #[must_use]
    pub fn new(side: SidebarSide, full_width: u32) -> Self {
        Self {
            side,
            mode: SidebarMode::Auto,
            full_width,
        }
    }

    /// Check cursor position and return a transition if one is needed.
    ///
    /// `cursor_x`:    current cursor X coordinate (logical pixels from left).
    /// `window_width`: total window width (logical pixels).
    /// `current`:     the current sidebar state.
    #[must_use]
    pub fn check(
        &self,
        cursor_x: f32,
        window_width: f32,
        current: SidebarState,
    ) -> Option<SidebarTransition> {
        if self.mode == SidebarMode::Pinned {
            return None;
        }

        let threshold = SidebarState::PROXIMITY_THRESHOLD;
        #[allow(clippy::cast_precision_loss)]
        let full_w = self.full_width as f32;
        #[allow(clippy::cast_precision_loss)]
        let collapsed_w = SidebarState::COLLAPSED_WIDTH as f32;

        // Hysteresis: expand triggers only when cursor is near the collapsed strip;
        // once open, the sidebar stays open while the cursor is within the full width.
        let in_zone = match (self.side, current) {
            (SidebarSide::Left, SidebarState::Collapsed | SidebarState::Expanding) => {
                cursor_x <= collapsed_w + threshold
            }
            (SidebarSide::Left, _) => cursor_x <= full_w + threshold,
            (SidebarSide::Right, SidebarState::Collapsed | SidebarState::Expanding) => {
                cursor_x >= window_width - collapsed_w - threshold
            }
            (SidebarSide::Right, _) => cursor_x >= window_width - full_w - threshold,
        };

        match (current, in_zone) {
            (SidebarState::Collapsed, true) => Some(SidebarTransition::StartExpand),
            (SidebarState::Expanding, true) => Some(SidebarTransition::Open),
            (SidebarState::Open | SidebarState::Expanding, false) => {
                Some(SidebarTransition::Collapse)
            }
            _ => None,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_width_is_icon_strip() {
        assert_eq!(SidebarState::COLLAPSED_WIDTH, 48);
    }

    #[test]
    fn render_width_collapsed() {
        assert_eq!(SidebarState::Collapsed.render_width(220), 48);
    }

    #[test]
    fn render_width_open() {
        assert_eq!(SidebarState::Open.render_width(220), 220);
    }

    #[test]
    fn proximity_left_triggers_expand() {
        let obs = MouseProximityObserver::new(SidebarSide::Left, 220);
        let t = obs.check(10.0, 1280.0, SidebarState::Collapsed);
        assert_eq!(t, Some(SidebarTransition::StartExpand));
    }

    #[test]
    fn proximity_left_far_triggers_collapse() {
        let obs = MouseProximityObserver::new(SidebarSide::Left, 220);
        let t = obs.check(600.0, 1280.0, SidebarState::Open);
        assert_eq!(t, Some(SidebarTransition::Collapse));
    }

    #[test]
    fn pinned_mode_never_triggers() {
        let mut obs = MouseProximityObserver::new(SidebarSide::Left, 220);
        obs.mode = SidebarMode::Pinned;
        assert_eq!(obs.check(600.0, 1280.0, SidebarState::Open), None);
    }

    #[test]
    fn proximity_right_triggers_expand() {
        // Expand triggers within collapsed_w + threshold = 48 + 16 = 64px of the right edge.
        // window_width=1280 → trigger zone: cursor_x >= 1280 - 64 = 1216
        let obs = MouseProximityObserver::new(SidebarSide::Right, 300);
        let t = obs.check(1240.0, 1280.0, SidebarState::Collapsed);
        assert_eq!(t, Some(SidebarTransition::StartExpand));
    }

    #[test]
    fn show_labels_only_when_open() {
        assert!(!SidebarState::Collapsed.show_labels());
        assert!(!SidebarState::Expanding.show_labels());
        assert!(SidebarState::Open.show_labels());
    }
}
