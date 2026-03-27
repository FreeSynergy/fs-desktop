// package_settings.rs — Settings Manager: all packages in one place.
//
// Design principle: Every package can have its settings changed individually
// (in its own Manager), but the Settings Manager aggregates ALL package
// settings into one view — so the user can find and change anything without
// opening package-by-package.
//
// Every setting MUST have a help text. Fields without help are flagged.
//
// Usage:
//   The caller collects PackageSettingsEntry for every installed Manageable
//   package and passes the list to PackageSettingsView.
//
// Layout:
//   Left  — Package list with search filter
//   Right — Selected package's config fields (with inline help text)

use dioxus::prelude::*;

use fs_pkg::manageable::{ConfigField, ConfigFieldKind, ConfigValue};

// ── PackageSettingsEntry ──────────────────────────────────────────────────────

/// One installed package's settings data for the Settings Manager.
///
/// Constructed by the caller (Desktop/Node) from any `Manageable` implementor.
#[derive(Clone, PartialEq, Debug)]
pub struct PackageSettingsEntry {
    /// Unique package id (e.g. `"proxy/zentinel"`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Emoji or icon identifier.
    pub icon: String,
    /// Category string (e.g. `"deploy.proxy"`).
    pub category: String,
    /// Config fields from `Manageable::config_fields()`.
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
    /// Options for select fields.
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
        ConfigFieldKind::Text => (SettingsKindTag::Text, vec![]),
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
        // Typed-value kinds — rendered as text inputs until dedicated UI controls exist.
        ConfigFieldKind::Url | ConfigFieldKind::LanguageCode | ConfigFieldKind::SemVer => {
            (SettingsKindTag::Text, vec![])
        }
        ConfigFieldKind::Tag { .. } => (SettingsKindTag::Text, vec![]),
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

// ── PackageSettingsView ───────────────────────────────────────────────────────

const SETTINGS_CSS: &str = r#"
.fs-pkgsettings-field {
    padding: 14px 0;
    border-bottom: 1px solid var(--fs-border);
}
.fs-pkgsettings-field:last-child { border-bottom: none; }

.fs-pkgsettings-label {
    font-size: 13px; font-weight: 600; color: var(--fs-text-primary);
    margin-bottom: 6px;
}
.fs-pkgsettings-label--required::after {
    content: " *"; color: var(--fs-error);
}
.fs-pkgsettings-input {
    width: 100%; box-sizing: border-box;
    padding: 7px 10px; font-size: 13px;
    background: var(--fs-bg-input); color: var(--fs-text-primary);
    border: 1px solid var(--fs-border); border-radius: var(--fs-radius-md);
    outline: none; transition: border-color 120ms;
}
.fs-pkgsettings-input:focus { border-color: var(--fs-border-focus); }
.fs-pkgsettings-help {
    font-size: 12px; color: var(--fs-text-muted); margin-top: 5px; line-height: 1.5;
}
.fs-pkgsettings-help--missing {
    font-size: 12px; color: var(--fs-warning); margin-top: 5px; font-style: italic;
}
.fs-pkgsettings-restart {
    font-size: 11px; color: var(--fs-warning); margin-top: 3px;
}
"#;

/// Props for the Settings Manager view.
#[derive(Props, Clone, PartialEq)]
pub struct PackageSettingsViewProps {
    /// All installed packages with their settings.
    pub packages: Vec<PackageSettingsEntry>,

