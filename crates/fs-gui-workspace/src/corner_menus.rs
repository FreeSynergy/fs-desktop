//! Corner menu descriptors for the `FreeSynergy` Desktop shell.
//!
//! Design Pattern: Descriptor (each struct describes a menu; the engine renders it)
//!
//! Four menus occupy the four screen corners:
//!   [`TasksMenu`]    — top-left:     all installed programs / App Launcher
//!   [`SettingsMenu`] — bottom-left:  desktop settings
//!   [`HelpMenu`]     — top-right:    general + context help
//!   [`AiMenu`]       — bottom-right: AI assistant (shown only when capability present)

use fs_render::navigation::{
    CompositeIcon, Corner, CornerMenuDescriptor, IconRef, MenuItemDescriptor,
};

// ── TasksMenu ─────────────────────────────────────────────────────────────────

/// Top-left corner menu — all installed programs and the App Launcher.
pub struct TasksMenu {
    /// `(action, label, icon_key)` — action is dispatched on item press.
    pub entries: Vec<(String, String, String)>,
}

impl TasksMenu {
    #[must_use]
    pub fn new(entries: Vec<(String, String, String)>) -> Self {
        Self { entries }
    }

    /// Build the default task menu from the built-in app list.
    #[must_use]
    pub fn default_entries() -> Self {
        Self::new(vec![
            (
                "open:launcher".into(),
                fs_i18n::t("desktop-corner-tasks-launcher").to_string(),
                "fs:nav/launcher".into(),
            ),
            (
                "open:store".into(),
                fs_i18n::t("desktop-corner-tasks-store").to_string(),
                "fs:nav/store".into(),
            ),
            (
                "open:browser".into(),
                fs_i18n::t("desktop-corner-tasks-browser").to_string(),
                "fs:nav/browser".into(),
            ),
            (
                "open:lenses".into(),
                fs_i18n::t("desktop-corner-tasks-lenses").to_string(),
                "fs:nav/lenses".into(),
            ),
            (
                "open:tasks".into(),
                fs_i18n::t("desktop-corner-tasks-tasks").to_string(),
                "fs:nav/tasks".into(),
            ),
            (
                "open:bots".into(),
                fs_i18n::t("desktop-corner-tasks-bots").to_string(),
                "fs:nav/bots".into(),
            ),
            (
                "open:managers".into(),
                fs_i18n::t("desktop-corner-tasks-managers").to_string(),
                "fs:nav/managers".into(),
            ),
            (
                "open:profile".into(),
                fs_i18n::t("desktop-corner-tasks-profile").to_string(),
                "fs:nav/profile".into(),
            ),
        ])
    }
}

impl CornerMenuDescriptor for TasksMenu {
    fn corner(&self) -> Corner {
        Corner::TopLeft
    }

    fn items(&self) -> Vec<MenuItemDescriptor> {
        self.entries
            .iter()
            .map(|(action, label, icon_key)| {
                MenuItemDescriptor::new(
                    action.clone(),
                    CompositeIcon::single(IconRef::new(icon_key.clone())),
                    label.clone(),
                    action.clone(),
                )
            })
            .collect()
    }
}

// ── SettingsMenu ──────────────────────────────────────────────────────────────

/// Bottom-left corner menu — desktop settings entries.
pub struct SettingsMenu {
    pub entries: Vec<(String, String)>,
}

impl SettingsMenu {
    #[must_use]
    pub fn default_entries() -> Self {
        Self {
            entries: vec![
                (
                    "settings:appearance".into(),
                    fs_i18n::t("desktop-corner-settings-appearance").to_string(),
                ),
                (
                    "settings:language".into(),
                    fs_i18n::t("desktop-corner-settings-language").to_string(),
                ),
                (
                    "settings:desktop".into(),
                    fs_i18n::t("desktop-corner-settings-desktop").to_string(),
                ),
            ],
        }
    }
}

impl CornerMenuDescriptor for SettingsMenu {
    fn corner(&self) -> Corner {
        Corner::BottomLeft
    }

    fn items(&self) -> Vec<MenuItemDescriptor> {
        self.entries
            .iter()
            .map(|(action, label)| {
                MenuItemDescriptor::new(
                    action.clone(),
                    CompositeIcon::single(IconRef::new("fs:nav/settings")),
                    label.clone(),
                    action.clone(),
                )
            })
            .collect()
    }
}

