/// Settings — root component: all settings sections in one place.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, FS_SIDEBAR_CSS};
use fs_i18n;

use crate::accounts::AccountSettings;
use crate::appearance::AppearanceSettings;
use crate::browser_settings::BrowserSettings;
use crate::desktop_settings::DesktopSettings;
use crate::language::LanguageSettings;
use crate::package_settings::{PackageSettingsEntry, PackageSettingsView};
use crate::service_roles::ServiceRoles;
use crate::shortcuts::ShortcutsSettings;

// ── Descriptor ────────────────────────────────────────────────────────────────

/// Static metadata for a settings section.
/// All per-variant constant data lives here — add a field once, not in every match.
pub struct SectionMeta {
    /// Stable identifier used for routing — never translated.
    pub id: &'static str,
    pub icon: &'static str,
    /// i18n key passed to `fs_i18n::t`.
    pub label_key: &'static str,
}

// ── SettingsSection ───────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum SettingsSection {
    Appearance,
    Language,
    ServiceRoles,
    Accounts,
    Desktop,
    Browser,
    Shortcuts,
    Packages,
}

impl SettingsSection {
    /// All sections in display order (including `Packages`).
    pub fn all() -> &'static [Self] {
        const ALL: &[SettingsSection] = &[
            SettingsSection::Appearance,
            SettingsSection::Language,
            SettingsSection::ServiceRoles,
            SettingsSection::Accounts,
            SettingsSection::Desktop,
            SettingsSection::Browser,
            SettingsSection::Shortcuts,
            SettingsSection::Packages,
        ];
        ALL
    }

    /// Static metadata for this section (id, icon, i18n key).
    /// Single source of truth — replaces parallel match blocks.
    pub fn meta(&self) -> SectionMeta {
        match self {
            Self::Appearance => SectionMeta {
                id: "appearance",
                icon: "🎨",
                label_key: "settings.section.appearance",
            },
            Self::Language => SectionMeta {
                id: "language",
                icon: "🌐",
                label_key: "settings.section.language",
            },
            Self::ServiceRoles => SectionMeta {
                id: "service_roles",
                icon: "🔗",
                label_key: "settings.section.roles",
            },
            Self::Accounts => SectionMeta {
                id: "accounts",
                icon: "👤",
                label_key: "settings.section.accounts",
            },
            Self::Desktop => SectionMeta {
                id: "desktop",
                icon: "🖥",
                label_key: "settings.section.desktop",
            },
            Self::Browser => SectionMeta {
                id: "browser",
                icon: "🌍",
                label_key: "settings.section.browser",
            },
            Self::Shortcuts => SectionMeta {
                id: "shortcuts",
                icon: "⌨",
                label_key: "settings.section.shortcuts",
            },
            Self::Packages => SectionMeta {
                id: "packages",
                icon: "📦",
                label_key: "settings.section.packages",
            },
        }
    }

    pub fn id(&self) -> &str {
        self.meta().id
    }
    pub fn icon(&self) -> &str {
        self.meta().icon
    }

    /// Translated display label.
    pub fn label(&self) -> String {
        fs_i18n::t(self.meta().label_key).into()
    }

    /// Sections shown without external package data (hides `Packages`).
    pub fn standard() -> impl Iterator<Item = &'static Self> {
        Self::all().iter().filter(|s| !matches!(s, Self::Packages))
    }

    /// Look up a section by its stable ID — delegates to `all()`, no duplicate match.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().iter().find(|s| s.id() == id).cloned()
    }
}

/// Props for the root Settings component.
///
/// All fields are optional — `SettingsApp` works standalone without any props.
/// When the Desktop provides `packages`, a "Packages" section appears in the sidebar.
#[derive(Props, Clone, PartialEq, Default)]
pub struct SettingsAppProps {
    /// Installed packages whose settings should be surfaced in the Packages section.
    /// When empty (default), the Packages section is hidden.
    #[props(default)]
    pub packages: Vec<PackageSettingsEntry>,

    /// Callback fired when the user saves a package setting.
    /// Receives `(package_id, field_key, new_value)`.
    #[props(default)]
    pub on_package_save: Option<EventHandler<(String, String, String)>>,
}

/// Root Settings component.
///
/// Pass `packages` + `on_package_save` props to enable the Packages section.
#[component]
pub fn SettingsApp(props: SettingsAppProps) -> Element {
    let has_packages = !props.packages.is_empty();
    let mut active = use_signal(|| SettingsSection::Appearance);

    let mut sidebar_items: Vec<SidebarItem> = SettingsSection::standard()
        .map(|s| SidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    if has_packages {
        let s = SettingsSection::Packages;
        sidebar_items.push(SidebarItem::new(s.id(), s.icon(), s.label()));
    }

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
                Sidebar {
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
                    if *active.read() == SettingsSection::Packages {
                        if has_packages {
                            PackageSettingsView {
                                packages: props.packages.clone(),
                                on_save: props.on_package_save
                                    .unwrap_or_else(|| EventHandler::new(|_| {})),
                            }
                        }
                    } else {
                        { active.read().render_panel() }
                    }
                }
            } // end sidebar + content row
        }
    }
}

/// Trait that gives a settings section the ability to render itself.
///
/// Extend settings without touching `SettingsApp` — implement this trait.
pub trait SettingsPanel {
    fn render_panel(&self) -> Element;
}

impl SettingsPanel for SettingsSection {
    fn render_panel(&self) -> Element {
        match self {
            Self::Appearance => rsx! { AppearanceSettings {} },
            Self::Language => rsx! { LanguageSettings {} },
            Self::ServiceRoles => rsx! { ServiceRoles {} },
            Self::Accounts => rsx! { AccountSettings {} },
            Self::Desktop => rsx! { DesktopSettings {} },
            Self::Browser => rsx! { BrowserSettings {} },
            Self::Shortcuts => rsx! { ShortcutsSettings {} },
            // Packages is rendered inline in SettingsApp (needs props data).
            Self::Packages => rsx! { div {} },
        }
    }
}