    /// Fired when the user saves a setting: `(package_id, field_key, new_value_string)`.
    pub on_save: EventHandler<(String, String, String)>,
}

/// The Settings Manager: all package settings in one place.
///
/// Left sidebar: searchable package list.
/// Right panel:  selected package's config fields with help text.
#[component]
pub fn PackageSettingsView(props: PackageSettingsViewProps) -> Element {
    let first_id = props
        .packages
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_default();
    let mut selected_id: Signal<String> = use_signal(|| first_id);
    let mut search: Signal<String> = use_signal(String::new);

    let filtered: Vec<PackageSettingsEntry> = {
        let q = search.read().to_lowercase();
        props
            .packages
            .iter()
            .filter(|p| {
                q.is_empty()
                    || p.name.to_lowercase().contains(&q)
                    || p.category.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    };

    let selected = props
        .packages
        .iter()
        .find(|p| p.id == *selected_id.read())
        .cloned();

    rsx! {
        style { "{SETTINGS_CSS}" }

        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-bg-base); color: var(--fs-text-primary);",

            // ── Left sidebar: package list ────────────────────────────────────
            div {
                style: "width: 220px; flex-shrink: 0; overflow: hidden; display: flex; \
                        flex-direction: column; background: var(--fs-bg-sidebar); \
                        border-right: 1px solid var(--fs-border);",

                // Search
                div {
                    style: "padding: 10px 10px 8px; flex-shrink: 0;",
                    input {
                        style: "width: 100%; box-sizing: border-box; padding: 6px 10px; \
                                font-size: 12px; border-radius: var(--fs-radius-md); \
                                background: var(--fs-bg-input); color: var(--fs-text-primary); \
                                border: 1px solid var(--fs-border); outline: none;",
                        placeholder: "Search packages…",
                        value: "{search}",
                        oninput: move |e| search.set(e.value()),
                    }
                }

                // Package list
                div {
                    style: "flex: 1; overflow-y: auto;",
                    if filtered.is_empty() {
                        div {
                            style: "padding: 12px 14px; font-size: 12px; color: var(--fs-text-muted);",
                            "No packages match."
                        }
                    }
                    for p in filtered {
                        SettingsSidebarRow {
                            key: "{p.id}",
                            pkg: p.clone(),
                            is_active: p.id == *selected_id.read(),
                            on_select: move |id: String| selected_id.set(id),
                        }
                    }
                }
            }

            // ── Right panel: selected package's settings ──────────────────────
            div {
                style: "flex: 1; overflow-y: auto; padding: 20px 28px;",

                if let Some(pkg) = selected {
                    PackageSettingsPanel {
                        pkg: pkg.clone(),
                        on_save: props.on_save,
                    }
                } else {
                    div {
                        style: "height: 100%; display: flex; align-items: center; \
                                justify-content: center; font-size: 13px; \
                                color: var(--fs-text-muted);",
                        "Select a package to view its settings."
                    }
                }
            }
        }
    }
}

// ── SettingsSidebarRow ────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct SettingsSidebarRowProps {
    pkg: PackageSettingsEntry,
    is_active: bool,
    on_select: EventHandler<String>,
}

#[component]
fn SettingsSidebarRow(props: SettingsSidebarRowProps) -> Element {
    let pkg_id = props.pkg.id.clone();
    let has_fields = !props.pkg.fields.is_empty();
    let bg = if props.is_active {
        "var(--fs-sidebar-active-bg)"
    } else {
        "transparent"
    };
    let color = if props.is_active {
        "var(--fs-sidebar-active)"
    } else {
        "var(--fs-text-secondary)"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 9px; \
                    padding: 8px 12px; cursor: pointer; \
                    border-radius: 6px; margin: 1px 6px; font-size: 13px; \
                    background: {bg}; color: {color};",
            onclick: move |_| props.on_select.call(pkg_id.clone()),

            span {
                style: "font-size: 15px; min-width: 18px; text-align: center; flex-shrink: 0;",
                if props.pkg.icon.is_empty() { "📦" } else { "{props.pkg.icon}" }
            }
            div {
                style: "flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                "{props.pkg.name}"
            }
            if !has_fields {
                span {
                    style: "font-size: 10px; color: var(--fs-text-muted); flex-shrink: 0;",
                    "—"
                }
            }
        }
    }
}

// ── PackageSettingsPanel ──────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct PackageSettingsPanelProps {
    pkg: PackageSettingsEntry,
    on_save: EventHandler<(String, String, String)>,
}

#[component]
fn PackageSettingsPanel(props: PackageSettingsPanelProps) -> Element {
    let pkg = &props.pkg;

    rsx! {
        div {
            // Package header
            div {
                style: "display: flex; align-items: center; gap: 12px; margin-bottom: 20px; \
                        padding-bottom: 16px; border-bottom: 1px solid var(--fs-border);",
                div {
                    style: "width: 40px; height: 40px; border-radius: 8px; font-size: 20px; \
                            background: var(--fs-bg-elevated); display: flex; \
                            align-items: center; justify-content: center; flex-shrink: 0;",
                    if pkg.icon.is_empty() { "📦" } else { "{pkg.icon}" }
                }
                div {
                    h3 {
                        style: "margin: 0; font-size: 15px; font-weight: 700; color: var(--fs-text-bright);",
                        "{pkg.name}"
                    }
                    if !pkg.category.is_empty() {
                        span {
                            style: "font-size: 11px; color: var(--fs-text-muted);",
                            "{pkg.category}"
                        }
                    }
                }
            }

            // Settings fields
            if pkg.fields.is_empty() {
                div {
                    style: "font-size: 13px; color: var(--fs-text-muted); padding: 8px 0;",
                    "This package has no configurable settings."
                }
            } else {
                for field in &pkg.fields {
                    SettingsFieldRow {
                        key: "{field.key}",
                        field: field.clone(),
                        pkg_id: pkg.id.clone(),
                        on_save: props.on_save,
                    }
                }
            }
        }
    }
}

