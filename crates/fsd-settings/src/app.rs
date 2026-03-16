/// Settings — root component: all settings sections in one place.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};

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
    pub fn label(&self) -> &str {
        match self {
            Self::Appearance   => "Appearance",
            Self::Language     => "Language",
            Self::ServiceRoles => "Service Roles",
            Self::Accounts     => "Accounts",
            Self::Desktop      => "Desktop",
            Self::Shortcuts    => "Shortcuts",
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

    /// Look up a section by its label string.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "Appearance"   => Some(Self::Appearance),
            "Language"     => Some(Self::Language),
            "Service Roles"=> Some(Self::ServiceRoles),
            "Accounts"     => Some(Self::Accounts),
            "Desktop"      => Some(Self::Desktop),
            "Shortcuts"    => Some(Self::Shortcuts),
            _              => None,
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
        .map(|s| FsnSidebarItem::new(s.label(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-settings",
            style: "display: flex; height: 100%; background: var(--fsn-color-bg-base);",

            // Collapsible sidebar navigation
            FsnSidebar {
                items:     sidebar_items,
                active_id: active.read().label().to_string(),
                on_select: move |id: String| {
                    if let Some(section) = SettingsSection::from_label(&id) {
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
        }
    }
}
