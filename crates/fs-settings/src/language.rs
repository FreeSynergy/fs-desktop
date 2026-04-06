// fs-settings/src/language.rs — Language settings section (iced).
//
// State: LanguageState — global language + per-app overrides.
// View:  view_language(&SettingsApp) -> Element<Message>

use std::collections::HashMap;

use fs_db_desktop::package_registry::{PackageKind, PackageRegistry};
use fs_gui_engine_iced::iced::{
    widget::{button, column, row, scrollable, text, text_input},
    Alignment, Element, Length,
};
use fs_i18n;
use fs_manager_language::LanguageManager;
use serde::Deserialize;

use crate::app::{Message, SettingsApp};

// ── Public types ─────────────────────────────────────────────────────────────

/// A single locale entry from the Store's locale catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct LocaleEntry {
    pub code: String,
    pub name: String,
    pub version: String,
    pub completeness: u8,
    pub direction: String,
    pub path: Option<String>,
}

/// Built-in (always available, cannot be removed) languages.
pub const BUILTIN_LANGUAGES: &[(&str, &str)] = &[("en", "English")];

/// Returns the currently active language code from the user's locale inventory.
#[must_use]
pub fn load_active_language() -> String {
    LanguageManager::new().effective_settings().language
}

// ── LangEntry ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
struct LangEntry {
    code: String,
    name: String,
    builtin: bool,
}

fn load_installed() -> Vec<LangEntry> {
    let mut entries: Vec<LangEntry> = BUILTIN_LANGUAGES
        .iter()
        .map(|(c, n)| LangEntry {
            code: c.to_string(),
            name: n.to_string(),
            builtin: true,
        })
        .collect();
    let builtin_codes: Vec<&str> = BUILTIN_LANGUAGES.iter().map(|(c, _)| *c).collect();
    for pkg in PackageRegistry::by_kind(PackageKind::Language) {
        if !builtin_codes.contains(&pkg.id.as_str()) {
            entries.push(LangEntry {
                code: pkg.id,
                name: pkg.name,
                builtin: false,
            });
        }
    }
    entries
}

// ── AppOverrideForm ───────────────────────────────────────────────────────────

/// Inline form state for adding a new per-app language override.
#[derive(Clone, Default, Debug)]
pub struct AppOverrideForm {
    /// App ID the user is typing (e.g. "fs-browser").
    pub app_id: String,
    /// Selected language code for the override.
    pub lang_code: String,
}

impl AppOverrideForm {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.app_id.trim().is_empty() && !self.lang_code.is_empty()
    }
}

// ── LanguageState ─────────────────────────────────────────────────────────────

/// State for the Language settings section.
#[derive(Debug, Clone)]
pub struct LanguageState {
    /// Currently selected global language code.
    pub selected: String,
    /// All installed language entries.
    installed: Vec<LangEntry>,
    /// Per-app language overrides: app-id → lang-code.
    pub app_overrides: HashMap<String, String>,
    /// Form for adding a new per-app override.
    pub override_form: AppOverrideForm,
    /// Whether the per-app section is expanded.
    pub overrides_expanded: bool,
}

impl LanguageState {
    #[must_use]
    pub fn new() -> Self {
        let installed = load_installed();
        let mgr = LanguageManager::new();
        let effective = mgr.effective_settings();
        Self {
            selected: effective.language.clone(),
            installed,
            app_overrides: effective.app_overrides,
            override_form: AppOverrideForm::default(),
            overrides_expanded: false,
        }
    }

    pub fn save_global(&self) {
        let _ = LanguageManager::new().set_active(&self.selected);
    }

    pub fn set_app_override(&mut self, app_id: &str, lang: &str) {
        let _ = LanguageManager::new().set_app_language(app_id, Some(lang));
        self.app_overrides
            .insert(app_id.to_string(), lang.to_string());
    }

    pub fn remove_app_override(&mut self, app_id: &str) {
        let _ = LanguageManager::new().set_app_language(app_id, None);
        self.app_overrides.remove(app_id);
    }
}

impl Default for LanguageState {
    fn default() -> Self {
        Self::new()
    }
}

// ── LanguageSettings (public re-export type) ──────────────────────────────────

/// Public type alias kept for backwards compatibility with `lib.rs` re-exports.
pub struct LanguageSettings;

