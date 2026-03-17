/// Language settings — select UI language, install/remove language packs.
use dioxus::prelude::*;
use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
use fsn_store::StoreClient;
use serde::Deserialize;

/// Built-in (always available, cannot be removed) languages.
pub const BUILTIN_LANGUAGES: &[(&str, &str)] = &[
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
];

/// A language entry for display/selection.
#[derive(Clone, PartialEq)]
struct LangEntry {
    code:    String,
    name:    String,
    builtin: bool,
}

// ── Minimal catalog types for parsing shared/catalog.toml ─────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct LangPack {
    id:      String,
    name:    String,
    version: String,
    #[serde(default)]
    kind:    String,
    #[serde(default)]
    path:    Option<String>,
}

#[derive(Deserialize)]
struct SharedCatalog {
    #[serde(default)]
    packages: Vec<LangPack>,
}

// ── Helpers ────────────────────────────────────────────────────────────────────

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

async fn install_language_pack(pack: LangPack) -> Result<(), String> {
    let home    = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let fsn_dir = std::path::PathBuf::from(&home).join(".local/share/fsn");

    let base = pack.path.clone()
        .unwrap_or_else(|| format!("shared/i18n/{}", pack.id));
    let url = format!("{base}/ui.toml");

    let file_path = match StoreClient::node_store().fetch_raw(&url).await {
        Ok(content) => {
            let dest_dir = fsn_dir.join("i18n").join(&pack.id);
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
        id:        pack.id.clone(),
        name:      pack.name.clone(),
        kind:      "language".into(),
        version:   pack.version.clone(),
        file_path,
    })
    .map_err(|e| format!("Registry error: {e}"))
}

// ── LanguageSettings ───────────────────────────────────────────────────────────

#[component]
pub fn LanguageSettings() -> Element {
    let installed          = use_signal(load_installed);
    let mut selected       = use_signal(|| "de".to_string());
    let mut show_available = use_signal(|| false);

    let count = installed.read().len();
    let list_style = if count >= 8 {
        "max-height: 240px; overflow-y: auto; border: 1px solid var(--fsn-color-border-default); \
         border-radius: var(--fsn-radius-md); scrollbar-width: thin;"
    } else {
        "border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);"
    };

    rsx! {
        div {
            class: "fsd-language",
            style: "padding: 24px; max-width: 500px;",

            h3 { style: "margin-top: 0;", "Language" }

            // ── Installed language list ────────────────────────────────────────
            div { style: "margin-bottom: 16px;",
                label {
                    style: "display: block; font-weight: 500; margin-bottom: 8px;",
                    "Interface Language"
                    span {
                        style: "margin-left: 8px; font-size: 12px; font-weight: 400; \
                                color: var(--fsn-color-text-muted);",
                        "({count} installed)"
                    }
                }
                div { style: "{list_style}",
                    for entry in installed.read().clone() {
                        LangRow {
                            key: "{entry.code}",
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

            // ── "Install more" toggle ──────────────────────────────────────────
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
                    if *show_available.read() { "Hide available languages" }
                    else { "Install more languages…" }
                }
            }

            // ── Available languages panel ──────────────────────────────────────
            if *show_available.read() {
                AvailableLanguages {
                    installed_ids: installed.read().iter().map(|e| e.code.clone()).collect(),
                    on_installed: {
                        let mut installed = installed.clone();
                        move |entry: LangEntry| installed.write().push(entry)
                    },
                }
            }

            // ── Apply button ───────────────────────────────────────────────────
            div { style: "margin-top: 24px;",
                button {
                    style: "padding: 8px 24px; background: var(--fsn-color-primary); \
                            color: white; border: none; border-radius: var(--fsn-radius-md); \
                            cursor: pointer;",
                    "Apply"
                }
            }
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
    let available: Signal<Vec<LangPack>>    = use_signal(Vec::new);
    let mut loading: Signal<bool>           = use_signal(|| true);
    let mut error:   Signal<Option<String>> = use_signal(|| None);
    let busy:        Signal<Option<String>> = use_signal(|| None); // id of pack being installed

    {
        let available = available.clone();
        let installed_ids = installed_ids.clone();
        use_future(move || {
            let available = available.clone();
            let installed_ids = installed_ids.clone();
            async move {
                match StoreClient::node_store().fetch_raw("shared/catalog.toml").await {
                    Ok(content) => {
                        match toml::from_str::<SharedCatalog>(&content) {
                            Ok(catalog) => {
                                let langs: Vec<LangPack> = catalog
                                    .packages
                                    .into_iter()
                                    .filter(|p| p.kind == "language")
                                    .filter(|p| !installed_ids.contains(&p.id))
                                    .collect();
                                available.clone().set(langs);
                                error.set(None);
                            }
                            Err(e) => error.set(Some(format!("Parse error: {e}"))),
                        }
                    }
                    Err(e) => error.set(Some(format!("Could not load catalog: {e}"))),
                }
                loading.set(false);
            }
        });
    }

    rsx! {
        div {
            style: "border: 1px solid var(--fsn-color-border-default); \
                    border-radius: var(--fsn-radius-md); margin-bottom: 16px;",

            div {
                style: "padding: 8px 14px; border-bottom: 1px solid var(--fsn-color-border-default); \
                        font-size: 12px; font-weight: 500; color: var(--fsn-color-text-muted);",
                "Available to install"
            }

            if *loading.read() {
                div {
                    style: "padding: 16px; text-align: center; color: var(--fsn-color-text-muted); \
                            font-size: 13px;",
                    "Loading…"
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "padding: 12px 14px; color: var(--fsn-color-error); font-size: 13px;",
                    "{err}"
                }
            } else if available.read().is_empty() {
                div {
                    style: "padding: 16px; text-align: center; color: var(--fsn-color-text-muted); \
                            font-size: 13px;",
                    "All available languages are already installed."
                }
            } else {
                div {
                    style: "max-height: 280px; overflow-y: auto; scrollbar-width: thin;",
                    for pack in available.read().clone() {
                        AvailableLangRow {
                            key:       "{pack.id}",
                            pack:      pack.clone(),
                            installing: busy.read().as_deref() == Some(pack.id.as_str()),
                            on_install: {
                                let mut available = available.clone();
                                let mut busy      = busy.clone();
                                move |p: LangPack| {
                                    let id   = p.id.clone();
                                    let name = p.name.clone();
                                    busy.set(Some(id.clone()));
                                    let mut available = available.clone();
                                    let mut busy      = busy.clone();
                                    let entry = LangEntry { code: id.clone(), name: name.clone(), builtin: false };
                                    let cb    = on_installed.clone();
                                    spawn(async move {
                                        match install_language_pack(p).await {
                                            Ok(()) => {
                                                available.write().retain(|x| x.id != id);
                                                cb.call(entry);
                                            }
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
    pack:       LangPack,
    installing: bool,
    on_install: EventHandler<LangPack>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; padding: 8px 14px; \
                    border-bottom: 1px solid var(--fsn-color-border-default); \
                    color: var(--fsn-color-text-primary);",
            span { style: "font-size: 14px; flex: 1;", "{pack.name}" }
            span { style: "font-size: 12px; color: var(--fsn-color-text-muted);", "{pack.id}" }
            button {
                style: "padding: 4px 12px; font-size: 12px; \
                        background: var(--fsn-color-primary); color: white; \
                        border: none; border-radius: var(--fsn-radius-sm); \
                        cursor: pointer; min-width: 60px;",
                disabled: installing,
                onclick: {
                    let pack = pack.clone();
                    move |_| on_install.call(pack.clone())
                },
                if installing { "…" } else { "Install" }
            }
        }
    }
}
