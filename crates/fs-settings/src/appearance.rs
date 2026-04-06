// fs-settings/src/appearance.rs — Appearance settings section (iced).
//
// State: AppearanceState — loads theme list from ThemeRegistry + custom themes dir.
// View:  view_appearance(&SettingsApp) -> Element<Message>

use std::path::PathBuf;

use fs_gui_engine_iced::iced::{
    widget::{button, checkbox, column, container, row, scrollable, text},
    Alignment, Element, Length,
};
use fs_i18n;
use fs_theme::{ThemeEngine, ThemeRegistry};

use crate::app::{Message, SettingsApp};

// ── Persistence helpers ───────────────────────────────────────────────────────

fn themes_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("freesynergy")
        .join("themes")
}

fn load_persisted_theme() -> Option<String> {
    let path = crate::config_path("appearance.toml");
    let content = std::fs::read_to_string(path).ok()?;
    let val: toml::Value = toml::from_str(&content).ok()?;
    val.get("theme")?.as_str().map(str::to_string)
}

fn load_animations_enabled() -> bool {
    let path = crate::config_path("appearance.toml");
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let Ok(val) = toml::from_str::<toml::Value>(&content) else {
        return true;
    };
    val.get("animations_enabled")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
}

// ── Theme loading ─────────────────────────────────────────────────────────────

/// Build a `ThemeRegistry` populated with the built-in theme + any .toml files
/// found in `~/.local/share/freesynergy/themes/`.
fn build_registry() -> ThemeRegistry {
    let mut reg = ThemeRegistry::default();
    let dir = themes_dir();
    if !dir.exists() {
        return reg;
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return reg;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            if let Ok(engine) = ThemeEngine::from_toml(&path) {
                reg.register(engine.theme().clone());
            }
        }
    }
    reg
}

// ── AppearanceState ───────────────────────────────────────────────────────────

/// State for the Appearance settings section.
#[derive(Debug, Clone)]
pub struct AppearanceState {
    /// Currently selected theme name.
    pub selected_theme: String,
    pub animations_enabled: bool,
    /// All available theme names (built-in + installed from themes dir).
    pub available_themes: Vec<String>,
}

impl AppearanceState {
    #[must_use]
    pub fn new() -> Self {
        let reg = build_registry();
        let available_themes: Vec<String> = reg.names().iter().map(|&s| s.to_string()).collect();
        let selected_theme = load_persisted_theme()
            .filter(|n| available_themes.contains(n))
            .unwrap_or_else(|| reg.active().name.clone());
        Self {
            selected_theme,
            animations_enabled: load_animations_enabled(),
            available_themes,
        }
    }

    pub fn save(&self) {
        let path = crate::config_path("appearance.toml");
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let content = format!(
            "theme = \"{}\"\nanimations_enabled = {}\n",
            self.selected_theme, self.animations_enabled
        );
        let _ = std::fs::write(&path, content);
    }
}

impl Default for AppearanceState {
    fn default() -> Self {
        Self::new()
    }
}

// ── view_appearance ───────────────────────────────────────────────────────────

/// Render the Appearance settings section.
pub fn view_appearance(app: &SettingsApp) -> Element<'_, Message> {
    let state = &app.appearance;

    let theme_buttons: Vec<Element<Message>> = state
        .available_themes
        .iter()
        .map(|name| {
            let is_active = state.selected_theme == *name;
            button(text(name.as_str()).size(13))
                .width(Length::Fill)
                .padding([8, 12])
                .style(if is_active {
                    fs_gui_engine_iced::iced::widget::button::primary
                } else {
                    fs_gui_engine_iced::iced::widget::button::secondary
                })
                .on_press(Message::ThemeSelected(name.clone()))
                .into()
        })
        .collect();

    let theme_col = column(theme_buttons).spacing(6);

    let anim_row = row![
        text(fs_i18n::t("settings-appearance-animations").to_string())
            .size(13)
            .width(Length::Fill),
        checkbox("", state.animations_enabled).on_toggle(Message::AnimationsToggled),
    ]
    .align_y(Alignment::Center)
    .spacing(8);

    let save_btn = button(text(fs_i18n::t("actions.save").to_string()).size(13))
        .padding([8, 20])
        .on_press(Message::SaveAppearance);

    let content = column![
        text(fs_i18n::t("settings-appearance-color-theme").to_string()).size(16),
        theme_col,
        text(fs_i18n::t("settings-appearance-animations").to_string()).size(14),
        anim_row,
        save_btn,
    ]
    .spacing(16)
    .width(Length::Fill);

    scrollable(container(content).width(Length::Fill)).into()
}
