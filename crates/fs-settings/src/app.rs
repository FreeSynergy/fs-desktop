// fs-settings/src/app.rs — root iced application for FreeSynergy Settings.
//
// Architecture:
//   SettingsApp  — owns all section state
//   Message      — flat enum for all actions
//   update()     — state transitions
//   view()       — sidebar + section content

use fs_gui_engine_iced::iced::{
    self,
    widget::{button, column, container, row, scrollable, text},
    Alignment, Element, Length, Task,
};
use fs_i18n;

use crate::accounts::AccountsState;
use crate::appearance::AppearanceState;
use crate::browser_settings::BrowserSettingsState;
use crate::desktop_settings::{DesktopConfig, DesktopTab};
use crate::language::LanguageState;
use crate::layout_settings::LayoutSettingsState;
use crate::package_settings::PackageSettingsState;
use crate::service_roles::ServiceRolesState;
use crate::shortcuts::ShortcutsState;

// ── SettingsSection ───────────────────────────────────────────────────────────

/// All settings sections in display order.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Language,
    ServiceRoles,
    Accounts,
    Desktop,
    Layout,
    Browser,
    Shortcuts,
    Packages,
}

impl SettingsSection {
    #[must_use]
    pub fn all() -> &'static [Self] {
        const ALL: &[SettingsSection] = &[
            SettingsSection::Appearance,
            SettingsSection::Language,
            SettingsSection::ServiceRoles,
            SettingsSection::Accounts,
            SettingsSection::Desktop,
            SettingsSection::Layout,
            SettingsSection::Browser,
            SettingsSection::Shortcuts,
            SettingsSection::Packages,
        ];
        ALL
    }

    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Appearance => "appearance",
            Self::Language => "language",
            Self::ServiceRoles => "service_roles",
            Self::Accounts => "accounts",
            Self::Desktop => "desktop",
            Self::Layout => "layout",
            Self::Browser => "browser",
            Self::Shortcuts => "shortcuts",
            Self::Packages => "packages",
        }
    }

    #[must_use]
    pub fn icon(&self) -> &str {
        match self {
            Self::Appearance => "Appearance",
            Self::Language => "Language",
            Self::ServiceRoles => "Roles",
            Self::Accounts => "Accounts",
            Self::Desktop => "Desktop",
            Self::Layout => "Layout",
            Self::Browser => "Browser",
            Self::Shortcuts => "Shortcuts",
            Self::Packages => "Packages",
        }
    }

    #[must_use]
    pub fn label(&self) -> String {
        let key = match self {
            Self::Appearance => "settings-section-appearance",
            Self::Language => "settings-section-language",
            Self::ServiceRoles => "settings-section-roles",
            Self::Accounts => "settings-section-accounts",
            Self::Desktop => "settings-section-desktop",
            Self::Layout => "settings-section-layout",
            Self::Browser => "settings-section-browser",
            Self::Shortcuts => "settings-section-shortcuts",
            Self::Packages => "settings-section-packages",
        };
        fs_i18n::t(key).into()
    }
}

// ── Message ───────────────────────────────────────────────────────────────────

/// All settings application messages — flat enum.
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    SectionSelected(SettingsSection),

    // Appearance
    ThemeSelected(String),
    AnimationsToggled(bool),
    SaveAppearance,

    // Desktop
    DesktopTabSelected(DesktopTab),
    DesktopTaskbarPositionChanged(String),
    DesktopFocusPolicyChanged(String),
    DesktopTitleBarStyleChanged(String),
    DesktopClickStyleChanged(String),
    DesktopAnimationsDisabledToggled(bool),
    DesktopWorkspaceColumnsChanged(String),
    SaveDesktop,

    // Browser
    BrowserSearchEngineSelected(String),
    SaveBrowser,

    // Language
    LanguageSelected(String),
    SaveLanguage,
    LanguageOverridesToggled,
    LanguageAppIdChanged(String),
    LanguageAppLangSelected(String),
    LanguageAppOverrideAdd,
    LanguageAppOverrideRemove(String),

    // Service Roles
    ServiceRoleChanged(String, String),
    SaveServiceRoles,

    // Accounts
    AccountsToggleAddForm,
    AccountsFormNameChanged(String),
    AccountsFormDiscoveryUrlChanged(String),
    AccountsFormClientIdChanged(String),
    AccountsFormScopesChanged(String),
    AccountsAddProvider,
    AccountsRemoveProvider(usize),
    AccountsToggleProvider(usize),
    SaveAccounts,

    // Shortcuts
    ShortcutsSearchChanged(String),
    ShortcutsStartRecording(String),
    ShortcutsStopRecording,
    ShortcutsResetAction(String),

    // Layout
    LayoutToggleSection(fs_render::layout::ShellKind),
    SaveLayout,

    // Packages
    PackageSelected(String),
    PackageSearchChanged(String),
    PackageFieldChanged(String, String, String),

    // Status
    StatusClear,
}

