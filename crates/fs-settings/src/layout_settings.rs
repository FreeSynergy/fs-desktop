// layout_settings.rs — Desktop Layout settings section (iced).
//
// Design Pattern: Strategy
//   LayoutSectionStrategy — trait: each section type (Topbar/Sidebar/etc.) is a Strategy.
//   SidebarSectionStrategy, TopbarSectionStrategy, etc. — concrete implementations.
//   LayoutSettingsState — context that holds the loaded ShellLayout and dispatches to strategies.
//
// Renders the "Layout" tab within Desktop Settings.
// Allows toggling section visibility and inspecting slot assignments.

use fs_gui_engine_iced::iced::{
    self,
    widget::{button, checkbox, column, container, row, scrollable, text, Space},
    Color, Element, Length,
};
use fs_i18n;
use fs_render::layout::{ShellKind, SlotKind};

use crate::app::{Message, SettingsApp};

// ── LayoutSectionInfo ─────────────────────────────────────────────────────────

/// Description of one shell section for display in Settings.
pub struct LayoutSectionInfo {
    pub kind: ShellKind,
    /// I18n key for the section name.
    pub name_key: &'static str,
    /// Slot entries: (`slot_kind`, list of component ids).
    pub slots: Vec<(SlotKind, Vec<String>)>,
    pub visible: bool,
}

// ── LayoutSectionStrategy ─────────────────────────────────────────────────────

/// Strategy: renders one shell section as a settings row.
///
/// Each concrete strategy knows how to display its section's options.
/// The Settings view iterates over strategies and calls `render()` on each.
pub trait LayoutSectionStrategy {
    fn info(&self) -> LayoutSectionInfo;

    fn render<'a>(&self, _app: &'a SettingsApp) -> Element<'a, Message> {
        let info = self.info();
        let name = fs_i18n::t(info.name_key);

        let visible_check = checkbox(
            String::from(fs_i18n::t("shell-layout-visible")),
            info.visible,
        )
        .on_toggle(move |_| Message::LayoutToggleSection(info.kind.clone()));

        let slot_rows: Vec<Element<Message>> = info
            .slots
            .iter()
            .map(|(slot_kind, component_ids)| {
                let slot_label = match slot_kind {
                    SlotKind::Top => fs_i18n::t("shell-layout-slot-top"),
                    SlotKind::Fill => fs_i18n::t("shell-layout-slot-fill"),
                    SlotKind::Bottom => fs_i18n::t("shell-layout-slot-bottom"),
                    SlotKind::Sidebar => fs_i18n::t("shell-layout-sidebar"),
                };
                let components_str = if component_ids.is_empty() {
                    "—".to_string()
                } else {
                    component_ids.join(", ")
                };
                row![
                    text(String::from(slot_label)).size(11).width(80),
                    text(components_str).size(11),
                ]
                .spacing(8)
                .into()
            })
            .collect();

        let detail: Element<'_, Message> = if slot_rows.is_empty() {
            Space::with_height(0).into()
        } else {
            column(slot_rows).spacing(2).padding([4, 0]).into()
        };

        container(
            column![
                row![
                    text(String::from(name)).size(14).width(Length::Fill),
                    visible_check,
                ]
                .spacing(8),
                detail,
            ]
            .spacing(4),
        )
        .padding([12, 16])
        .width(Length::Fill)
        .style(|_theme| container::Style {
            border: iced::Border {
                color: Color::from_rgba(0.58, 0.67, 0.78, 0.15),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..container::Style::default()
        })
        .into()
    }
}

// ── Concrete strategies ───────────────────────────────────────────────────────

pub struct TopbarStrategy {
    pub visible: bool,
    pub slots: Vec<(SlotKind, Vec<String>)>,
}

impl LayoutSectionStrategy for TopbarStrategy {
    fn info(&self) -> LayoutSectionInfo {
        LayoutSectionInfo {
            kind: ShellKind::Topbar,
            name_key: "shell-layout-topbar",
            slots: self.slots.clone(),
            visible: self.visible,
        }
    }
}

pub struct SidebarStrategy {
    pub visible: bool,
    pub slots: Vec<(SlotKind, Vec<String>)>,
}

impl LayoutSectionStrategy for SidebarStrategy {
    fn info(&self) -> LayoutSectionInfo {
        LayoutSectionInfo {
            kind: ShellKind::Sidebar,
            name_key: "shell-layout-sidebar",
            slots: self.slots.clone(),
            visible: self.visible,
        }
    }
}

pub struct BottombarStrategy {
    pub visible: bool,
    pub slots: Vec<(SlotKind, Vec<String>)>,
}

impl LayoutSectionStrategy for BottombarStrategy {
    fn info(&self) -> LayoutSectionInfo {
        LayoutSectionInfo {
            kind: ShellKind::Bottombar,
            name_key: "shell-layout-bottombar",
            slots: self.slots.clone(),
            visible: self.visible,
        }
    }
}

// ── LayoutSettingsState ───────────────────────────────────────────────────────

