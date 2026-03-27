/// Language settings — panel sidebar (Default / Installed / Install) + detail pane.
///
/// The sidebar is the shared `Sidebar { mode: SidebarMode::Panel }` component —
/// no custom sidebar code here.  The domain enum `LangPanel` maps sidebar item
/// keys to the concrete pane to render on the right.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem as NavItem, SidebarMode, SidebarSection};
use fs_db_desktop::package_registry::{InstalledPackage, PackageKind, PackageRegistry};
use fs_i18n;
use fs_manager_language::{
    git_contributor::{ContributorStatus, GitContributorCheck},
    DateFormat, FormatVariant, HasFlag, Language, LanguageManager, LocaleSettings, NumberFormat,
    TimeFormat,
};
use fs_store::StoreReader;
use serde::Deserialize;

use crate::translation_editor::TranslationEditor;

// ── Store catalog helper ─────────────────────────────────────────────────────

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

/// TOML structure of the locale catalog file in the Store.
#[derive(Deserialize)]
struct LocaleCatalog {
    #[serde(default)]
    locales: Vec<LocaleEntry>,
}

// ── Public types ─────────────────────────────────────────────────────────────

/// Newtype wrapper around `Signal<String>` used as a Dioxus context for the active language.
///
/// Deprecated: use `AppContext.locale` instead. Kept for backward compatibility.
#[deprecated(note = "Use AppContext.locale from fs_components::AppContext instead")]
#[derive(Clone, Copy)]
pub struct LangContext(pub dioxus::prelude::Signal<String>);

/// Built-in (always available, cannot be removed) languages.
pub const BUILTIN_LANGUAGES: &[(&str, &str)] = &[("en", "English")];

/// Returns the currently active language code, read from the user's locale inventory.
/// Falls back to "en" when no preference is saved.
pub fn load_active_language() -> String {
    LanguageManager::new().effective_settings().language
}

// ── Internal types ───────────────────────────────────────────────────────────

/// Local view of a `LocaleEntry` — implements PartialEq for use as Dioxus prop.
#[derive(Clone, PartialEq, Debug)]
struct LocaleInfo {
    code: String,
    name: String,
    version: String,
    completeness: u8,
    direction: String,
    path: Option<String>,
}

impl From<LocaleEntry> for LocaleInfo {
    fn from(l: LocaleEntry) -> Self {
        Self {
            code: l.code,
            name: l.name,
            version: l.version,
            completeness: l.completeness,
            direction: l.direction,
            path: l.path,
        }
    }
}

/// A language entry for display / selection.
#[derive(Clone, PartialEq)]
struct LangEntry {
    code: String,
    name: String,
    builtin: bool,
}

/// Which panel is currently shown in the detail pane.
///
/// Maps directly to sidebar item keys so that `Sidebar.on_select` can drive
/// the pane without any manual match blocks.
#[derive(Clone, PartialEq, Debug)]
enum LangPanel {
    /// Active language picker + locale formats.
    Default,
    /// Info / Edit view for a specific installed language.
    Language(String),
    /// Browse & install from the Store.
    Install,
}

impl LangPanel {
    fn to_key(&self) -> String {
        match self {
            Self::Default => "default".into(),
            Self::Language(code) => code.clone(),
            Self::Install => "install".into(),
        }
    }

    fn from_key(key: &str, installed: &[LangEntry]) -> Self {
        match key {
            "install" => Self::Install,
            "default" => Self::Default,
            code if installed.iter().any(|e| e.code == code) => Self::Language(code.into()),
            _ => Self::Default,
        }
    }
}

/// Tabs in the per-language detail pane.
#[derive(Clone, PartialEq, Debug)]
enum LangDetailTab {
    Info,
    Edit,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

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

async fn install_language_pack(locale: LocaleInfo) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let fs_dir = std::path::PathBuf::from(&home).join(".local/share/fsn");
    let base = locale
        .path
        .clone()
        .unwrap_or_else(|| format!("Node/i18n/{}", locale.code));
    let url = format!("{base}/ui.toml");

