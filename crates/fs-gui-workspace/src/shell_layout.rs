// shell_layout.rs — ShellLayout: Composite pattern for the Desktop-Shell.
//
// Design Pattern: Composite
//   ShellLayout is the composite root — it contains ShellSections.
//   Each ShellSection contains SlotEntries.
//   The engine walks the composite tree and renders each section.
//
// TOML config: ~/.config/freesynergy/desktop/desktop-layout.toml
// Hot-reload:  inotify via fs-render HotReloadWatcher (Phase 3).
//
// Default layout (Topbar + Sidebar + Main):
//   Topbar  → top:  NotificationBell
//           → fill: SearchBar (Phase 6)
//   Sidebar → fill: InventoryList
//           → bottom: PinnedApps
//   Main    → fill: <active app content>
//   Bottombar → (hidden by default)

use std::path::{Path, PathBuf};

use fs_i18n;
use serde::{Deserialize, Serialize};

use fs_render::layout::{
    ComponentRef, LayoutDescriptor, ShellConfig, ShellKind, SlotConfig, SlotKind,
};

pub use crate::sidebar_state::SidebarSide;

// ── SlotEntry ─────────────────────────────────────────────────────────────────

/// One slot within a shell section — holds one or more component IDs.
///
/// Components are rendered in order within the slot.
/// The slot kind determines vertical positioning within the section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotEntry {
    /// Slot position within the section (top | fill | bottom).
    pub kind: SlotKind,
    /// Ordered list of component IDs rendered in this slot.
    pub component_ids: Vec<String>,
}

impl SlotEntry {
    /// Create a slot with a single component.
    pub fn single(kind: SlotKind, component_id: impl Into<String>) -> Self {
        Self {
            kind,
            component_ids: vec![component_id.into()],
        }
    }

    /// Create a slot with multiple components.
    #[must_use]
    pub fn multi(kind: SlotKind, component_ids: Vec<String>) -> Self {
        Self {
            kind,
            component_ids,
        }
    }
}

// ── ShellSection ──────────────────────────────────────────────────────────────

/// One shell section (Topbar, Sidebar, Bottombar, or Main).
///
/// Leaf node in the Composite tree: contains `SlotEntry` items but no child sections.
/// A hidden section is preserved in the config but skipped during rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellSection {
    /// Which shell container this section represents.
    pub kind: ShellKind,
    /// Whether this section is rendered at all.
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Fixed size in logical pixels (0 = auto-sized by the engine).
    /// For sidebar: width. For topbar/bottombar: height.
    #[serde(default)]
    pub size: u32,
    /// Slot assignments for this section.
    #[serde(default)]
    pub slots: Vec<SlotEntry>,
    /// Side on which this sidebar is placed (Left or Right).
    /// Only meaningful for `ShellKind::Sidebar`-family sections.
    #[serde(default)]
    pub position: SidebarSide,
}

fn default_true() -> bool {
    true
}

impl ShellSection {
    /// Create a visible section with the given slots.
    #[must_use]
    pub fn new(kind: ShellKind, slots: Vec<SlotEntry>) -> Self {
        Self {
            kind,
            visible: true,
            size: 0,
            slots,
            position: SidebarSide::Left,
        }
    }

    /// Create a visible section placed on the given side.
    #[must_use]
    pub fn with_position(mut self, position: SidebarSide) -> Self {
        self.position = position;
        self
    }

    /// Translated label for this section.
    #[must_use]
    pub fn label(&self) -> String {
        let key = match self.kind {
            ShellKind::Topbar => "shell-layout-topbar",
            ShellKind::Sidebar => "shell-layout-sidebar",
            ShellKind::Bottombar => "shell-layout-bottombar",
            ShellKind::Main => "shell-layout-main",
        };
        fs_i18n::t(key).into()
    }

    /// Returns all component IDs across all slots in order.
    #[must_use]
    pub fn all_component_ids(&self) -> Vec<&str> {
        self.slots
            .iter()
            .flat_map(|s| s.component_ids.iter().map(String::as_str))
            .collect()
    }

    /// Returns component IDs for a specific slot kind.
    #[must_use]
    pub fn components_for_slot(&self, slot: &SlotKind) -> Vec<&str> {
        self.slots
            .iter()
            .filter(|s| &s.kind == slot)
            .flat_map(|s| s.component_ids.iter().map(String::as_str))
            .collect()
    }
}

// ── ShellLayout ───────────────────────────────────────────────────────────────

