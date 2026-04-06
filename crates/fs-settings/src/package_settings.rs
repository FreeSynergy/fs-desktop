// fs-settings/src/package_settings.rs — Package settings view (iced).
//
// Aggregates all installed packages' settings into one place.
// Left: searchable package list. Right: selected package's config fields.

use fs_gui_engine_iced::iced::{
    widget::{button, checkbox, column, row, scrollable, text, text_input},
    Alignment, Element, Length,
};
use fs_i18n;
use fs_pkg::manageable::{ConfigField, ConfigFieldKind, ConfigValue};

use crate::app::{Message, SettingsApp};

// ── PackageSettingsEntry ──────────────────────────────────────────────────────

/// One installed package's settings data for the Settings Manager.
#[derive(Clone, PartialEq, Debug)]
pub struct PackageSettingsEntry {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub category: String,
    pub fields: Vec<SettingsFieldView>,
}

/// One config field extracted for the Settings Manager.
#[derive(Clone, PartialEq, Debug)]
pub struct SettingsFieldView {
    pub key: String,
    pub label: String,
    /// Always required — shown as a warning if empty.
    pub help: String,
    pub kind_tag: SettingsKindTag,
    pub current_value: String,
    pub required: bool,
    pub needs_restart: bool,
    pub options: Vec<(String, String)>,
}

/// Simplified kind for rendering.
#[derive(Clone, PartialEq, Debug)]
pub enum SettingsKindTag {
    Text,
    Password,
    Number,
    Bool,
    Select,
    Port,
    Path,
    Textarea,
}

impl PackageSettingsEntry {
    /// Build an entry from a `Manageable` implementor.
    pub fn from_manageable(pkg: &dyn fs_pkg::manageable::Manageable) -> Self {
        let meta = pkg.meta();
        let fields = pkg.config_fields().iter().map(field_to_view).collect();
        Self {
            id: meta.id.to_string(),
            name: meta.name.clone(),
            icon: meta.icon.clone(),
            category: meta.category.clone(),
            fields,
        }
    }
}

fn field_to_view(f: &ConfigField) -> SettingsFieldView {
    let (kind_tag, options) = match &f.kind {
        ConfigFieldKind::Password => (SettingsKindTag::Password, vec![]),
        ConfigFieldKind::Number { .. } => (SettingsKindTag::Number, vec![]),
        ConfigFieldKind::Bool => (SettingsKindTag::Bool, vec![]),
        ConfigFieldKind::Select { options } => (
            SettingsKindTag::Select,
            options
                .iter()
                .map(|o| (o.value.clone(), o.label.clone()))
                .collect(),
        ),
        ConfigFieldKind::Port => (SettingsKindTag::Port, vec![]),
        ConfigFieldKind::Path => (SettingsKindTag::Path, vec![]),
        ConfigFieldKind::Textarea => (SettingsKindTag::Textarea, vec![]),
        ConfigFieldKind::Url | ConfigFieldKind::LanguageCode | ConfigFieldKind::SemVer => {
            (SettingsKindTag::Text, vec![])
        }
        ConfigFieldKind::Tag { .. } | ConfigFieldKind::Text => (SettingsKindTag::Text, vec![]),
    };

    let current_value = match &f.value {
        ConfigValue::Text(s) => s.clone(),
        ConfigValue::Bool(b) => b.to_string(),
        ConfigValue::Number(n) => n.to_string(),
        ConfigValue::Port(p) => p.to_string(),
        ConfigValue::Empty => String::new(),
    };

    SettingsFieldView {
        key: f.key.clone(),
        label: f.label.clone(),
        help: f.help.clone(),
        kind_tag,
        current_value,
        required: f.required,
        needs_restart: f.needs_restart,
        options,
    }
}

// ── PackageSettingsView (public re-export type) ───────────────────────────────

/// Public type alias kept for backwards compatibility with `lib.rs` re-exports.
pub struct PackageSettingsView;

// ── PackageSettingsState ──────────────────────────────────────────────────────

/// Runtime state for the Packages settings section.
#[derive(Debug, Clone)]
pub struct PackageSettingsState {
    pub packages: Vec<PackageSettingsEntry>,
    pub selected_id: String,
    pub search: String,
}

impl PackageSettingsState {
    #[must_use]
    pub fn new(packages: Vec<PackageSettingsEntry>) -> Self {
        let selected_id = packages.first().map(|p| p.id.clone()).unwrap_or_default();
        Self {
            packages,
            selected_id,
            search: String::new(),
        }
    }

    pub fn on_field_changed(&mut self, pkg_id: &str, field_key: &str, value: &str) {
        if let Some(pkg) = self.packages.iter_mut().find(|p| p.id == pkg_id) {
            if let Some(field) = pkg.fields.iter_mut().find(|f| f.key == field_key) {
                field.current_value = value.to_string();
            }
        }
    }
}

// ── view_packages ─────────────────────────────────────────────────────────────