/// Context for the Layout Settings section.
///
/// Loads `ShellLayout` from disk and builds the strategy list from it.
/// Re-loaded on every `update(Message::LayoutToggleSection)` call.
#[derive(Debug, Clone)]
pub struct LayoutSettingsState {
    /// Serialized current layout (loaded from TOML).
    pub layout_toml: String,
}

impl LayoutSettingsState {
    #[must_use]
    pub fn new() -> Self {
        let layout_toml = std::fs::read_to_string(layout_config_path()).unwrap_or_default();
        Self { layout_toml }
    }

    /// Reload layout from disk.
    pub fn reload(&mut self) {
        self.layout_toml = std::fs::read_to_string(layout_config_path()).unwrap_or_default();
    }
}

impl Default for LayoutSettingsState {
    fn default() -> Self {
        Self::new()
    }
}

fn layout_config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("freesynergy")
        .join("desktop")
        .join("desktop-layout.toml")
}

// ── view_layout_settings ─────────────────────────────────────────────────────

/// Render the Layout sub-page within Desktop Settings.
///
/// Reads the live `ShellLayout` from disk and builds the strategy list.
/// Each section is rendered via its strategy.
#[must_use]
pub fn view_layout_settings(app: &SettingsApp) -> Element<'_, Message> {
    // Parse current layout from saved TOML (fallback: standard defaults).
    let layout = if app.layout.layout_toml.is_empty() {
        None
    } else {
        toml::from_str::<LayoutTableProxy>(&app.layout.layout_toml).ok()
    };

    let title = text(String::from(fs_i18n::t("shell-layout-config-title"))).size(18);
    let hint = text(String::from(fs_i18n::t("shell-layout-config-hint")))
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.6));

    let strategies: Vec<Box<dyn LayoutSectionStrategy>> = if let Some(proxy) = layout {
        proxy.into_strategies()
    } else {
        default_strategies()
    };

    let section_rows: Vec<Element<Message>> = strategies.iter().map(|s| s.render(app)).collect();

    let save_btn = button(text(String::from(fs_i18n::t("actions.save"))).size(13))
        .on_press(Message::SaveLayout)
        .padding([8, 20]);

    column![
        title,
        hint,
        Space::with_height(16),
        scrollable(column(section_rows).spacing(8)).height(Length::Fill),
        Space::with_height(16),
        save_btn,
    ]
    .spacing(4)
    .into()
}

// ── Fallback strategies ───────────────────────────────────────────────────────

fn default_strategies() -> Vec<Box<dyn LayoutSectionStrategy>> {
    vec![
        Box::new(TopbarStrategy {
            visible: true,
            slots: vec![(SlotKind::Top, vec!["notification-bell".into()])],
        }),
        Box::new(SidebarStrategy {
            visible: true,
            slots: vec![
                (SlotKind::Fill, vec!["inventory-list".into()]),
                (SlotKind::Bottom, vec!["pinned-apps".into()]),
            ],
        }),
        Box::new(BottombarStrategy {
            visible: false,
            slots: vec![(SlotKind::Bottom, vec!["system-info".into()])],
        }),
    ]
}

// ── LayoutTableProxy ─────────────────────────────────────────────────────────

/// Minimal TOML proxy for parsing `ShellLayout` sections (avoids importing fs-gui-workspace).
#[derive(serde::Deserialize, Debug, Clone)]
struct ShellSectionProxy {
    kind: String,
    #[serde(default = "default_true")]
    visible: bool,
    #[serde(default)]
    slots: Vec<SlotProxy>,
}

fn default_true() -> bool {
    true
}

#[derive(serde::Deserialize, Debug, Clone)]
struct SlotProxy {
    kind: String,
    #[serde(default)]
    component_ids: Vec<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct LayoutTableProxy {
    sections: Vec<ShellSectionProxy>,
}

impl LayoutTableProxy {
    fn into_strategies(self) -> Vec<Box<dyn LayoutSectionStrategy>> {
        self.sections
            .into_iter()
            .filter_map(|s| {
                let slots: Vec<(SlotKind, Vec<String>)> = s
                    .slots
                    .iter()
                    .map(|slot| {
                        let kind = match slot.kind.as_str() {
                            "top" => SlotKind::Top,
                            "bottom" => SlotKind::Bottom,
                            _ => SlotKind::Fill,
                        };
                        (kind, slot.component_ids.clone())
                    })
                    .collect();
                match s.kind.as_str() {
                    "topbar" => Some(Box::new(TopbarStrategy {
                        visible: s.visible,
                        slots,
                    }) as Box<dyn LayoutSectionStrategy>),
                    "sidebar" => Some(Box::new(SidebarStrategy {
                        visible: s.visible,
                        slots,
                    }) as Box<dyn LayoutSectionStrategy>),
                    "bottombar" => Some(Box::new(BottombarStrategy {
                        visible: s.visible,
                        slots,
                    }) as Box<dyn LayoutSectionStrategy>),
                    _ => None,
                }
            })
            .collect()
    }
}