// ── SettingsFieldRow ──────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct SettingsFieldRowProps {
    field: SettingsFieldView,
    pkg_id: String,
    on_save: EventHandler<(String, String, String)>,
}

#[component]
fn SettingsFieldRow(props: SettingsFieldRowProps) -> Element {
    let field = &props.field;
    let key_c = field.key.clone();
    let pkg_id_c = props.pkg_id.clone();
    let mut val: Signal<String> = use_signal(|| field.current_value.clone());

    rsx! {
        div {
            class: "fs-pkgsettings-field",

            div {
                class: if field.required { "fs-pkgsettings-label fs-pkgsettings-label--required" }
                       else { "fs-pkgsettings-label" },
                "{field.label}"
            }

            // Input widget
            match field.kind_tag {
                SettingsKindTag::Bool => rsx! {
                    div {
                        style: "display: flex; align-items: center; gap: 10px; margin: 4px 0;",
                        input {
                            r#type: "checkbox",
                            checked: "{field.current_value == \"true\"}",
                            style: "width: 16px; height: 16px; cursor: pointer; accent-color: var(--fs-primary);",
                            onchange: move |e| {
                                let v = if e.checked() { "true" } else { "false" };
                                props.on_save.call((pkg_id_c.clone(), key_c.clone(), v.to_string()));
                            },
                        }
                    }
                },
                SettingsKindTag::Select => rsx! {
                    select {
                        class: "fs-pkgsettings-input",
                        style: "margin: 4px 0;",
                        onchange: move |e| {
                            props.on_save.call((pkg_id_c.clone(), key_c.clone(), e.value()));
                        },
                        for (opt_val, opt_label) in &field.options {
                            option {
                                value: "{opt_val}",
                                selected: *opt_val == field.current_value,
                                "{opt_label}"
                            }
                        }
                    }
                },
                SettingsKindTag::Password => rsx! {
                    input {
                        r#type: "password",
                        class: "fs-pkgsettings-input",
                        style: "margin: 4px 0;",
                        value: "{val}",
                        placeholder: "••••••••",
                        oninput: move |e| val.set(e.value()),
                        onblur: {
                            let pk = props.pkg_id.clone(); let kk = field.key.clone();
                            move |_| props.on_save.call((pk.clone(), kk.clone(), val.read().clone()))
                        },
                    }
                },
                SettingsKindTag::Textarea => rsx! {
                    textarea {
                        class: "fs-pkgsettings-input",
                        style: "margin: 4px 0; min-height: 72px; resize: vertical;",
                        value: "{val}",
                        oninput: move |e| val.set(e.value()),
                        onblur: {
                            let pk = props.pkg_id.clone(); let kk = field.key.clone();
                            move |_| props.on_save.call((pk.clone(), kk.clone(), val.read().clone()))
                        },
                    }
                },
                _ => rsx! {
                    input {
                        r#type: "text",
                        class: "fs-pkgsettings-input",
                        style: "margin: 4px 0;",
                        value: "{val}",
                        oninput: move |e| val.set(e.value()),
                        onblur: {
                            let pk = props.pkg_id.clone(); let kk = field.key.clone();
                            move |_| props.on_save.call((pk.clone(), kk.clone(), val.read().clone()))
                        },
                    }
                },
            }

            // Help text — always shown, warning if missing
            if field.help.is_empty() {
                div {
                    class: "fs-pkgsettings-help--missing",
                    "⚠ No help text defined for this setting."
                }
            } else {
                div { class: "fs-pkgsettings-help", "{field.help}" }
            }

            if field.needs_restart {
                div { class: "fs-pkgsettings-restart", "⟳ Restart required to apply this change." }
            }
        }
    }
}