/// Render the Packages settings section.
pub fn view_packages(app: &SettingsApp) -> Element<'_, Message> {
    let state = &app.packages;

    if state.packages.is_empty() {
        return column![
            text(fs_i18n::t("settings-packages-title").to_string()).size(16),
            text(fs_i18n::t("settings-packages-empty").to_string()).size(13),
        ]
        .spacing(12)
        .into();
    }

    let q = state.search.to_lowercase();
    let filtered: Vec<&PackageSettingsEntry> = state
        .packages
        .iter()
        .filter(|p| {
            q.is_empty()
                || p.name.to_lowercase().contains(&q)
                || p.category.to_lowercase().contains(&q)
        })
        .collect();

    // Left sidebar: package list
    let search_input = text_input(
        fs_i18n::t("settings-packages-search-placeholder").as_ref(),
        &state.search,
    )
    .on_input(Message::PackageSearchChanged)
    .padding([6, 10])
    .width(Length::Fill);

    let pkg_btns: Vec<Element<Message>> = filtered
        .iter()
        .map(|p| {
            let is_active = p.id == state.selected_id;
            let id = p.id.clone();
            let icon = if p.icon.is_empty() {
                "Pkg"
            } else {
                p.icon.as_str()
            };
            button(
                row![
                    text(icon).size(11).width(24),
                    text(p.name.as_str()).size(12).width(Length::Fill),
                ]
                .align_y(Alignment::Center)
                .spacing(6),
            )
            .width(Length::Fill)
            .padding([6, 10])
            .style(if is_active {
                fs_gui_engine_iced::iced::widget::button::primary
            } else {
                fs_gui_engine_iced::iced::widget::button::text
            })
            .on_press(Message::PackageSelected(id))
            .into()
        })
        .collect();

    let sidebar = column![search_input, column(pkg_btns).spacing(2)]
        .spacing(8)
        .width(200);

    // Right panel: selected package's settings
    let detail: Element<Message> =
        if let Some(pkg) = state.packages.iter().find(|p| p.id == state.selected_id) {
            view_package_detail(pkg)
        } else {
            text(fs_i18n::t("settings-packages-select-hint").to_string())
                .size(13)
                .into()
        };

    row![
        sidebar,
        fs_gui_engine_iced::iced::widget::vertical_rule(1),
        scrollable(detail).width(Length::Fill),
    ]
    .spacing(16)
    .height(Length::Fill)
    .into()
}

fn view_package_detail(pkg: &PackageSettingsEntry) -> Element<'_, Message> {
    let header = row![
        text(pkg.icon.as_str()).size(18).width(32),
        column![
            text(pkg.name.as_str()).size(14),
            text(pkg.category.as_str()).size(11),
        ]
        .spacing(2),
    ]
    .align_y(Alignment::Center)
    .spacing(8);

    if pkg.fields.is_empty() {
        return column![
            header,
            text(fs_i18n::t("settings-packages-none-configured").to_string()).size(12),
        ]
        .spacing(12)
        .into();
    }

    let field_rows: Vec<Element<Message>> = pkg
        .fields
        .iter()
        .map(|field| view_field_row(&pkg.id, field))
        .collect();

    column![header, column(field_rows).spacing(12)]
        .spacing(16)
        .padding([0, 8])
        .into()
}

fn view_field_row<'a>(pkg_id: &'a str, field: &'a SettingsFieldView) -> Element<'a, Message> {
    let req_star = if field.required { " *" } else { "" };
    let label_text = format!("{}{}", field.label, req_star);

    let input_widget: Element<Message> = match field.kind_tag {
        SettingsKindTag::Bool => {
            let checked = field.current_value == "true";
            let pkg = pkg_id.to_string();
            let key = field.key.clone();
            checkbox("", checked)
                .on_toggle(move |v| {
                    Message::PackageFieldChanged(
                        pkg.clone(),
                        key.clone(),
                        if v { "true" } else { "false" }.to_string(),
                    )
                })
                .into()
        }
        SettingsKindTag::Select => {
            // Render select as a row of buttons
            let btns: Vec<Element<Message>> = field
                .options
                .iter()
                .map(|(val, lbl)| {
                    let is_active = &field.current_value == val;
                    let pkg = pkg_id.to_string();
                    let key = field.key.clone();
                    let v = val.clone();
                    button(text(lbl.as_str()).size(12))
                        .padding([4, 8])
                        .style(if is_active {
                            fs_gui_engine_iced::iced::widget::button::primary
                        } else {
                            fs_gui_engine_iced::iced::widget::button::secondary
                        })
                        .on_press(Message::PackageFieldChanged(pkg, key, v))
                        .into()
                })
                .collect();
            row(btns).spacing(4).into()
        }
        _ => {
            let pkg = pkg_id.to_string();
            let key = field.key.clone();
            text_input("", &field.current_value)
                .on_input(move |v| Message::PackageFieldChanged(pkg.clone(), key.clone(), v))
                .padding([6, 10])
                .width(Length::Fill)
                .into()
        }
    };

    let help: Element<Message> = if field.help.is_empty() {
        text(fs_i18n::t("settings-packages-no-help").to_string())
            .size(11)
            .into()
    } else {
        text(field.help.as_str()).size(11).into()
    };

    let restart: Element<Message> = if field.needs_restart {
        text(fs_i18n::t("settings-field-restart-required").to_string())
            .size(10)
            .into()
    } else {
        fs_gui_engine_iced::iced::widget::Space::with_height(0).into()
    };

    column![text(label_text).size(13), input_widget, help, restart,]
        .spacing(4)
        .into()
}