/// Composite root — collects all shell sections and persists the layout config.
///
/// The Desktop shell owns exactly one `ShellLayout`. The engine walks the
/// section list top-to-bottom and renders each visible section.
///
/// Load from TOML: `ShellLayout::load()` (falls back to `default()` on error).
/// Save to TOML:   `layout.save()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellLayout {
    /// All shell sections in render order.
    pub sections: Vec<ShellSection>,
}

impl Default for ShellLayout {
    fn default() -> Self {
        Self::standard()
    }
}

impl ShellLayout {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Standard Desktop layout:
    ///   Topbar       (`notification-bell`)
    ///   Left Sidebar (`inventory-list` / `pinned-apps` / settings fixed bottom)
    ///   Main         (active app content)
    ///   Right Sidebar (`help` / AI input — hidden by default, expands on hover)
    ///   Bottombar    (`system-info` — hidden by default)
    #[must_use]
    pub fn standard() -> Self {
        Self {
            sections: vec![
                ShellSection {
                    kind: ShellKind::Topbar,
                    visible: true,
                    size: 60,
                    slots: vec![SlotEntry::single(SlotKind::Top, "notification-bell")],
                    position: SidebarSide::Left,
                },
                // Left sidebar = Taskbar (installed apps + pinned apps + settings)
                ShellSection {
                    kind: ShellKind::Sidebar,
                    visible: true,
                    size: 220,
                    slots: vec![
                        SlotEntry::single(SlotKind::Fill, "inventory-list"),
                        SlotEntry::single(SlotKind::Bottom, "pinned-apps"),
                    ],
                    position: SidebarSide::Left,
                },
                ShellSection {
                    kind: ShellKind::Main,
                    visible: true,
                    size: 0,
                    slots: vec![],
                    position: SidebarSide::Left,
                },
                // Right sidebar = Help + AI (collapsed by default, expands on hover)
                ShellSection {
                    kind: ShellKind::Sidebar,
                    visible: true,
                    size: 320,
                    slots: vec![SlotEntry::single(SlotKind::Fill, "help-panel")],
                    position: SidebarSide::Right,
                },
                ShellSection {
                    kind: ShellKind::Bottombar,
                    visible: false,
                    size: 32,
                    slots: vec![SlotEntry::single(SlotKind::Bottom, "system-info")],
                    position: SidebarSide::Left,
                },
            ],
        }
    }

    /// Returns all sidebar sections on the given side.
    #[must_use]
    pub fn sidebars_on_side(&self, side: SidebarSide) -> Vec<&ShellSection> {
        self.sections
            .iter()
            .filter(|s| s.kind == ShellKind::Sidebar && s.position == side)
            .collect()
    }

    // ── Section accessors ─────────────────────────────────────────────────────

    /// Returns the section for the given kind, if present.
    #[must_use]
    pub fn section(&self, kind: &ShellKind) -> Option<&ShellSection> {
        self.sections.iter().find(|s| &s.kind == kind)
    }

    /// Returns a mutable reference to the section for the given kind.
    pub fn section_mut(&mut self, kind: &ShellKind) -> Option<&mut ShellSection> {
        self.sections.iter_mut().find(|s| &s.kind == kind)
    }