// ── view_language ─────────────────────────────────────────────────────────────

/// Render the Language settings section.
#[must_use]
pub fn view_language(app: &SettingsApp) -> Element<'_, Message> {
    let state = &app.language;

    // ── Global language picker ────────────────────────────────────────────────
    let lang_btns: Vec<Element<Message>> = state
        .installed
        .iter()
        .map(|entry| {
            let is_active = state.selected == entry.code;
            let label = if entry.builtin {
                format!(
                    "{} ({})",
                    entry.name,
                    fs_i18n::t("settings-language-builtin")
                )
            } else {
                entry.name.clone()
            };
            let code = entry.code.clone();
            button(text(label).size(13))
                .width(Length::Fill)
                .padding([8, 12])
                .style(if is_active {
                    fs_gui_engine_iced::iced::widget::button::primary
                } else {
                    fs_gui_engine_iced::iced::widget::button::secondary
                })
                .on_press(Message::LanguageSelected(code))
                .into()
        })
        .collect();

    let save_btn = button(text(fs_i18n::t("actions.save").to_string()).size(13))
        .padding([8, 20])
        .on_press(Message::SaveLanguage);

    // ── Per-app overrides section ─────────────────────────────────────────────
    let toggle_label = if state.overrides_expanded {
        fs_i18n::t("settings-language-per-app-hide").to_string()
    } else {
        fs_i18n::t("settings-language-per-app-show").to_string()
    };

    let override_toggle = button(text(toggle_label).size(12))
        .padding([5, 12])
        .on_press(Message::LanguageOverridesToggled);

    let overrides_section: Element<Message> = if state.overrides_expanded {
        let mut rows: Vec<Element<Message>> = state
            .app_overrides
            .iter()
            .map(|(app_id, lang)| {
                let aid = app_id.clone();
                row![
                    text(app_id.as_str()).size(12).width(Length::Fill),
                    text(lang.as_str()).size(12).width(80),
                    button(text("×").size(12))
                        .padding([3, 8])
                        .on_press(Message::LanguageAppOverrideRemove(aid)),
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .into()
            })
            .collect();

        // Add-override form
        let app_id_input = text_input(
            fs_i18n::t("settings-language-per-app-id-placeholder").as_ref(),
            &state.override_form.app_id,
        )
        .on_input(Message::LanguageAppIdChanged)
        .padding([5, 8])
        .width(Length::Fill);

        let lang_btns_small: Vec<Element<Message>> = state
            .installed
            .iter()
            .map(|entry| {
                let is_sel = state.override_form.lang_code == entry.code;
                let code = entry.code.clone();
                button(text(entry.code.as_str()).size(11))
                    .padding([3, 8])
                    .style(if is_sel {
                        fs_gui_engine_iced::iced::widget::button::primary
                    } else {
                        fs_gui_engine_iced::iced::widget::button::secondary
                    })
                    .on_press(Message::LanguageAppLangSelected(code))
                    .into()
            })
            .collect();

        let add_btn = {
            let b = button(text(fs_i18n::t("settings-language-per-app-add").to_string()).size(12))
                .padding([5, 12]);
            if state.override_form.is_valid() {
                b.on_press(Message::LanguageAppOverrideAdd)
            } else {
                b
            }
        };

        rows.push(
            column![
                text(fs_i18n::t("settings-language-per-app-form-title").to_string()).size(12),
                app_id_input,
                row(lang_btns_small).spacing(4),
                add_btn,
            ]
            .spacing(6)
            .padding([8, 0])
            .into(),
        );

        column(rows).spacing(6).into()
    } else {
        fs_gui_engine_iced::iced::widget::Space::with_height(0).into()
    };

    let content = column![
        text(fs_i18n::t("settings-language-default-label").to_string()).size(16),
        row![
            text(fs_i18n::t("settings-language-active").to_string())
                .size(13)
                .width(Length::Fill),
            text(state.selected.as_str()).size(13),
        ]
        .spacing(8),
        column(lang_btns).spacing(6),
        save_btn,
        row![
            text(fs_i18n::t("settings-language-per-app-title").to_string())
                .size(14)
                .width(Length::Fill),
            override_toggle,
        ]
        .align_y(Alignment::Center)
        .spacing(8),
        overrides_section,
    ]
    .spacing(12)
    .width(Length::Fill);

    scrollable(content).into()
}
