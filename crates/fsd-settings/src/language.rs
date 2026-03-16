/// Language settings — choose UI language, load language packs from store.
use dioxus::prelude::*;

/// Built-in (always installed) languages.
pub const BUILTIN_LANGUAGES: &[(&str, &str)] = &[
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
];

/// A language entry (code + native name).
#[derive(Clone, PartialEq)]
struct LangEntry {
    code: &'static str,
    name: &'static str,
}

/// Language settings component.
///
/// Shows only installed languages. Built-in languages are always considered
/// installed. Additional packs installed from the Store appear in the list too.
/// When 8+ entries are shown a scrollbar appears. The "Install more…" hint
/// opens the Store filtered to Language packages.
#[component]
pub fn LanguageSettings() -> Element {
    // Currently "installed" languages (built-ins are always present).
    // In a real system this would be loaded from fsn-config / installed packs.
    let installed: Signal<Vec<LangEntry>> = use_signal(|| {
        BUILTIN_LANGUAGES
            .iter()
            .map(|(code, name)| LangEntry { code, name })
            .collect()
    });

    let mut selected = use_signal(|| "de".to_string());
    let mut install_hint = use_signal(|| false);

    let count = installed.read().len();
    // Show scrollbar once there are 8 or more installed languages.
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

            // Installed language list
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
                            code: entry.code,
                            name: entry.name,
                            selected: *selected.read() == entry.code,
                            on_select: {
                                let code = entry.code.to_string();
                                move |_| *selected.write() = code.clone()
                            },
                        }
                    }
                }
            }

            // "Install more" button
            div { style: "margin-bottom: 24px;",
                button {
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 16px; \
                            background: var(--fsn-color-bg-surface); \
                            border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); font-size: 13px; \
                            cursor: pointer; color: var(--fsn-color-primary); width: 100%;",
                    onclick: move |_| {
                        let cur = *install_hint.read();
                        install_hint.set(!cur);
                    },
                    span { "🌐" }
                    span { "Install more languages…" }
                }
                if *install_hint.read() {
                    div {
                        style: "margin-top: 8px; padding: 10px 14px; \
                                background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-md); font-size: 13px;",
                        "Open "
                        strong { "Store" }
                        " → filter by "
                        strong { "Language" }
                        " to find and install additional language packs."
                    }
                }
            }

            // Apply button
            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); \
                        color: white; border: none; border-radius: var(--fsn-radius-md); \
                        cursor: pointer;",
                "Apply"
            }
        }
    }
}

// ── LangRow ───────────────────────────────────────────────────────────────────

#[component]
fn LangRow(
    code: &'static str,
    name: &'static str,
    selected: bool,
    on_select: EventHandler<MouseEvent>,
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
            // Radio indicator
            span {
                style: "font-size: 16px;",
                if selected { "◉" } else { "○" }
            }
            span { style: "font-size: 14px;", "{name}" }
            span {
                style: "margin-left: auto; font-size: 12px; opacity: 0.6;",
                "{code}"
            }
        }
    }
}
