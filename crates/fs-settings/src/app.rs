/// Settings — root component: all settings sections in one place.
use dioxus::prelude::*;
use fs_components::{FsSidebar, FsSidebarItem, FS_SIDEBAR_CSS};
use fs_i18n;

use crate::appearance::AppearanceSettings;
use crate::language::LanguageSettings;
use crate::service_roles::ServiceRoles;
use crate::accounts::AccountSettings;
use crate::desktop_settings::DesktopSettings;
use crate::shortcuts::ShortcutsSettings;

#[derive(Clone, PartialEq, Debug)]
pub enum SettingsSection {
    Appearance,
    Language,
    ServiceRoles,
    Accounts,
    Desktop,
    Shortcuts,
}

impl SettingsSection {
    /// Stable identifier used for routing — never translated.
    pub fn id(&self) -> &str {
        match self {
            Self::Appearance   => "appearance",
            Self::Language     => "language",
            Self::ServiceRoles => "service_roles",
            Self::Accounts     => "accounts",
            Self::Desktop      => "desktop",
            Self::Shortcuts    => "shortcuts",
        }
    }

    /// Translated display label.
    pub fn label(&self) -> String {
        match self {
            Self::Appearance   => fs_i18n::t("settings.section.appearance"),
            Self::Language     => fs_i18n::t("settings.section.language"),
            Self::ServiceRoles => fs_i18n::t("settings.section.roles"),
            Self::Accounts     => fs_i18n::t("settings.section.accounts"),
            Self::Desktop      => fs_i18n::t("settings.section.desktop"),
            Self::Shortcuts    => fs_i18n::t("settings.section.shortcuts"),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Appearance   => "🎨",
            Self::Language     => "🌐",
            Self::ServiceRoles => "🔗",
            Self::Accounts     => "👤",
            Self::Desktop      => "🖥",
            Self::Shortcuts    => "⌨",
        }
    }

    /// Look up a section by its stable ID string.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "appearance"    => Some(Self::Appearance),
            "language"      => Some(Self::Language),
            "service_roles" => Some(Self::ServiceRoles),
            "accounts"      => Some(Self::Accounts),
            "desktop"       => Some(Self::Desktop),
            "shortcuts"     => Some(Self::Shortcuts),
            _               => None,
        }
    }
}

/// Trait that gives a settings section the ability to render itself.
///
/// Extend settings without touching `SettingsApp` — implement this trait.
pub trait SettingsPanel {
    fn render(&self) -> Element;
}

impl SettingsPanel for SettingsSection {
    fn render(&self) -> Element {
        match self {
            Self::Appearance   => rsx! { AppearanceSettings {} },
            Self::Language     => rsx! { LanguageSettings {} },
            Self::ServiceRoles => rsx! { ServiceRoles {} },
            Self::Accounts     => rsx! { AccountSettings {} },
            Self::Desktop      => rsx! { DesktopSettings {} },
            Self::Shortcuts    => rsx! { ShortcutsSettings {} },
        }
    }
}

const ALL_SECTIONS: &[SettingsSection] = &[
    SettingsSection::Appearance,
    SettingsSection::Language,
    SettingsSection::ServiceRoles,
    SettingsSection::Accounts,
    SettingsSection::Desktop,
    SettingsSection::Shortcuts,
];

/// Root Settings component.
#[component]
pub fn SettingsApp() -> Element {
    let mut active = use_signal(|| SettingsSection::Appearance);

    let sidebar_items: Vec<FsSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsSidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        div {
            class: "fs-settings",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fs-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fs-border); \
                        flex-shrink: 0; background: var(--fs-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fs-text-primary);",
                    {fs_i18n::t("settings.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

            // Collapsible sidebar navigation
            FsSidebar {
                items:     sidebar_items,
                active_id: active.read().id().to_string(),
                on_select: move |id: String| {
                    if let Some(section) = SettingsSection::from_id(&id) {
                        active.set(section);
                    }
                },
            }

            // Content
            div {
                style: "flex: 1; overflow: auto;",
                { active.read().render() }
            }
            } // end sidebar + content row
        }
    }
}
