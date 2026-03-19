/// Language settings — interface language, locale formats, translation contributions.
///
/// Two-layer model:
///   Store  → system-wide defaults (language, formats, auto-update)
///   Inventory → per-user overrides stored in ~/.config/fsn/locale_settings.toml
use dioxus::prelude::*;
use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
use fsn_i18n;
use fsn_manager_language::{
    DateFormat, LanguageManager, LocaleSettings, NumberFormat, TimeFormat,
    git_contributor::{ContributorStatus, GitContributorCheck},
};
use fsn_store::{LocaleEntry, Manifest, StoreClient};
use serde::Deserialize;

use crate::translation_editor::TranslationEditor;

/// Minimal package type used only to satisfy `fetch_catalog`'s `Manifest` bound.
#[derive(Deserialize)]
struct MinPkg {
    id:       String,
    name:     String,
    version:  String,
    category: String,
}

impl Manifest for MinPkg {
    fn id(&self)       -> &str { &self.id }
    fn name(&self)     -> &str { &self.name }
    fn version(&self)  -> &str { &self.version }
    fn category(&self) -> &str { &self.category }
}

/// Newtype wrapper around `Signal<String>` used as a Dioxus context for the active language.
///
/// A dedicated type avoids collisions with other `Signal<String>` contexts.
/// Provided by `Desktop`, consumed by `LanguageSettings` when the user switches languages.
#[derive(Clone, Copy)]
pub struct LangContext(pub dioxus::prelude::Signal<String>);

/// Local view of a `LocaleEntry` — implements PartialEq for use as Dioxus prop.
#[derive(Clone, PartialEq, Debug)]
struct LocaleInfo {
    code:         String,
    name:         String,
    version:      String,
    completeness: u8,
    direction:    String,
    path:         Option<String>,
}

impl From<LocaleEntry> for LocaleInfo {
    fn from(l: LocaleEntry) -> Self {
        Self {
            code:         l.code,
            name:         l.name,
            version:      l.version,
            completeness: l.completeness,
            direction:    l.direction,
            path:         l.path,
        }
    }
}

/// Built-in (always available, cannot be removed) languages.
pub const BUILTIN_LANGUAGES: &[(&str, &str)] = &[("en", "English")];

/// A language entry for display/selection.
#[derive(Clone, PartialEq)]
struct LangEntry {
    code:    String,
    name:    String,
    builtin: bool,
}

// ── Persistence (active language file) ────────────────────────────────────────

fn active_language_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".local/share/fsn/settings/language")
}

pub fn load_active_language() -> String {
    std::fs::read_to_string(active_language_path())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "en".to_string())
}

fn save_active_language(code: &str) {
    let path = active_language_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, code);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn load_installed() -> Vec<LangEntry> {
    let mut entries: Vec<LangEntry> = BUILTIN_LANGUAGES
        .iter()
        .map(|(code, name)| LangEntry {
            code:    code.to_string(),
            name:    name.to_string(),
            builtin: true,
        })
        .collect();

    let builtin_codes: Vec<&str> = BUILTIN_LANGUAGES.iter().map(|(c, _)| *c).collect();
    for pkg in PackageRegistry::by_kind("language") {
        if !builtin_codes.contains(&pkg.id.as_str()) {
            entries.push(LangEntry { code: pkg.id, name: pkg.name, builtin: false });
        }
    }
    entries
}