    let file_path = match StoreReader::official().fetch_raw(&url).await {
        Ok(content) => {
            let dest = fs_dir.join("i18n").join(&locale.code).join("ui.toml");
            if let Some(p) = dest.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            Some(dest.to_string_lossy().into_owned())
        }
        Err(e) => {
            tracing::warn!("Language pack download failed (registering anyway): {e}");
            None
        }
    };

    PackageRegistry::install(InstalledPackage {
        id: locale.code.clone(),
        name: locale.name.clone(),
        kind: PackageKind::Language,
        version: locale.version.clone(),
        icon: String::new(),
        file_path,
        installed_by: None,
        pinned: false,
    })
    .map_err(|e| format!("Registry error: {e}"))
}

// ── LanguageSettings (root) ──────────────────────────────────────────────────

#[component]
pub fn LanguageSettings() -> Element {
    let installed = use_signal(load_installed);
    let mut panel = use_signal(|| LangPanel::Default);
    let mut editor_lang: Signal<Option<(String, String)>> = use_signal(|| None);

    // Full-screen translation editor replaces the whole view when open.
    if let Some((code, name)) = editor_lang.read().clone() {
        return rsx! {
            TranslationEditor {
                lang_code: code,
                lang_name: name,
                on_close: move |_| editor_lang.set(None),
            }
        };
    }

    let sel = panel.read().clone();
    let detail_code = if let LangPanel::Language(c) = &sel {
        Some(c.clone())
    } else {
        None
    };

    // Build sidebar items from the installed language list.
    // Flag SVG is used as icon when available; falls back to "🌐".
    // Builtin languages get a "✦" badge.
    let lang_items: Vec<NavItem> = installed
        .read()
        .iter()
        .map(|e| {
            let flag = Language::from_code(&e.code).flag_svg().to_string();
            let icon = if flag.is_empty() { "🌐".into() } else { flag };
            let item = NavItem::new(e.code.clone(), icon, e.name.clone());
            if e.builtin {
                item.with_badge("✦")
            } else {
                item
            }
        })
        .collect();

    rsx! {
        div {
            class: "fs-language",
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // ── Left Sidebar — the shared Sidebar component, Panel mode ───
            Sidebar {
                mode: SidebarMode::Panel,
                sections: vec![
                    SidebarSection::untitled(vec![
                        NavItem::new("default", "⚙", fs_i18n::t("settings.language.default_label")),
                    ]),
                    SidebarSection::new(
                        fs_i18n::t("settings.language.section.installed"),
                        lang_items,
                    ),
                ],
                pinned_items: vec![
                    NavItem::new("install", "➕", fs_i18n::t("settings.language.tabs.install")),
                ],
                active_id: sel.to_key(),
                on_select: move |id: String| {
                    panel.set(LangPanel::from_key(&id, &installed.read()));
                },
                // Right-click removes a non-builtin language.
                on_context_menu: {
                    let mut installed  = installed;
                    let mut panel_sig  = panel;
                    move |id: String| {
                        let is_builtin = BUILTIN_LANGUAGES.iter().any(|(c, _)| *c == id.as_str());
                        if !is_builtin {
                            let _ = PackageRegistry::remove(&id);
                            installed.write().retain(|e| e.code != id);
                            if *panel_sig.read() == LangPanel::Language(id.clone()) {
                                panel_sig.set(LangPanel::Default);
                            }
                        }
                    }
                },
            }

            // ── Right Detail Pane ─────────────────────────────────────────
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                if sel == LangPanel::Default {
                    DefaultPane { installed: installed.read().clone() }
                }

                if let Some(code) = detail_code {
                    {
                        let entry = installed.read().iter().find(|e| e.code == code).cloned();
                        if let Some(e) = entry {
                            rsx! {
                                LanguageDetailPane {
                                    entry: e,
                                    on_edit: {
                                        let mut el = editor_lang;
                                        move |pair| el.set(Some(pair))
                                    },
                                }
                            }
                        } else {
                            rsx! { div {} }
                        }
                    }
                }

                if sel == LangPanel::Install {
                    InstallPane {
                        installed_ids: installed.read().iter().map(|e| e.code.clone()).collect(),
                        on_installed: {
                            let mut installed = installed;
                            move |entry: LangEntry| installed.write().push(entry)
                        },
                    }
                }
            }
        }
    }
}

