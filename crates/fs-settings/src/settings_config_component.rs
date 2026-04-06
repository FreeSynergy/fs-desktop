// settings_config_component.rs — SettingsConfigComponent: multi-section settings panel.
//
// Design Pattern: Facade (SettingsHub — bundles all setting sections)
//   Each section is a discrete LayoutElement::ExpandableGroup.
//   Sections adjust to the active program via ComponentCtx.config["program_id"].
//   Changes are applied immediately (no restart), emitted as Bus-Events.
//
// Data source: fs-db-desktop (settings table) via gRPC stub.
// Writes: Bus-Events ("settings.changed" topic).

use fs_render::component::{ButtonStyle, ComponentCtx, ComponentTrait, LayoutElement, TextSize};
use fs_render::layout::SlotKind;

// ── SettingsSection ────────────────────────────────────────────────────────────

/// A settings section shown as an expandable group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSection {
    /// Visual appearance: theme, icon size, menu style.
    Appearance,
    /// Interface language selection.
    Language,
    /// Desktop background (colour or image).
    Background,
    /// Keyboard shortcuts editor.
    Shortcuts,
    /// Any program-specific section injected via ctx.config.
    Custom(String),
}

impl SettingsSection {
    fn label_key(&self) -> String {
        match self {
            Self::Appearance => "settings-section-appearance".into(),
            Self::Language => "settings-section-language".into(),
            Self::Background => "settings-section-background".into(),
            Self::Shortcuts => "settings-section-shortcuts".into(),
            Self::Custom(name) => name.clone(),
        }
    }

    fn icon_key(&self) -> &str {
        match self {
            Self::Appearance => "fs:settings/appearance",
            Self::Language => "fs:settings/language",
            Self::Background => "fs:settings/background",
            Self::Shortcuts => "fs:settings/shortcuts",
            Self::Custom(_) => "fs:settings/custom",
        }
    }

    fn render_children(&self) -> Vec<LayoutElement> {
        match self {
            Self::Appearance => appearance_children(),
            Self::Language => language_children(),
            Self::Background => background_children(),
            Self::Shortcuts => shortcuts_children(),
            Self::Custom(name) => vec![LayoutElement::Text {
                content: format!("{name}-placeholder"),
                size: TextSize::Body,
                color: None,
            }],
        }
    }
}

fn appearance_children() -> Vec<LayoutElement> {
    vec![
        // Icon size slider (16–64 px, default 32 px)
        LayoutElement::Row {
            children: vec![
                LayoutElement::Text {
                    content: "settings-appearance-icon-size".into(),
                    size: TextSize::Body,
                    color: None,
                },
                LayoutElement::Badge {
                    content: "32".into(),
                    color: None,
                },
            ],
            gap: 8,
        },
        // Menu style: Round | Sidebar
        LayoutElement::Row {
            children: vec![
                LayoutElement::Text {
                    content: "settings-appearance-menu-style".into(),
                    size: TextSize::Body,
                    color: None,
                },
                LayoutElement::Button {
                    label_key: "settings-appearance-menu-round".into(),
                    action: "settings.appearance.menu_style=round".into(),
                    style: ButtonStyle::Ghost,
                },
                LayoutElement::Button {
                    label_key: "settings-appearance-menu-sidebar".into(),
                    action: "settings.appearance.menu_style=sidebar".into(),
                    style: ButtonStyle::Ghost,
                },
            ],
            gap: 8,
        },
    ]
}

fn language_children() -> Vec<LayoutElement> {
    vec![
        LayoutElement::Text {
            content: "settings-language-active".into(),
            size: TextSize::Body,
            color: None,
        },
        LayoutElement::Button {
            label_key: "settings-language-change".into(),
            action: "settings.language.open_picker".into(),
            style: ButtonStyle::Ghost,
        },
    ]
}

fn background_children() -> Vec<LayoutElement> {
    vec![LayoutElement::Row {
        children: vec![
            LayoutElement::Button {
                label_key: "settings-background-color".into(),
                action: "settings.background.mode=color".into(),
                style: ButtonStyle::Ghost,
            },
            LayoutElement::Button {
                label_key: "settings-background-image".into(),
                action: "settings.background.mode=image".into(),
                style: ButtonStyle::Ghost,
            },
        ],
        gap: 8,
    }]
}

fn shortcuts_children() -> Vec<LayoutElement> {
    vec![
        LayoutElement::Text {
            content: "settings-shortcuts-hint".into(),
            size: TextSize::Body,
            color: None,
        },
        LayoutElement::Button {
            label_key: "settings-shortcuts-open-editor".into(),
            action: "settings.shortcuts.open_editor".into(),
            style: ButtonStyle::Primary,
        },
    ]
}