// ── HelpMenu ──────────────────────────────────────────────────────────────────

/// Top-right corner menu — general and context-sensitive help.
pub struct HelpMenu {
    pub entries: Vec<(String, String)>,
}

impl HelpMenu {
    #[must_use]
    pub fn default_entries() -> Self {
        Self {
            entries: vec![
                (
                    "help:general".into(),
                    fs_i18n::t("desktop-corner-help-general").to_string(),
                ),
                (
                    "help:focus".into(),
                    fs_i18n::t("desktop-corner-help-focus").to_string(),
                ),
                (
                    "help:docs".into(),
                    fs_i18n::t("desktop-corner-help-docs").to_string(),
                ),
            ],
        }
    }
}

impl CornerMenuDescriptor for HelpMenu {
    fn corner(&self) -> Corner {
        Corner::TopRight
    }

    fn items(&self) -> Vec<MenuItemDescriptor> {
        self.entries
            .iter()
            .map(|(action, label)| {
                MenuItemDescriptor::new(
                    action.clone(),
                    CompositeIcon::single(IconRef::new("fs:nav/help")),
                    label.clone(),
                    action.clone(),
                )
            })
            .collect()
    }
}

// ── AiMenu ────────────────────────────────────────────────────────────────────

/// Bottom-right corner menu — AI assistant (shown only when capability present).
pub struct AiMenu {
    pub entries: Vec<(String, String)>,
}

impl AiMenu {
    #[must_use]
    pub fn default_entries() -> Self {
        Self {
            entries: vec![
                (
                    "ai:chat".into(),
                    fs_i18n::t("desktop-corner-ai-chat").to_string(),
                ),
                (
                    "ai:suggest".into(),
                    fs_i18n::t("desktop-corner-ai-suggest").to_string(),
                ),
            ],
        }
    }
}

impl CornerMenuDescriptor for AiMenu {
    fn corner(&self) -> Corner {
        Corner::BottomRight
    }

    fn items(&self) -> Vec<MenuItemDescriptor> {
        self.entries
            .iter()
            .map(|(action, label)| {
                MenuItemDescriptor::new(
                    action.clone(),
                    CompositeIcon::single(IconRef::new("fs:nav/ai")),
                    label.clone(),
                    action.clone(),
                )
            })
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tasks_menu_corner_is_top_left() {
        let m = TasksMenu::new(vec![]);
        assert_eq!(m.corner(), Corner::TopLeft);
    }

    #[test]
    fn settings_menu_corner_is_bottom_left() {
        let m = SettingsMenu { entries: vec![] };
        assert_eq!(m.corner(), Corner::BottomLeft);
    }

    #[test]
    fn help_menu_corner_is_top_right() {
        let m = HelpMenu { entries: vec![] };
        assert_eq!(m.corner(), Corner::TopRight);
    }

    #[test]
    fn ai_menu_corner_is_bottom_right() {
        let m = AiMenu { entries: vec![] };
        assert_eq!(m.corner(), Corner::BottomRight);
    }

    #[test]
    fn tasks_menu_items_match_entries() {
        let m = TasksMenu::new(vec![(
            "open:browser".into(),
            "Browser".into(),
            "fs:apps/browser".into(),
        )]);
        let items = m.items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action, "open:browser");
        assert_eq!(items[0].label_key, "Browser");
    }

    #[test]
    fn settings_menu_items_have_correct_actions() {
        let m = SettingsMenu {
            entries: vec![("settings:desktop".into(), "Desktop".into())],
        };
        let items = m.items();
        assert_eq!(items[0].action, "settings:desktop");
    }

    #[test]
    fn help_menu_items_have_correct_actions() {
        let m = HelpMenu {
            entries: vec![("help:general".into(), "Help".into())],
        };
        let items = m.items();
        assert_eq!(items[0].action, "help:general");
    }

    #[test]
    fn ai_menu_items_have_correct_actions() {
        let m = AiMenu {
            entries: vec![("ai:chat".into(), "AI Chat".into())],
        };
        let items = m.items();
        assert_eq!(items[0].action, "ai:chat");
    }
}