// ── DefaultPane ──────────────────────────────────────────────────────────────

/// Active-language picker + locale format settings.
#[component]
fn DefaultPane(installed: Vec<LangEntry>) -> Element {
    let mut selected = use_signal(load_active_language);
    let mut saved_msg = use_signal(|| Option::<bool>::None);
    let inv = use_signal(LocaleSettings::load_inventory);

    let mgr = LanguageManager::new();
    let effective = mgr.effective_settings();

    rsx! {
        div {
            style: "flex: 1; overflow-y: auto;",
            div {
                style: "padding: 24px; max-width: 560px; \
                        display: flex; flex-direction: column; gap: 28px;",

                // ── Active language picker ────────────────────────────────
                div {
                    SectionHeading { label: fs_i18n::t("settings.language.title") }
                    {
                        let count      = installed.len();
                        let list_style = if count >= 8 {
                            "max-height: 240px; overflow-y: auto; \
                             border: 1px solid var(--fs-color-border-default); \
                             border-radius: var(--fs-radius-md); scrollbar-width: thin;"
                        } else {
                            "border: 1px solid var(--fs-color-border-default); \
                             border-radius: var(--fs-radius-md);"
                        };
                        rsx! {
                            div { style: "margin-bottom: 12px;",
                                label {
                                    style: "display: block; font-weight: 500; \
                                            margin-bottom: 8px; font-size: 13px;",
                                    {fs_i18n::t("settings.language.interface_label")}
                                    span {
                                        style: "margin-left: 8px; font-size: 12px; font-weight: 400; \
                                                color: var(--fs-color-text-muted);",
                                        {fs_i18n::t_with(
                                            "settings.language.installed_count",
                                            &[("count", &count.to_string())]
                                        )}
                                    }
                                }
                                div { style: "{list_style}",
                                    for entry in installed.clone() {
                                        LangRow {
                                            key:      "{entry.code}",
                                            code:     entry.code.clone(),
                                            name:     entry.name.clone(),
                                            selected: *selected.read() == entry.code,
                                            on_select: {
                                                let code = entry.code.clone();
                                                move |_| *selected.write() = code.clone()
                                            },
                                        }
                                    }
                                }
                            }
                            div { style: "display: flex; align-items: center; gap: 12px;",
                                button {
                                    style: "padding: 8px 24px; \
                                            background: var(--fs-color-primary); \
                                            color: white; border: none; \
                                            border-radius: var(--fs-radius-md); cursor: pointer;",
                                    onclick: {
                                        let mut inv = inv;
                                        move |_| {
                                            let code = selected.read().clone();
                                            // Load user-installed pack from disk before switching.
                                            if code != "en" {
                                                let home = std::env::var("HOME")
                                                    .unwrap_or_else(|_| ".".into());
                                                let pack = std::path::PathBuf::from(home)
                                                    .join(".local/share/fsn/i18n")
                                                    .join(&code)
                                                    .join("ui.toml");
                                                if let Ok(content) = std::fs::read_to_string(&pack) {
                                                    let _ = fs_i18n::add_toml_lang(&code, &content);
                                                }
                                            }
                                            let _ = LanguageManager::new().set_active(&code);
                                            fs_i18n::set_active_lang(&code);
                                            if let Some(mut ctx) =
                                                dioxus::prelude::try_consume_context::<fs_components::AppContext>()
                                            {
                                                ctx.locale.set(code);
                                            }
                                            // Refresh inv so locale formats stay in sync.
                                            inv.set(LocaleSettings::load_inventory());
                                            saved_msg.set(Some(true));
                                        }
                                    },
                                    {fs_i18n::t("actions.apply")}
                                }
                                if saved_msg.read().is_some() {
                                    span {
                                        style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                        {fs_i18n::t("settings.language.applied")}
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Locale Formats ────────────────────────────────────────
                div {
                    SectionHeading { label: fs_i18n::t("settings.language.formats.title") }

                    div {
                        style: "font-size: 11px; color: var(--fs-color-text-muted); \
                                margin-bottom: 14px; padding: 8px 12px; \
                                background: var(--fs-color-bg-surface); \
                                border-radius: var(--fs-radius-sm); \
                                border-left: 3px solid var(--fs-color-primary);",
                        {fs_i18n::t("settings.language.formats.store_note")}
                    }

                    div { style: "display: flex; flex-direction: column; gap: 16px;",

                        // Fallback language
                        FormatRow {
                            label: fs_i18n::t("settings.language.formats.fallback"),
                            hint:  fs_i18n::t("settings.language.formats.fallback_hint"),
                            content: rsx! {
                                select {
                                    style: SELECT_STYLE,
                                    value: inv.read().fallback_language.clone()
                                        .unwrap_or_else(|| effective.fallback_language.clone()),
                                    onchange: {
                                        let mut inv = inv;
                                        move |e: Event<FormData>| {
                                            inv.write().fallback_language = Some(e.value());
                                            let _ = inv.read().save_inventory();
                                        }
                                    },
                                    for entry in installed.clone() {
                                        option { value: "{entry.code}", "{entry.name}" }
                                    }
                                }
                            },
                        }

                        // Date format
                        FormatRow {
                            label: fs_i18n::t("settings.language.formats.date"),
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
                                                         border-radius: var(--fs-radius-sm); \
                                                         cursor: pointer; border: 1px solid {}; \
                                                         background: {}; color: {};",
                                                        if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-border-default)" },
                                                        if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-bg-surface)" },
                                                        if is_active { "white" } else { "var(--fs-color-text-primary)" },
                                                    ),
                                                    onclick: {
                                                        let mut inv = inv;
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

                        // Time format
                        FormatRow {
                            label: fs_i18n::t("settings.language.formats.time"),
                            hint:  String::new(),
                            content: rsx! {
                                div { style: "display: flex; gap: 8px;",
                                    for fmt in TimeFormat::all() {
                                        {
                                            let is_active = inv.read().time_format.as_ref()
                                                .unwrap_or(&effective.time_format) == fmt;
                                            let f = fmt.clone();
                                            rsx! {
                                                button {
                                                    key: "{f.label()}",
                                                    style: format!(
                                                        "padding: 5px 14px; font-size: 12px; \
                                                         border-radius: var(--fs-radius-sm); \
                                                         cursor: pointer; border: 1px solid {}; \
                                                         background: {}; color: {};",
                                                        if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-border-default)" },
                                                        if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-bg-surface)" },
                                                        if is_active { "white" } else { "var(--fs-color-text-primary)" },
                                                    ),
                                                    onclick: {
                                                        let mut inv = inv;
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

                        // Number format
                        FormatRow {
                            label: fs_i18n::t("settings.language.formats.number"),
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
                                                         border-radius: var(--fs-radius-sm); \
                                                         cursor: pointer; border: 1px solid {}; \
                                                         background: {}; color: {};",
                                                        if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-border-default)" },
                                                        if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-bg-surface)" },
                                                        if is_active { "white" } else { "var(--fs-color-text-primary)" },
                                                    ),
                                                    onclick: {
                                                        let mut inv = inv;
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

                        // Auto-update packs
                        FormatRow {
                            label: fs_i18n::t("settings.language.formats.auto_update"),
                            hint:  fs_i18n::t("settings.language.formats.auto_update_hint"),
                            content: rsx! {
                                label {
                                    style: "display: flex; align-items: center; \
                                            gap: 8px; cursor: pointer;",
                                    input {
                                        r#type: "checkbox",
                                        checked: inv.read().auto_update_packs
                                            .unwrap_or(effective.auto_update_packs),
                                        onchange: {
                                            let mut inv = inv;
                                            move |e: Event<FormData>| {
                                                inv.write().auto_update_packs = Some(e.value() == "true");
                                                let _ = inv.read().save_inventory();
                                            }
                                        },
                                    }
                                    span {
                                        style: "font-size: 13px;",
                                        {fs_i18n::t("settings.language.formats.auto_update_label")}
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

// ── LanguageDetailPane ───────────────────────────────────────────────────────

/// Detail pane for a specific installed language: Info tab + Edit tab.
#[component]
fn LanguageDetailPane(entry: LangEntry, on_edit: EventHandler<(String, String)>) -> Element {
    let mut active_tab = use_signal(|| LangDetailTab::Info);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; overflow: hidden;",

            // Tab bar — Info is always first
            div {
                style: "display: flex; border-bottom: 1px solid var(--fs-color-border-default); \
                        padding: 0 24px; flex-shrink: 0; \
                        background: var(--fs-color-bg-surface);",
                TabBtn {
                    label:     "Info".to_string(),
                    is_active: *active_tab.read() == LangDetailTab::Info,
                    onclick:   move |_| active_tab.set(LangDetailTab::Info),
                }
                TabBtn {
                    label:     fs_i18n::t("settings.language.tabs.edit"),
                    is_active: *active_tab.read() == LangDetailTab::Edit,
                    onclick:   move |_| active_tab.set(LangDetailTab::Edit),
                }
            }

            div { style: "flex: 1; overflow-y: auto;",
                if *active_tab.read() == LangDetailTab::Info {
                    LangInfoPane { entry: entry.clone() }
                }
                if *active_tab.read() == LangDetailTab::Edit {
                    LangEditPane { entry: entry.clone(), on_edit }
                }
            }
        }
    }
}

// ── LangInfoPane ─────────────────────────────────────────────────────────────

/// Info tab — language pack metadata and explanations.
#[component]
fn LangInfoPane(entry: LangEntry) -> Element {
    let lang = Language::from_code(&entry.code);
    let flag = lang.flag_svg();

    rsx! {
        div {
            style: "padding: 24px; max-width: 520px; \
                    display: flex; flex-direction: column; gap: 20px;",

            // Header: flag + name + locale
            div {
                style: "display: flex; align-items: center; gap: 16px;",
                div {
                    style: "width: 40px; height: 28px; flex-shrink: 0; \
                            border-radius: 3px; overflow: hidden; \
                            background: var(--fs-color-bg-surface); \
                            display: flex; align-items: center; justify-content: center;",
                    if flag.is_empty() {
                        span { style: "font-size: 22px;", "🌐" }
                    } else {
                        div {
                            style: "width: 100%; height: 100%;",
                            dangerous_inner_html: "{flag}",
                        }
                    }
                }
                div {
                    h2 {
                        style: "margin: 0; font-size: 17px; font-weight: 600; \
                                color: var(--fs-color-text-primary);",
                        "{lang.display_name}"
                    }
                    div {
                        style: "font-size: 12px; color: var(--fs-color-text-muted); margin-top: 2px;",
                        "{lang.locale}"
                    }
                }
                if entry.builtin {
                    span {
                        style: "margin-left: auto; padding: 2px 8px; font-size: 11px; \
                                background: var(--fs-color-primary); color: white; \
                                border-radius: 999px; white-space: nowrap;",
                        "Built-in"
                    }
                }
            }

            // Metadata table
            div {
                style: "border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md); overflow: hidden;",

                LangInfoRow { label: "Language code".to_string(), value: lang.id.clone() }
                LangInfoRow { label: "Locale (BCP-47)".to_string(), value: lang.locale.clone() }
                LangInfoRow {
                    label: "Direction".to_string(),
                    value: lang.direction_label().to_string(),
                }
                LangInfoRow {
                    label: "Type".to_string(),
                    value: if entry.builtin {
                        "Built-in (always available)".to_string()
                    } else {
                        "Installed pack".to_string()
                    },
                }
            }
        }
    }
}

#[component]
fn LangInfoRow(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; padding: 8px 14px; \
                    border-bottom: 1px solid var(--fs-color-border-default); \
                    font-size: 13px;",
            span {
                style: "width: 140px; flex-shrink: 0; font-weight: 500; \
                        color: var(--fs-color-text-secondary, var(--fs-color-text-muted));",
                "{label}"
            }
            span { style: "color: var(--fs-color-text-primary);", "{value}" }
        }
    }
}

// ── LangEditPane ──────────────────────────────────────────────────────────────

/// Edit tab — SSH/GitHub contributor status + translation editor launcher.
#[component]
fn LangEditPane(entry: LangEntry, on_edit: EventHandler<(String, String)>) -> Element {
    let contrib =
        use_signal(|| GitContributorCheck::cached().unwrap_or(ContributorStatus::Unknown));
    {
        let mut contrib = contrib;
        use_future(move || async move {
            if *contrib.read() == ContributorStatus::Unknown {
                contrib.set(GitContributorCheck::check_and_cache());
            }
        });
    }

    rsx! {
        div {
            style: "padding: 24px; max-width: 560px; \
                    display: flex; flex-direction: column; gap: 20px;",

            SectionHeading { label: fs_i18n::t("settings.language.contrib.title") }

            p {
                style: "font-size: 13px; color: var(--fs-color-text-muted); margin: 0;",
                {fs_i18n::t("settings.language.contrib.description")}
            }

            // Contributor status card
            div {
                style: "padding: 12px 16px; background: var(--fs-color-bg-surface); \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md); \
                        display: flex; align-items: center; gap: 12px;",
                {
                    let (icon, text, sub, color) = match contrib.read().clone() {
                        ContributorStatus::Authenticated { ref github_user } => (
                            "✓",
                            format!("GitHub: @{github_user}"),
                            fs_i18n::t("settings.language.contrib.ssh_ok").into(),
                            "#16a34a",
                        ),
                        ContributorStatus::NotAuthenticated => (
                            "✕",
                            fs_i18n::t("settings.language.contrib.ssh_none").into(),
                            fs_i18n::t("settings.language.contrib.ssh_none_hint").into(),
                            "var(--fs-color-text-muted)",
                        ),
                        ContributorStatus::Unknown => (
                            "…",
                            fs_i18n::t("settings.language.contrib.ssh_checking").into(),
                            String::new(),
                            "var(--fs-color-text-muted)",
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
                                    style: "font-size: 11px; color: var(--fs-color-text-muted); \
                                            margin-top: 2px;",
                                    "{sub}"
                                }
                            }
                        }
                        button {
                            style: "padding: 5px 12px; font-size: 11px; background: none; \
                                    border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-sm); cursor: pointer; \
                                    color: var(--fs-color-text-muted); white-space: nowrap;",
                            onclick: {
                                let mut contrib = contrib;
                                move |_| {
                                    GitContributorCheck::clear_cache();
                                    contrib.set(ContributorStatus::Unknown);
                                }
                            },
                            {fs_i18n::t("settings.language.contrib.btn_recheck")}
                        }
                    }
                }
            }

            // Open translation editor for this language
            if entry.code != "en" {
                div {
                    p {
                        style: "font-size: 12px; color: var(--fs-color-text-muted); margin: 0 0 10px 0;",
                        {fs_i18n::t("settings.language.contrib.pick_language")}
                    }
                    div { style: "display: flex; flex-wrap: wrap; gap: 8px;",
                        {
                            let code = entry.code.clone();
                            let name = entry.name.clone();
                            rsx! {
                                button {
                                    style: "padding: 7px 16px; font-size: 13px; \
                                            background: var(--fs-color-bg-surface); \
                                            border: 1px solid var(--fs-color-border-default); \
                                            border-radius: var(--fs-radius-md); cursor: pointer; \
                                            color: var(--fs-color-text-primary);",
                                    onclick: {
                                        let code2 = code.clone();
                                        let name2 = name.clone();
                                        move |_| on_edit.call((code2.clone(), name2.clone()))
                                    },
                                    "{entry.name}"
                                    span {
                                        style: "margin-left: 6px; font-size: 11px; \
                                                color: var(--fs-color-text-muted);",
                                        "({entry.code})"
                                    }
                                }
                            }
                        }

                        if let ContributorStatus::Authenticated { .. } = contrib.read().clone() {
                            button {
                                style: "padding: 7px 16px; font-size: 13px; \
                                        background: transparent; \
                                        border: 1px dashed var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); cursor: pointer; \
                                        color: var(--fs-color-primary);",
                                onclick: move |_| {
                                    on_edit.call(("new".to_string(), "New Language".to_string()))
                                },
                                {fs_i18n::t("settings.language.contrib.btn_new_lang")}
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── InstallPane ───────────────────────────────────────────────────────────────

/// Panel for installing language packs from the Store.
#[component]
fn InstallPane(installed_ids: Vec<String>, on_installed: EventHandler<LangEntry>) -> Element {
    rsx! {
        div {
            style: "flex: 1; overflow-y: auto;",
            div { style: "padding: 24px; max-width: 560px;",
                SectionHeading { label: fs_i18n::t("settings.language.tabs.install") }
                p {
                    style: "font-size: 13px; color: var(--fs-color-text-muted); \
                            margin: 0 0 16px 0;",
                    {fs_i18n::t("settings.language.install_hint")}
                }
                AvailableLanguages { installed_ids, on_installed }
            }
        }
    }
}

const SELECT_STYLE: &str = "padding: 5px 10px; font-size: 13px; \
     background: var(--fs-color-bg-surface); \
     border: 1px solid var(--fs-color-border-default); \
     border-radius: var(--fs-radius-sm); \
     color: var(--fs-color-text-primary);";

#[component]
fn SectionHeading(label: String) -> Element {
    rsx! {
        h3 {
            style: "margin: 0 0 14px 0; font-size: 14px; font-weight: 600; \
                    color: var(--fs-color-text-primary); padding-bottom: 8px; \
                    border-bottom: 1px solid var(--fs-color-border-default);",
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
                            color: var(--fs-color-text-primary);",
                    "{label}"
                }
                if !hint.is_empty() {
                    div {
                        style: "font-size: 11px; color: var(--fs-color-text-muted); margin-top: 2px;",
                        "{hint}"
                    }
                }
            }
            div { style: "flex: 1;", {content} }
        }
    }
}

#[component]
fn TabBtn(label: String, is_active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            style: format!(
                "padding: 10px 18px; font-size: 13px; background: none; border: none; \
                 cursor: pointer; font-weight: {}; color: {}; \
                 border-bottom: 2px solid {}; margin-bottom: -1px; white-space: nowrap;",
                if is_active { "600" } else { "400" },
                if is_active { "var(--fs-color-primary)" } else { "var(--fs-color-text-secondary, var(--fs-color-text-muted))" },
                if is_active { "var(--fs-color-primary)" } else { "transparent" },
            ),
            onclick,
            "{label}"
        }
    }
}

/// Language radio row used in the Default pane's language picker.
#[component]
fn LangRow(
    code: String,
    name: String,
    selected: bool,
    on_select: EventHandler<MouseEvent>,
) -> Element {
    let bg = if selected {
        "background: var(--fs-color-primary); color: white;"
    } else {
        "background: transparent; color: var(--fs-color-text-primary);"
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
        }
    }
}

// ── AvailableLanguages ────────────────────────────────────────────────────────

#[component]
fn AvailableLanguages(
    installed_ids: Vec<String>,
    on_installed: EventHandler<LangEntry>,
) -> Element {
    let all_locales: Signal<Vec<LocaleInfo>> = use_signal(Vec::new);
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let busy: Signal<Option<String>> = use_signal(|| None);

    {
        use_future(move || {
            let mut all_locales = all_locales;
            async move {
                match StoreReader::official()
                    .fetch_toml::<LocaleCatalog>("Node/locale-catalog.toml")
                    .await
                {
                    Ok(catalog) => {
                        all_locales
                            .set(catalog.locales.into_iter().map(LocaleInfo::from).collect());
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
            style: "border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-md);",

            div {
                style: "padding: 8px 14px; \
                        border-bottom: 1px solid var(--fs-color-border-default); \
                        font-size: 12px; font-weight: 500; \
                        color: var(--fs-color-text-muted);",
                {fs_i18n::t("settings.language.available_heading")}
            }

            if *loading.read() {
                div {
                    style: "padding: 16px; text-align: center; \
                            color: var(--fs-color-text-muted); font-size: 13px;",
                    {fs_i18n::t("labels.loading")}
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "padding: 12px 14px; color: var(--fs-color-error); font-size: 13px;",
                    "{err}"
                }
            } else if available.is_empty() {
                div {
                    style: "padding: 16px; text-align: center; \
                            color: var(--fs-color-text-muted); font-size: 13px;",
                    {fs_i18n::t("settings.language.all_installed")}
                }
            } else {
                div {
                    style: "max-height: 400px; overflow-y: auto; scrollbar-width: thin;",
                    for locale in available {
                        AvailableLangRow {
                            key:        "{locale.code}",
                            locale:     locale.clone(),
                            installing: busy.read().as_deref() == Some(locale.code.as_str()),
                            on_install: {
                                let mut busy = busy;
                                move |l: LocaleInfo| {
                                    let id   = l.code.clone();
                                    let name = l.name.clone();
                                    busy.set(Some(id.clone()));
                                    let mut busy = busy;
                                    let entry = LangEntry { code: id.clone(), name, builtin: false };
                                    let cb    = on_installed;
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

// ── AvailableLangRow ──────────────────────────────────────────────────────────

#[component]
fn AvailableLangRow(
    locale: LocaleInfo,
    installing: bool,
    on_install: EventHandler<LocaleInfo>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; padding: 8px 14px; \
                    border-bottom: 1px solid var(--fs-color-border-default); \
                    color: var(--fs-color-text-primary);",
            span { style: "font-size: 14px; flex: 1;", "{locale.name}" }
            span { style: "font-size: 12px; color: var(--fs-color-text-muted);", "{locale.code}" }
            if locale.completeness < 100 {
                span {
                    style: "font-size: 11px; color: var(--fs-color-text-muted); \
                            background: var(--fs-color-bg-overlay); padding: 2px 6px; \
                            border-radius: 999px;",
                    "{locale.completeness}%"
                }
            }
            button {
                style: "padding: 4px 12px; font-size: 12px; \
                        background: var(--fs-color-primary); color: white; \
                        border: none; border-radius: var(--fs-radius-sm); \
                        cursor: pointer; min-width: 60px;",
                disabled: installing,
                onclick: {
                    let locale = locale.clone();
                    move |_| on_install.call(locale.clone())
                },
                { if installing { fs_i18n::t("labels.loading") } else { fs_i18n::t("actions.install") } }
            }
        }
    }
}