// ── SettingsHub ────────────────────────────────────────────────────────────────

/// Facade: determines which sections to display.
///
/// `program_id` (from `ComponentCtx.config`) scopes the sections:
/// - `""` or `"desktop"` → all desktop sections
/// - any other id       → generic sections + a Custom("program-settings.{id}") section
struct SettingsHub;

impl SettingsHub {
    fn sections_for(program_id: &str) -> Vec<SettingsSection> {
        let mut sections = vec![
            SettingsSection::Appearance,
            SettingsSection::Language,
            SettingsSection::Background,
            SettingsSection::Shortcuts,
        ];
        if !program_id.is_empty() && program_id != "desktop" {
            sections.push(SettingsSection::Custom(format!(
                "settings-program-{program_id}"
            )));
        }
        sections
    }
}

// ── SettingsConfigComponent ───────────────────────────────────────────────────

/// Multi-section settings panel — adapts to the active program.
///
/// Wiring:
/// - `ctx.config["program_id"]` → which program's settings to show.
///   Omit or set to `"desktop"` for desktop-global settings.
/// - All changes emit Bus-Events (`"settings.changed"` topic) — no restart needed.
pub struct SettingsConfigComponent {
    id: &'static str,
}

impl SettingsConfigComponent {
    /// Create a new settings config component.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: "settings-config",
        }
    }
}

impl Default for SettingsConfigComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentTrait for SettingsConfigComponent {
    fn component_id(&self) -> &str {
        self.id
    }

    fn name_key(&self) -> &'static str {
        "component-settings-config-name"
    }

    fn description_key(&self) -> &'static str {
        "component-settings-config-desc"
    }

    fn slot_preference(&self) -> SlotKind {
        SlotKind::Fill
    }

    fn min_width(&self) -> u32 {
        260
    }

    fn render(&self, ctx: &ComponentCtx) -> Vec<LayoutElement> {
        let program_id = ctx.config.get("program_id").map_or("", String::as_str);

        let sections = SettingsHub::sections_for(program_id);

        let mut elements = vec![
            LayoutElement::Text {
                content: "component-settings-config-name".into(),
                size: TextSize::Label,
                color: None,
            },
            LayoutElement::Separator { label_key: None },
        ];

        for section in &sections {
            elements.push(LayoutElement::ExpandableGroup {
                label_key: section.label_key(),
                icon_key: section.icon_key().into(),
                children: section.render_children(),
                expanded: false,
            });
        }

        elements
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_render::layout::{ShellKind, SlotKind};

    #[test]
    fn component_id() {
        let c = SettingsConfigComponent::new();
        assert_eq!(c.component_id(), "settings-config");
    }

    #[test]
    fn slot_preference_is_fill() {
        let c = SettingsConfigComponent::new();
        assert_eq!(c.slot_preference(), SlotKind::Fill);
    }

    #[test]
    fn desktop_sections_count() {
        let sections = SettingsHub::sections_for("desktop");
        assert_eq!(sections.len(), 4);
    }

    #[test]
    fn program_sections_include_custom() {
        let sections = SettingsHub::sections_for("kanidm");
        assert!(sections
            .iter()
            .any(|s| matches!(s, SettingsSection::Custom(n) if n.contains("kanidm"))));
    }

    #[test]
    fn render_produces_expandable_groups() {
        let c = SettingsConfigComponent::new();
        let ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        let els = c.render(&ctx);
        let groups = els
            .iter()
            .filter(|e| matches!(e, LayoutElement::ExpandableGroup { .. }))
            .count();
        assert_eq!(groups, 4); // desktop default = 4 sections
    }

    #[test]
    fn render_with_program_id_adds_custom_section() {
        let c = SettingsConfigComponent::new();
        let mut ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        ctx.config.insert("program_id".into(), "stalwart".into());
        let els = c.render(&ctx);
        let groups = els
            .iter()
            .filter(|e| matches!(e, LayoutElement::ExpandableGroup { .. }))
            .count();
        assert_eq!(groups, 5); // 4 standard + 1 custom
    }

    #[test]
    fn section_label_keys_are_unique() {
        let sections = SettingsHub::sections_for("desktop");
        let keys: Vec<String> = sections.iter().map(SettingsSection::label_key).collect();
        let unique: std::collections::HashSet<&String> = keys.iter().collect();
        assert_eq!(keys.len(), unique.len());
    }
}
