/// Settings — root component: all settings sections in one place.
use dioxus::prelude::*;
use fsn_components::SidebarNavBtn;

use crate::appearance::AppearanceSettings;
use crate::language::LanguageSettings;
use crate::service_roles::ServiceRoles;
use crate::accounts::AccountSettings;
use crate::desktop_settings::DesktopSettings;

#[derive(Clone, PartialEq, Debug)]
pub enum SettingsSection {
    Appearance,
    Language,
    ServiceRoles,
    Accounts,
    Desktop,
}

impl SettingsSection {
    pub fn label(&self) -> &str {
        match self {
            Self::Appearance   => "Appearance",
            Self::Language     => "Language",
            Self::ServiceRoles => "Service Roles",
            Self::Accounts     => "Accounts",
            Self::Desktop      => "Desktop",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Appearance   => "🎨",
            Self::Language     => "🌐",
            Self::ServiceRoles => "🔗",
            Self::Accounts     => "👤",
            Self::Desktop      => "🖥",
        }
    }
}

const ALL_SECTIONS: &[SettingsSection] = &[
    SettingsSection::Appearance,
    SettingsSection::Language,
    SettingsSection::ServiceRoles,
    SettingsSection::Accounts,
    SettingsSection::Desktop,
];

/// Root Settings component.
#[component]
pub fn SettingsApp() -> Element {
    let mut active = use_signal(|| SettingsSection::Appearance);

    rsx! {
        div {
            class: "fsd-settings",
            style: "display: flex; height: 100%; background: var(--fsn-color-bg-base);",

            // Sidebar navigation
            div {
                style: "width: 220px; background: var(--fsn-color-bg-surface); border-right: 1px solid var(--fsn-color-border-default); padding: 16px 8px;",

                h2 { style: "margin: 0 0 16px 8px; font-size: 16px;", "Settings" }

                for section in ALL_SECTIONS {
                    SidebarNavBtn {
                        key: "{section.label()}",
                        label: section.label().to_string(),
                        icon:  section.icon().to_string(),
                        is_active: *active.read() == *section,
                        on_click: {
                            let s = (*section).clone();
                            move |_| active.set(s.clone())
                        }
                    }
                }
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
                }
            }
        }
    }
}