async fn install_language_pack(locale: LocaleInfo) -> Result<(), String> {
    let home    = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let fsn_dir = std::path::PathBuf::from(&home).join(".local/share/fsn");

    let base = locale.path.clone()
        .unwrap_or_else(|| format!("Node/i18n/{}", locale.code));
    let url = format!("{base}/ui.toml");

    let file_path = match StoreClient::node_store().fetch_raw(&url).await {
        Ok(content) => {
            let dest_dir = fsn_dir.join("i18n").join(&locale.code);
            std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            let dest = dest_dir.join("ui.toml");
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            Some(dest.to_string_lossy().into_owned())
        }
        Err(e) => {
            tracing::warn!("Language pack download failed (registering anyway): {e}");
            None
        }
    };

    PackageRegistry::install(InstalledPackage {
        id:           locale.code.clone(),
        name:         locale.name.clone(),
        kind:         "language".into(),
        version:      locale.version.clone(),
        icon:         String::new(),
        file_path,
        installed_by: None,
    })
    .map_err(|e| format!("Registry error: {e}"))
}

// ── LanguageSettings ───────────────────────────────────────────────────────────

#[component]
pub fn LanguageSettings() -> Element {
    let installed          = use_signal(load_installed);
    let mut selected       = use_signal(load_active_language);
    let mut show_available = use_signal(|| false);
    let mut saved_msg      = use_signal(|| Option::<bool>::None);

    // Inventory settings (user overrides on top of Store defaults).
    let mgr       = LanguageManager::new();
    let effective = mgr.effective_settings();
    let mut inv   = use_signal(LocaleSettings::load_inventory);

    // Translation editor: which language is open (None = editor closed).
    let mut editor_lang: Signal<Option<(String, String)>> = use_signal(|| None);

    // Git contributor status — start from cache, refresh in background.
    let mut contrib = use_signal(|| {
        GitContributorCheck::cached().unwrap_or(ContributorStatus::Unknown)
    });
    {
        let mut contrib = contrib.clone();
        use_future(move || async move {
            if *contrib.read() == ContributorStatus::Unknown {
                let result = GitContributorCheck::check_and_cache();
                contrib.set(result);
            }
        });
    }

    // If the translation editor is open, render it full-height.
    if let Some((code, name)) = editor_lang.read().clone() {
        return rsx! {
            TranslationEditor {
                lang_code: code,
                lang_name: name,
                on_close:  move |_| editor_lang.set(None),
            }
        };
    }

    let count      = installed.read().len();
    let list_style = if count >= 8 {
        "max-height: 240px; overflow-y: auto; border: 1px solid var(--fsn-color-border-default); \
         border-radius: var(--fsn-radius-md); scrollbar-width: thin;"
    } else {
        "border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);"
    };

    rsx! {
        div {
            class: "fsd-language",
            style: "padding: 24px; max-width: 560px; display: flex; flex-direction: column; gap: 28px;",

            // ══ Section 1: Interface Language ══════════════════════════════════
            div {
                SectionHeading { label: fsn_i18n::t("settings.language.title") }

                div { style: "margin-bottom: 12px;",
                    label {
                        style: "display: block; font-weight: 500; margin-bottom: 8px; font-size: 13px;",
                        {fsn_i18n::t("settings.language.interface_label")}
                        span {
                            style: "margin-left: 8px; font-size: 12px; font-weight: 400; \
                                    color: var(--fsn-color-text-muted);",
                            {fsn_i18n::t_with("settings.language.installed_count",
                                &[("count", &count.to_string())])}
                        }
                    }
                    div { style: "{list_style}",
                        for entry in installed.read().clone() {
                            LangRow {
                                key:      "{entry.code}",
                                code:     entry.code.clone(),
                                name:     entry.name.clone(),
                                selected: *selected.read() == entry.code,
                                builtin:  entry.builtin,
                                on_select: {
                                    let code = entry.code.clone();
                                    move |_| *selected.write() = code.clone()
                                },
                                on_remove: {
                                    let code = entry.code.clone();
                                    let mut installed = installed.clone();
                                    move |_| {
                                        let _ = PackageRegistry::remove(&code);
                                        installed.write().retain(|e| e.code != code);
                                    }
                                },
                            }
                        }
                    }
                }

                // Install more toggle
                button {
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 16px; \
                            background: var(--fsn-color-bg-surface); \
                            border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); font-size: 13px; \
                            cursor: pointer; color: var(--fsn-color-primary); width: 100%; \
                            margin-bottom: 8px;",
                    onclick: move |_| {
                        let cur = *show_available.read();
                        show_available.set(!cur);
                    },
                    span { "🌐" }
                    span {
                        {
                            if *show_available.read() { fsn_i18n::t("settings.language.btn_hide") }
                            else { fsn_i18n::t("settings.language.btn_show_more") }
                        }
                    }
                }

                if *show_available.read() {
                    AvailableLanguages {
                        installed_ids: installed.read().iter().map(|e| e.code.clone()).collect(),
                        on_installed: {
                            let mut installed = installed.clone();
                            move |entry: LangEntry| installed.write().push(entry)
                        },
                    }
                }

                // Apply button
                div { style: "display: flex; align-items: center; gap: 12px; margin-top: 4px;",
                    button {
                        style: "padding: 8px 24px; background: var(--fsn-color-primary); \
                                color: white; border: none; border-radius: var(--fsn-radius-md); \
                                cursor: pointer;",
                        onclick: move |_| {
                            let code = selected.read().clone();
                            save_active_language(&code);

                            // Load language pack into i18n before switching.
                            if code != "en" {
                                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                                let pack = std::path::PathBuf::from(home)
                                    .join(".local/share/fsn/i18n")
                                    .join(&code)
                                    .join("ui.toml");
                                if let Ok(content) = std::fs::read_to_string(&pack) {
                                    let _ = fsn_i18n::add_toml_lang(&code, &content);
                                }
                            }
                            fsn_i18n::set_active_lang(&code);

                            // Persist to inventory.
                            let _ = LanguageManager::new().set_active(&code);

                            if let Some(LangContext(mut sig)) =
                                dioxus::prelude::try_consume_context::<LangContext>()
                            {
                                *sig.write() = code;
                            }
                            saved_msg.set(Some(true));
                        },
                        {fsn_i18n::t("actions.apply")}
                    }
                    if saved_msg.read().is_some() {
                        span {
                            style: "font-size: 12px; color: var(--fsn-color-text-muted);",
                            {fsn_i18n::t("settings.language.applied")}
                        }
                    }
                }
            }

            // ══ Section 2: Locale Formats ══════════════════════════════════════
            div {
                SectionHeading { label: fsn_i18n::t("settings.language.formats.title") }

                // Note explaining two-layer model
                div {
                    style: "font-size: 11px; color: var(--fsn-color-text-muted); \
                            margin-bottom: 14px; padding: 8px 12px; \
                            background: var(--fsn-color-bg-surface); \
                            border-radius: var(--fsn-radius-sm); \
                            border-left: 3px solid var(--fsn-color-primary);",
                    {fsn_i18n::t("settings.language.formats.store_note")}
                }

                div { style: "display: flex; flex-direction: column; gap: 16px;",

                    // ── Fallback language ──────────────────────────────────────
                    FormatRow {
                        label: fsn_i18n::t("settings.language.formats.fallback"),
                        hint:  fsn_i18n::t("settings.language.formats.fallback_hint"),
                        content: rsx! {
                            select {
                                style: SELECT_STYLE,
                                value: inv.read().fallback_language.clone()
                                    .unwrap_or_else(|| effective.fallback_language.clone()),
                                onchange: {
                                    let mut inv = inv.clone();
                                    move |e: Event<FormData>| {
                                        inv.write().fallback_language = Some(e.value());
                                        let _ = inv.read().save_inventory();
                                    }
                                },
                                for entry in installed.read().clone() {
                                    option { value: "{entry.code}", "{entry.name}" }
                                }
                            }
                        },
                    }

                    // ── Date format ────────────────────────────────────────────
                    FormatRow {
                        label: fsn_i18n::t("settings.language.formats.date"),
                        hint:  String::new(),
                        content: rsx! {
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                for fmt in DateFormat::all() {
                                    {
                                        let is_active = inv.read().date_format.as_ref()
                                            .unwrap_or(&effective.date_format) == fmt;
                                        let f = fmt.clone();
                                        rsx! {
                                            button {
                                                key: "{f.label()}",
                                                style: format!(
                                                    "padding: 5px 12px; font-size: 12px; \
                                                     border-radius: var(--fsn-radius-sm); cursor: pointer; \
                                                     border: 1px solid {}; background: {}; color: {};",
                                                    if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" },
                                                    if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-bg-surface)" },
                                                    if is_active { "white" } else { "var(--fsn-color-text-primary)" },
                                                ),
                                                onclick: {
                                                    let mut inv = inv.clone();
                                                    let f2 = f.clone();
                                                    move |_| {
                                                        inv.write().date_format = Some(f2.clone());
                                                        let _ = inv.read().save_inventory();
                                                    }
                                                },
                                                "{f.label()}"
                                                span {
                                                    style: "margin-left: 6px; opacity: 0.6; font-size: 11px;",
                                                    "({f.example()})"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }

                    // ── Time format ────────────────────────────────────────────
                    FormatRow {
                        label: fsn_i18n::t("settings.language.formats.time"),
                        hint:  String::new(),
                        content: rsx! {
                            div { style: "display: flex; gap: 8px;",
                                for fmt in &[TimeFormat::H24, TimeFormat::H12] {
                                    {
                                        let is_active = inv.read().time_format.as_ref()
                                            .unwrap_or(&effective.time_format) == fmt;
                                        let f = fmt.clone();
                                        rsx! {
                                            button {
                                                key: "{f.label()}",
                                                style: format!(
                                                    "padding: 5px 14px; font-size: 12px; \
                                                     border-radius: var(--fsn-radius-sm); cursor: pointer; \
                                                     border: 1px solid {}; background: {}; color: {};",
                                                    if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" },
                                                    if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-bg-surface)" },
                                                    if is_active { "white" } else { "var(--fsn-color-text-primary)" },
                                                ),
                                                onclick: {
                                                    let mut inv = inv.clone();
                                                    let f2 = f.clone();
                                                    move |_| {
                                                        inv.write().time_format = Some(f2.clone());
                                                        let _ = inv.read().save_inventory();
                                                    }
                                                },
                                                "{f.label()}"
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }

                    // ── Number format ──────────────────────────────────────────
                    FormatRow {
                        label: fsn_i18n::t("settings.language.formats.number"),
                        hint:  String::new(),
                        content: rsx! {
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                for fmt in NumberFormat::all() {
                                    {
                                        let is_active = inv.read().number_format.as_ref()
                                            .unwrap_or(&effective.number_format) == fmt;
                                        let f = fmt.clone();
                                        rsx! {
                                            button {
                                                key: "{f.label()}",
                                                style: format!(
                                                    "padding: 5px 14px; font-size: 13px; \
                                                     font-family: monospace; \
                                                     border-radius: var(--fsn-radius-sm); cursor: pointer; \
                                                     border: 1px solid {}; background: {}; color: {};",
                                                    if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" },
                                                    if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-bg-surface)" },
                                                    if is_active { "white" } else { "var(--fsn-color-text-primary)" },
                                                ),
                                                onclick: {
                                                    let mut inv = inv.clone();
                                                    let f2 = f.clone();
                                                    move |_| {
                                                        inv.write().number_format = Some(f2.clone());
                                                        let _ = inv.read().save_inventory();
                                                    }
                                                },
                                                "{f.label()}"
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }

                    // ── Auto-update packs ──────────────────────────────────────
                    FormatRow {
                        label: fsn_i18n::t("settings.language.formats.auto_update"),
                        hint:  fsn_i18n::t("settings.language.formats.auto_update_hint"),
                        content: rsx! {
                            label {
                                style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: inv.read().auto_update_packs
                                        .unwrap_or(effective.auto_update_packs),
                                    onchange: {
                                        let mut inv = inv.clone();
                                        move |e: Event<FormData>| {
                                            let checked = e.value() == "true";
                                            inv.write().auto_update_packs = Some(checked);
                                            let _ = inv.read().save_inventory();
                                        }
                                    },
                                }
                                span {
                                    style: "font-size: 13px;",
                                    {fsn_i18n::t("settings.language.formats.auto_update_label")}
                                }
                            }
                        },
                    }
                }
            }

            // ══ Section 3: Translation Contributions ════════════════════════════
            div {
                SectionHeading { label: fsn_i18n::t("settings.language.contrib.title") }

                p {
                    style: "font-size: 13px; color: var(--fsn-color-text-muted); margin: 0 0 14px 0;",
                    {fsn_i18n::t("settings.language.contrib.description")}
                }

                // SSH / GitHub contributor status card
                div {
                    style: "padding: 12px 16px; background: var(--fsn-color-bg-surface); \
                            border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); margin-bottom: 14px; \
                            display: flex; align-items: center; gap: 12px;",

                    {
                        let (icon, text, sub, color) = match contrib.read().clone() {
                            ContributorStatus::Authenticated { ref github_user } => (
                                "✓",
                                format!("GitHub: @{github_user}"),
                                fsn_i18n::t("settings.language.contrib.ssh_ok"),
                                "#16a34a",
                            ),
                            ContributorStatus::NotAuthenticated => (
                                "✕",
                                fsn_i18n::t("settings.language.contrib.ssh_none"),
                                fsn_i18n::t("settings.language.contrib.ssh_none_hint"),
                                "var(--fsn-color-text-muted)",
                            ),
                            ContributorStatus::Unknown => (
                                "…",
                                fsn_i18n::t("settings.language.contrib.ssh_checking"),
                                String::new(),
                                "var(--fsn-color-text-muted)",
                            ),
                        };
                        rsx! {
                            span {
                                style: "font-size: 18px; color: {color}; font-weight: 700; \
                                        width: 24px; text-align: center; flex-shrink: 0;",
                                "{icon}"
                            }
                            div { style: "flex: 1;",
                                div {
                                    style: "font-size: 13px; font-weight: 500; color: {color};",
                                    "{text}"
                                }
                                if !sub.is_empty() {
                                    div {
                                        style: "font-size: 11px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                                        "{sub}"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        style: "padding: 5px 12px; font-size: 11px; background: none; \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-sm); cursor: pointer; \
                                color: var(--fsn-color-text-muted); white-space: nowrap;",
                        onclick: {
                            let mut contrib = contrib.clone();
                            move |_| {
                                GitContributorCheck::clear_cache();
                                contrib.set(ContributorStatus::Unknown);
                            }
                        },
                        {fsn_i18n::t("settings.language.contrib.btn_recheck")}
                    }
                }

                // Language buttons → open Translation Editor
                p {
                    style: "font-size: 12px; color: var(--fsn-color-text-muted); margin: 0 0 10px 0;",
                    {fsn_i18n::t("settings.language.contrib.pick_language")}
                }
                div { style: "display: flex; flex-wrap: wrap; gap: 8px;",
                    for entry in installed.read().clone().into_iter().filter(|e| e.code != "en") {
                        {
                            let code = entry.code.clone();
                            let name = entry.name.clone();
                            rsx! {
                                button {
                                    key: "{code}",
                                    style: "padding: 7px 16px; font-size: 13px; \
                                            background: var(--fsn-color-bg-surface); \
                                            border: 1px solid var(--fsn-color-border-default); \
                                            border-radius: var(--fsn-radius-md); cursor: pointer; \
                                            color: var(--fsn-color-text-primary);",
                                    onclick: {
                                        let code2 = code.clone();
                                        let name2 = name.clone();
                                        move |_| editor_lang.set(Some((code2.clone(), name2.clone())))
                                    },
                                    "{name}"
                                    span {
                                        style: "margin-left: 6px; font-size: 11px; \
                                                color: var(--fsn-color-text-muted);",
                                        "({code})"
                                    }
                                }
                            }
                        }
                    }

                    // "Start new language" — only for authenticated contributors
                    if let ContributorStatus::Authenticated { .. } = contrib.read().clone() {
                        button {
                            style: "padding: 7px 16px; font-size: 13px; background: transparent; \
                                    border: 1px dashed var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); cursor: pointer; \
                                    color: var(--fsn-color-primary);",
                            onclick: move |_| {
                                editor_lang.set(Some(("new".to_string(), "New Language".to_string())));
                            },
                            {fsn_i18n::t("settings.language.contrib.btn_new_lang")}
                        }
                    }
                }
            }
        }
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

const SELECT_STYLE: &str =
    "padding: 5px 10px; font-size: 13px; \
     background: var(--fsn-color-bg-surface); \
     border: 1px solid var(--fsn-color-border-default); \
     border-radius: var(--fsn-radius-sm); \
     color: var(--fsn-color-text-primary);";

#[component]
fn SectionHeading(label: String) -> Element {
    rsx! {
        h3 {
            style: "margin: 0 0 14px 0; font-size: 14px; font-weight: 600; \
                    color: var(--fsn-color-text-primary); padding-bottom: 8px; \
                    border-bottom: 1px solid var(--fsn-color-border-default);",
            "{label}"
        }
    }
}

#[component]
fn FormatRow(label: String, hint: String, content: Element) -> Element {
    rsx! {
        div { style: "display: flex; align-items: flex-start; gap: 16px;",
            div { style: "width: 160px; flex-shrink: 0; padding-top: 6px;",
                div {
                    style: "font-size: 13px; font-weight: 500; \
                            color: var(--fsn-color-text-primary);",
                    "{label}"
                }
                if !hint.is_empty() {
                    div {
                        style: "font-size: 11px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                        "{hint}"
                    }
                }
            }
            div { style: "flex: 1;", {content} }
        }
    }
}

// ── LangRow ────────────────────────────────────────────────────────────────────

#[component]
fn LangRow(
    code:      String,
    name:      String,
    selected:  bool,
    builtin:   bool,
    on_select: EventHandler<MouseEvent>,
    on_remove: EventHandler<MouseEvent>,
) -> Element {
    let bg = if selected {
        "background: var(--fsn-color-primary); color: white;"
    } else {
        "background: transparent; color: var(--fsn-color-text-primary);"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; padding: 10px 14px; \
                    cursor: pointer; transition: background 0.1s; {bg}",
            onclick: on_select,
            span { style: "font-size: 16px;",
                if selected { "◉" } else { "○" }
            }
            span { style: "font-size: 14px;", "{name}" }
            span { style: "margin-left: auto; font-size: 12px; opacity: 0.6;", "{code}" }
            if !builtin {
                button {
                    style: "margin-left: 8px; padding: 2px 8px; font-size: 11px; \
                            background: transparent; border: 1px solid currentColor; \
                            border-radius: var(--fsn-radius-sm); cursor: pointer; opacity: 0.6;",
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        on_remove.call(e);
                    },
                    "✕"
                }
            }
        }
    }
}

// ── AvailableLanguages ─────────────────────────────────────────────────────────

#[component]
fn AvailableLanguages(
    installed_ids: Vec<String>,
    on_installed:  EventHandler<LangEntry>,
) -> Element {
    let all_locales: Signal<Vec<LocaleInfo>>    = use_signal(Vec::new);
    let mut loading: Signal<bool>               = use_signal(|| true);
    let mut error:   Signal<Option<String>>     = use_signal(|| None);
    let busy:        Signal<Option<String>>     = use_signal(|| None);

    {
        let all_locales = all_locales.clone();
        use_future(move || {
            let mut all_locales = all_locales.clone();
            async move {
                match StoreClient::node_store()
                    .fetch_catalog::<MinPkg>("Node", false)
                    .await
                {
                    Ok(catalog) => {
                        all_locales.set(
                            catalog.locales.into_iter().map(LocaleInfo::from).collect()
                        );
                        error.set(None);
                    }
                    Err(e) => error.set(Some(format!("Could not load catalog: {e}"))),
                }
                loading.set(false);
            }
        });
    }

    let available: Vec<LocaleInfo> = all_locales
        .read()
        .iter()
        .filter(|l| !installed_ids.contains(&l.code))
        .cloned()
        .collect();

    rsx! {
        div {
            style: "border: 1px solid var(--fsn-color-border-default); \
                    border-radius: var(--fsn-radius-md); margin-bottom: 16px;",

            div {
                style: "padding: 8px 14px; border-bottom: 1px solid var(--fsn-color-border-default); \
                        font-size: 12px; font-weight: 500; color: var(--fsn-color-text-muted);",
                {fsn_i18n::t("settings.language.available_heading")}
            }

            if *loading.read() {
                div {
                    style: "padding: 16px; text-align: center; \
                            color: var(--fsn-color-text-muted); font-size: 13px;",
                    {fsn_i18n::t("labels.loading")}
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "padding: 12px 14px; color: var(--fsn-color-error); font-size: 13px;",
                    "{err}"
                }
            } else if available.is_empty() {
                div {
                    style: "padding: 16px; text-align: center; \
                            color: var(--fsn-color-text-muted); font-size: 13px;",
                    {fsn_i18n::t("settings.language.all_installed")}
                }
            } else {
                div {
                    style: "max-height: 280px; overflow-y: auto; scrollbar-width: thin;",
                    for locale in available {
                        AvailableLangRow {
                            key:        "{locale.code}",
                            locale:     locale.clone(),
                            installing: busy.read().as_deref() == Some(locale.code.as_str()),
                            on_install: {
                                let mut busy = busy.clone();
                                move |l: LocaleInfo| {
                                    let id   = l.code.clone();
                                    let name = l.name.clone();
                                    busy.set(Some(id.clone()));
                                    let mut busy = busy.clone();
                                    let entry = LangEntry { code: id.clone(), name, builtin: false };
                                    let cb = on_installed.clone();
                                    spawn(async move {
                                        match install_language_pack(l).await {
                                            Ok(()) => cb.call(entry),
                                            Err(e) => tracing::error!("Install failed: {e}"),
                                        }
                                        busy.set(None);
                                    });
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

// ── AvailableLangRow ───────────────────────────────────────────────────────────

#[component]
fn AvailableLangRow(
    locale:     LocaleInfo,
    installing: bool,
    on_install: EventHandler<LocaleInfo>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; padding: 8px 14px; \
                    border-bottom: 1px solid var(--fsn-color-border-default); \
                    color: var(--fsn-color-text-primary);",
            span { style: "font-size: 14px; flex: 1;", "{locale.name}" }
            span { style: "font-size: 12px; color: var(--fsn-color-text-muted);", "{locale.code}" }
            if locale.completeness < 100 {
                span {
                    style: "font-size: 11px; color: var(--fsn-color-text-muted); \
                            background: var(--fsn-color-bg-overlay); padding: 2px 6px; \
                            border-radius: 999px;",
                    "{locale.completeness}%"
                }
            }
            button {
                style: "padding: 4px 12px; font-size: 12px; \
                        background: var(--fsn-color-primary); color: white; \
                        border: none; border-radius: var(--fsn-radius-sm); \
                        cursor: pointer; min-width: 60px;",
                disabled: installing,
                onclick: {
                    let locale = locale.clone();
                    move |_| on_install.call(locale.clone())
                },
                { if installing { fsn_i18n::t("labels.loading") } else { fsn_i18n::t("actions.install") } }
            }
        }
    }
}
