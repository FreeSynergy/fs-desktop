/// Language settings — choose UI language, load language packs from store.
use dioxus::prelude::*;

/// Available built-in languages.
pub const BUILTIN_LANGUAGES: &[(&str, &str)] = &[
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
];

/// Language settings component.
#[component]
pub fn LanguageSettings() -> Element {
    let selected = use_signal(|| "de".to_string());

    rsx! {
        div {
            class: "fsd-language",
            style: "padding: 24px; max-width: 500px;",

            h3 { style: "margin-top: 0;", "Language" }

            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Interface Language" }
                select {
                    style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 14px;",
                    onchange: move |e| *selected.write() = e.value(),
                    for (code, name) in BUILTIN_LANGUAGES {
                        option {
                            value: "{code}",
                            selected: *selected.read() == *code,
                            "{name}"
                        }
                    }
                }
            }

            div { style: "padding: 12px 16px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); margin-bottom: 24px;",
                p { style: "margin: 0; font-size: 13px;",
                    "More language packs are available in the "
                    strong { "Store" }
                    ". Install them and they will appear here."
                }
            }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                "Apply"
            }
        }
    }
}