// ── SettingsApp ───────────────────────────────────────────────────────────────

/// Root settings application state.
pub struct SettingsApp {
    pub active_section: SettingsSection,
    pub status_msg: Option<String>,
    pub appearance: AppearanceState,
    pub desktop: DesktopState,
    pub layout: LayoutSettingsState,
    pub browser: BrowserSettingsState,
    pub language: LanguageState,
    pub service_roles: ServiceRolesState,
    pub accounts: AccountsState,
    pub shortcuts: ShortcutsState,
    pub packages: PackageSettingsState,
}

/// Desktop section sub-state (active tab + loaded config).
#[derive(Debug, Clone)]
pub struct DesktopState {
    pub active_tab: DesktopTab,
    pub config: DesktopConfig,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            active_tab: DesktopTab::General,
            config: DesktopConfig::load(),
        }
    }
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsApp {
    /// Create a new settings application, loading all configs from disk.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_section: SettingsSection::Appearance,
            status_msg: None,
            appearance: AppearanceState::new(),
            desktop: DesktopState::default(),
            layout: LayoutSettingsState::new(),
            browser: BrowserSettingsState::new(),
            language: LanguageState::new(),
            service_roles: ServiceRolesState::new(),
            accounts: AccountsState::new(),
            shortcuts: ShortcutsState::new(),
            packages: PackageSettingsState::new(vec![]),
        }
    }

    // ── Update ────────────────────────────────────────────────────────────────

    /// Handle a message and return the next task.
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SectionSelected(s) => {
                self.active_section = s;
                self.status_msg = None;
            }

            // Appearance
            Message::ThemeSelected(t) => self.appearance.selected_theme = t,
            Message::AnimationsToggled(v) => self.appearance.animations_enabled = v,
            Message::SaveAppearance => {
                self.appearance.save();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }

            // Desktop
            Message::DesktopTabSelected(t) => self.desktop.active_tab = t,
            Message::DesktopTaskbarPositionChanged(v) => {
                self.desktop.config.taskbar_pos = match v.as_str() {
                    "top" => crate::desktop_settings::TaskbarPosition::Top,
                    "left" => crate::desktop_settings::TaskbarPosition::Left,
                    "right" => crate::desktop_settings::TaskbarPosition::Right,
                    _ => crate::desktop_settings::TaskbarPosition::Bottom,
                };
            }
            Message::DesktopFocusPolicyChanged(v) => {
                self.desktop.config.window.focus_policy = match v.as_str() {
                    "focus_follows_mouse" => {
                        crate::desktop_settings::FocusPolicy::FocusFollowsMouse
                    }
                    "strict_follows_mouse" => {
                        crate::desktop_settings::FocusPolicy::StrictFollowsMouse
                    }
                    _ => crate::desktop_settings::FocusPolicy::Click,
                };
            }
            Message::DesktopTitleBarStyleChanged(v) => {
                self.desktop.config.window.title_bar_style = match v.as_str() {
                    "compact" => crate::desktop_settings::TitleBarStyle::Compact,
                    "minimal" => crate::desktop_settings::TitleBarStyle::Minimal,
                    "hidden" => crate::desktop_settings::TitleBarStyle::Hidden,
                    _ => crate::desktop_settings::TitleBarStyle::Full,
                };
            }
            Message::DesktopClickStyleChanged(v) => {
                self.desktop.config.click.icon_click = match v.as_str() {
                    "single" => crate::desktop_settings::ClickStyle::Single,
                    _ => crate::desktop_settings::ClickStyle::Double,
                };
            }
            Message::DesktopAnimationsDisabledToggled(v) => {
                self.desktop.config.animation.disabled = v;
            }
            Message::DesktopWorkspaceColumnsChanged(v) => {
                if let Ok(n) = v.parse::<u32>() {
                    self.desktop.config.workspace.columns = n.clamp(1, 6);
                }
            }
            Message::SaveDesktop => {
                self.desktop.config.save();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }

            // Browser
            Message::BrowserSearchEngineSelected(id) => {
                self.browser.config.search_engine = id;
            }
            Message::SaveBrowser => {
                self.browser.config.save();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }

            // Language
            Message::LanguageSelected(code) => {
                self.language.selected = code;
            }
            Message::SaveLanguage => {
                self.language.save_global();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }
            Message::LanguageOverridesToggled => {
                self.language.overrides_expanded = !self.language.overrides_expanded;
            }
            Message::LanguageAppIdChanged(v) => {
                self.language.override_form.app_id = v;
            }
            Message::LanguageAppLangSelected(code) => {
                self.language.override_form.lang_code = code;
            }
            Message::LanguageAppOverrideAdd => {
                let form = self.language.override_form.clone();
                if form.is_valid() {
                    self.language
                        .set_app_override(form.app_id.trim(), &form.lang_code);
                    self.language.override_form = crate::language::AppOverrideForm::default();
                    self.status_msg = Some(fs_i18n::t("settings-saved").into());
                }
            }
            Message::LanguageAppOverrideRemove(app_id) => {
                self.language.remove_app_override(&app_id);
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }

            // Service Roles
            Message::ServiceRoleChanged(role, provider) => {
                self.service_roles.config.set(role, provider);
            }
            Message::SaveServiceRoles => {
                match crate::service_roles::save_role_assignments(&self.service_roles.config) {
                    Ok(()) => {
                        self.status_msg = Some(fs_i18n::t("settings-saved").into());
                    }
                    Err(e) => {
                        self.status_msg = Some(format!("Save error: {e}"));
                    }
                }
            }

            // Accounts
            Message::AccountsToggleAddForm => {
                self.accounts.show_add = !self.accounts.show_add;
                self.accounts.form = crate::accounts::AddProviderForm::default();
            }
            Message::AccountsFormNameChanged(v) => self.accounts.form.name = v,
            Message::AccountsFormDiscoveryUrlChanged(v) => self.accounts.form.discovery_url = v,
            Message::AccountsFormClientIdChanged(v) => self.accounts.form.client_id = v,
            Message::AccountsFormScopesChanged(v) => self.accounts.form.scopes = v,
            Message::AccountsAddProvider => {
                if self.accounts.form.is_valid() {
                    let provider = self.accounts.form.build();
                    self.accounts.providers.push(provider);
                    self.accounts.save();
                    self.accounts.show_add = false;
                    self.accounts.form = crate::accounts::AddProviderForm::default();
                    self.status_msg = Some(fs_i18n::t("settings-saved").into());
                }
            }
            Message::AccountsRemoveProvider(idx) => {
                if idx < self.accounts.providers.len() {
                    self.accounts.providers.remove(idx);
                    self.accounts.save();
                }
            }
            Message::AccountsToggleProvider(idx) => {
                if let Some(p) = self.accounts.providers.get_mut(idx) {
                    p.enabled = !p.enabled;
                    self.accounts.save();
                }
            }
            Message::SaveAccounts => {
                self.accounts.save();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }

            // Shortcuts
            Message::ShortcutsSearchChanged(q) => self.shortcuts.search = q,
            Message::ShortcutsStartRecording(id) => {
                self.shortcuts.recording = Some(id);
            }
            Message::ShortcutsStopRecording => self.shortcuts.recording = None,
            Message::ShortcutsResetAction(id) => {
                self.shortcuts.config.custom.remove(&id);
                self.shortcuts.config.save();
            }

            // Layout
            Message::LayoutToggleSection(kind) => {
                // Load → toggle → save → reload state for next render.
                let layout_path = layout_config_path();
                let content = std::fs::read_to_string(&layout_path).unwrap_or_default();
                let mut proxy: Option<LayoutProxy> = toml::from_str(&content).ok();
                if let Some(ref mut p) = proxy {
                    for section in &mut p.sections {
                        if section_kind_matches(&section.kind, &kind) {
                            section.visible = !section.visible;
                        }
                    }
                    if let Ok(new_content) = toml::to_string_pretty(p) {
                        if let Some(parent) = layout_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(&layout_path, new_content);
                    }
                }
                self.layout.reload();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }
            Message::SaveLayout => {
                self.layout.reload();
                self.status_msg = Some(fs_i18n::t("settings-saved").into());
            }

            // Packages
            Message::PackageSelected(id) => self.packages.selected_id = id,
            Message::PackageSearchChanged(q) => self.packages.search = q,
            Message::PackageFieldChanged(pkg_id, field_key, value) => {
                self.packages.on_field_changed(&pkg_id, &field_key, &value);
            }

            Message::StatusClear => self.status_msg = None,
        }
        Task::none()
    }

    // ── View ──────────────────────────────────────────────────────────────────

    /// Render the full settings UI.
    #[must_use]
    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = self.view_section();

        let status_bar: Element<Message> = if let Some(msg) = &self.status_msg {
            container(text(msg.as_str()).size(12))
                .width(Length::Fill)
                .padding([4, 16])
                .into()
        } else {
            iced::widget::Space::with_height(0).into()
        };

        column![row![sidebar, content].height(Length::Fill), status_bar,].into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let items: Vec<Element<Message>> = SettingsSection::all()
            .iter()
            .map(|s| {
                let is_active = *s == self.active_section;
                let label = s.label();
                let btn = button(
                    row![text(s.icon()).size(12).width(60), text(label).size(13),]
                        .align_y(Alignment::Center)
                        .spacing(4),
                )
                .width(Length::Fill)
                .padding([8, 12])
                .style(if is_active {
                    iced::widget::button::primary
                } else {
                    iced::widget::button::text
                })
                .on_press(Message::SectionSelected(s.clone()));
                btn.into()
            })
            .collect();

        let col = column(items).spacing(2).padding(8);
        container(scrollable(col))
            .width(200)
            .height(Length::Fill)
            .into()
    }

    fn view_section(&self) -> Element<'_, Message> {
        let content: Element<Message> = match self.active_section {
            SettingsSection::Appearance => crate::appearance::view_appearance(self),
            SettingsSection::Desktop => crate::desktop_settings::view_desktop(self),
            SettingsSection::Layout => crate::layout_settings::view_layout_settings(self),
            SettingsSection::Browser => crate::browser_settings::view_browser(self),
            SettingsSection::Language => crate::language::view_language(self),
            SettingsSection::ServiceRoles => crate::service_roles::view_service_roles(self),
            SettingsSection::Accounts => crate::accounts::view_accounts(self),
            SettingsSection::Shortcuts => crate::shortcuts::view_shortcuts(self),
            SettingsSection::Packages => crate::package_settings::view_packages(self),
        };
        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .into()
    }
}

// ── Layout helpers (used by update() for LayoutToggleSection) ─────────────────

fn layout_config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("freesynergy")
        .join("desktop")
        .join("desktop-layout.toml")
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct LayoutSectionProxy {
    kind: String,
    #[serde(default = "default_true_app")]
    visible: bool,
    #[serde(flatten)]
    extra: toml::Value,
}

fn default_true_app() -> bool {
    true
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct LayoutProxy {
    sections: Vec<LayoutSectionProxy>,
}

fn section_kind_matches(kind_str: &str, kind: &fs_render::layout::ShellKind) -> bool {
    use fs_render::layout::ShellKind;
    matches!(
        (kind_str, kind),
        ("topbar", ShellKind::Topbar)
            | ("sidebar", ShellKind::Sidebar)
            | ("bottombar", ShellKind::Bottombar)
            | ("main", ShellKind::Main)
    )
}
