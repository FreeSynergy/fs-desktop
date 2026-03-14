/// i18n editor — create and edit FSN language files (.ftl snippets).
use dioxus::prelude::*;

/// A single i18n entry (key → value).
#[derive(Clone, Debug, PartialEq)]
pub struct I18nEntry {
    pub key: String,
    pub value: String,
    pub category: String,
}

/// i18n editor component.
#[component]
pub fn I18nEditor() -> Element {
    let entries = use_signal(Vec::<I18nEntry>::new);
    let selected_lang = use_signal(|| "de".to_string());
    let selected_category = use_signal(|| "actions".to_string());

    rsx! {
        div {
            class: "fsd-i18n-editor",
            style: "display: flex; height: 100%;",

            // Sidebar — language + category picker
            div {
                style: "width: 200px; border-right: 1px solid var(--fsn-color-border-default); padding: 16px;",

                div { style: "margin-bottom: 16px;",
                    label { style: "display: block; font-size: 12px; font-weight: 600; margin-bottom: 8px; color: var(--fsn-color-text-muted);", "LANGUAGE" }
                    select {
                        style: "width: 100%; padding: 6px; border: 1px solid var(--fsn-color-border-default); border-radius: 4px;",
                        option { value: "de", "Deutsch" }
                        option { value: "en", "English" }
                        option { value: "fr", "Français" }
                    }
                }

                div {
                    label { style: "display: block; font-size: 12px; font-weight: 600; margin-bottom: 8px; color: var(--fsn-color-text-muted);", "CATEGORY" }
                    for cat in &["actions", "nouns", "status", "errors", "phrases", "time", "validation", "help"] {
                        button {
                            style: "display: block; width: 100%; text-align: left; padding: 6px 8px; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; background: {if *selected_category.read() == *cat { \"var(--fsn-color-bg-overlay)\" } else { \"transparent\" }};",
                            onclick: {
                                let cat = cat.to_string();
                                move |_| *selected_category.write() = cat.clone()
                            },
                            "{cat}"
                        }
                    }
                }
            }

            // Main editor area
            div {
                style: "flex: 1; padding: 16px; overflow: auto;",

                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                    h3 { style: "margin: 0;", "{selected_lang.read()} / {selected_category.read()}.ftl" }
                    button {
                        style: "padding: 6px 12px; background: var(--fsn-color-primary); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px;",
                        "+ Add Key"
                    }
                }

                if entries.read().is_empty() {
                    div {
                        style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                        "No entries yet. Click '+ Add Key' to start."
                    }
                } else {
                    div {
                        for entry in entries.read().iter() {
                            div {
                                style: "display: flex; gap: 8px; margin-bottom: 8px; align-items: center;",
                                input {
                                    r#type: "text",
                                    value: "{entry.key}",
                                    style: "width: 200px; padding: 6px 8px; border: 1px solid var(--fsn-color-border-default); border-radius: 4px; font-family: var(--fsn-font-mono); font-size: 13px;",
                                }
                                span { style: "color: var(--fsn-color-text-muted);", "=" }
                                input {
                                    r#type: "text",
                                    value: "{entry.value}",
                                    style: "flex: 1; padding: 6px 8px; border: 1px solid var(--fsn-color-border-default); border-radius: 4px; font-size: 13px;",
                                }
                            }
                        }
                    }
                }

                // Export button
                if !entries.read().is_empty() {
                    button {
                        style: "margin-top: 16px; padding: 8px 16px; background: var(--fsn-color-success); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                        "Export .ftl"
                    }
                }
            }
        }
    }
}
