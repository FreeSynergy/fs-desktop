/// Settings — root component: all settings sections in one place.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};
use fsn_i18n;

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
            Self::Appearance   => fsn_i18n::t("settings.section.appearance"),
            Self::Language     => fsn_i18n::t("settings.section.language"),
            Self::ServiceRoles => fsn_i18n::t("settings.section.roles"),
            Self::Accounts     => fsn_i18n::t("settings.section.accounts"),
            Self::Desktop      => fsn_i18n::t("settings.section.desktop"),
            Self::Shortcuts    => fsn_i18n::t("settings.section.shortcuts"),
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

    let sidebar_items: Vec<FsnSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsnSidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-settings",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fsn-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fsn-border); \
                        flex-shrink: 0; background: var(--fsn-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fsn-text-primary);",
                    {fsn_i18n::t("settings.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

            // Collapsible sidebar navigation
            FsnSidebar {
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
                match *active.read() {
                    SettingsSection::Appearance   => rsx! { AppearanceSettings {} },
                    SettingsSection::Language     => rsx! { LanguageSettings {} },
                    SettingsSection::ServiceRoles => rsx! { ServiceRoles {} },
                    SettingsSection::Accounts     => rsx! { AccountSettings {} },
                    SettingsSection::Desktop      => rsx! { DesktopSettings {} },
                    SettingsSection::Shortcuts    => rsx! { ShortcutsSettings {} },
                }
            }
            } // end sidebar + content row
        }
    }
}