    /// All visible sections in render order.
    #[must_use]
    pub fn visible_sections(&self) -> Vec<&ShellSection> {
        self.sections.iter().filter(|s| s.visible).collect()
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    /// Returns the TOML config path: `~/.config/freesynergy/desktop/desktop-layout.toml`.
    #[must_use]
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".config")
            .join("freesynergy")
            .join("desktop")
            .join("desktop-layout.toml")
    }

    /// Load layout from TOML — falls back to `ShellLayout::standard()` on any error.
    #[must_use]
    pub fn load() -> Self {
        let path = Self::config_path();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::standard();
        };
        toml::from_str(&content).unwrap_or_else(|_| Self::standard())
    }

    /// Persist the current layout to TOML (best-effort, errors are logged).
    pub fn save(&self) {
        let path = Self::config_path();
        if !Self::ensure_config_dir(&path) {
            return;
        }
        Self::write_toml(self, &path);
    }

    fn ensure_config_dir(path: &std::path::Path) -> bool {
        let Some(dir) = path.parent() else {
            return true;
        };
        if std::fs::create_dir_all(dir).is_err() {
            tracing::warn!("shell_layout: could not create config dir");
            return false;
        }
        true
    }

    fn write_toml(layout: &Self, path: &std::path::Path) {
        match toml::to_string_pretty(layout) {
            Ok(content) => {
                if std::fs::write(path, content).is_err() {
                    tracing::warn!("shell_layout: could not write config file");
                }
            }
            Err(e) => tracing::warn!("shell_layout: serialization failed: {e}"),
        }
    }

    // ── Conversion ────────────────────────────────────────────────────────────

    /// Convert this `ShellLayout` into a `LayoutDescriptor` suitable for
    /// the `IcedLayoutInterpreter`.
    ///
    /// Component IDs from each section's slots are mapped to lightweight
    /// `ComponentRef` entries (no WASM path — in-process registered components).
    #[must_use]
    pub fn to_layout_descriptor(&self) -> LayoutDescriptor {
        let mut desc = LayoutDescriptor::default();

        for section in &self.sections {
            if !section.visible {
                continue;
            }
            let shell_cfg = section_to_shell_config(section);
            match section.kind {
                ShellKind::Topbar => desc.topbar = shell_cfg,
                ShellKind::Sidebar => desc.sidebar = shell_cfg,
                ShellKind::Bottombar => desc.bottombar = shell_cfg,
                ShellKind::Main => desc.main = shell_cfg,
            }
        }

        desc
    }

    // ── Mutations ─────────────────────────────────────────────────────────────

    /// Toggle visibility of the section with the given kind.
    pub fn toggle_visibility(&mut self, kind: &ShellKind) {
        if let Some(section) = self.section_mut(kind) {
            section.visible = !section.visible;
        }
    }

    /// Set slot components for a section (replaces existing slot entry of that kind).
    pub fn set_slot(
        &mut self,
        section_kind: &ShellKind,
        slot_kind: SlotKind,
        component_ids: Vec<String>,
    ) {
        if let Some(section) = self.section_mut(section_kind) {
            section.slots.retain(|s| s.kind != slot_kind);
            if !component_ids.is_empty() {
                section.slots.push(SlotEntry {
                    kind: slot_kind,
                    component_ids,
                });
            }
        }
    }
}

// ── Conversion helpers ────────────────────────────────────────────────────────

fn section_to_shell_config(section: &ShellSection) -> ShellConfig {
    let mut slot_cfg = SlotConfig::default();

    for slot_entry in &section.slots {
        let refs: Vec<ComponentRef> = slot_entry
            .component_ids
            .iter()
            .map(|id| ComponentRef {
                id: id.clone(),
                slot: slot_entry.kind.clone(),
                // In-process components have no WASM path.
                wasm: Path::new("").to_path_buf(),
                config: None,
                bounds: fs_render::layout::SlotBounds::default(),
            })
            .collect();

        match slot_entry.kind {
            SlotKind::Top => slot_cfg.top.extend(refs),
            SlotKind::Fill | SlotKind::Sidebar => slot_cfg.fill.extend(refs),
            SlotKind::Bottom => slot_cfg.bottom.extend(refs),
        }
    }

    ShellConfig {
        enabled: section.visible,
        size: section.size,
        slots: slot_cfg,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_layout_has_five_sections() {
        // Topbar + LeftSidebar + Main + RightSidebar + Bottombar
        let layout = ShellLayout::standard();
        assert_eq!(layout.sections.len(), 5);
    }

    #[test]
    fn standard_layout_has_two_sidebars() {
        let layout = ShellLayout::standard();
        let left = layout.sidebars_on_side(SidebarSide::Left);
        let right = layout.sidebars_on_side(SidebarSide::Right);
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
    }

    #[test]
    fn sidebar_visible_by_default() {
        let layout = ShellLayout::standard();
        let sidebar = layout.section(&ShellKind::Sidebar).unwrap();
        assert!(sidebar.visible);
    }

    #[test]
    fn bottombar_hidden_by_default() {
        let layout = ShellLayout::standard();
        let bottombar = layout.section(&ShellKind::Bottombar).unwrap();
        assert!(!bottombar.visible);
    }

    #[test]
    fn toggle_visibility_flips_state() {
        let mut layout = ShellLayout::standard();
        layout.toggle_visibility(&ShellKind::Bottombar);
        assert!(layout.section(&ShellKind::Bottombar).unwrap().visible);
    }

    #[test]
    fn sidebar_inventory_in_fill_slot() {
        let layout = ShellLayout::standard();
        let sidebar = layout.section(&ShellKind::Sidebar).unwrap();
        let fill_ids = sidebar.components_for_slot(&SlotKind::Fill);
        assert!(fill_ids.contains(&"inventory-list"));
    }

    #[test]
    fn roundtrip_toml_serialization() {
        let layout = ShellLayout::standard();
        let toml_str = toml::to_string_pretty(&layout).unwrap();
        let parsed: ShellLayout = toml::from_str(&toml_str).unwrap();
        assert_eq!(layout.sections.len(), parsed.sections.len());
    }
}
