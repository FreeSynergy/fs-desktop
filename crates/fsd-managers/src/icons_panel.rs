/// Icons Manager panel — shows available icon sets and their details.
use dioxus::prelude::*;
use fsn_i18n;
use fsn_manager_icons::IconManager;

#[component]
pub fn IconsManagerPanel() -> Element {
    let icons_root = std::path::PathBuf::from(
        std::env::var("FSN_ICONS_ROOT")
            .unwrap_or_else(|_| "../FreeSynergy.Icons".into())
    );
    let mgr  = IconManager::new(icons_root, vec![]);
    let sets = mgr.sets();

    rsx! {
        div {
            style: "padding: 24px; max-width: 480px;",

            h3 { style: "margin-top: 0; color: var(--fsn-text-primary);",
                {fsn_i18n::t("managers.icons.title")}
            }
            p { style: "font-size: 13px; color: var(--fsn-color-text-muted); margin-top: -8px;",
                {fsn_i18n::t("managers.icons.description")}
            }

            if sets.is_empty() {
                div {
                    style: "padding: 20px; text-align: center; \
                            color: var(--fsn-color-text-muted); font-size: 13px; \
                            border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md);",
                    {fsn_i18n::t("managers.icons.no_sets")}
                }
            } else {
                div {
                    style: "border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); overflow: hidden;",
                    for set in &sets {
                        div {
                            style: "display: flex; align-items: center; gap: 14px; \
                                    padding: 14px 16px; \
                                    border-bottom: 1px solid var(--fsn-color-border-default); \
                                    color: var(--fsn-color-text-primary);",
                            span { style: "font-size: 24px;", "🖼" }
                            div { style: "flex: 1;",
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "font-size: 14px; font-weight: 500;", "{set.name}" }
                                    if set.builtin {
                                        span {
                                            style: "padding: 1px 6px; \
                                                    background: var(--fsn-color-primary, #06b6d4); \
                                                    color: #fff; \
                                                    border-radius: 999px; font-size: 10px;",
                                            "Built-in"
                                        }
                                    }
                                }
                                div { style: "font-size: 11px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                                    if !set.description.is_empty() {
                                        "{set.description}"
                                    } else {
                                        "ID: {set.id}"
                                    }
                                    if set.has_dark_variants {
                                        span {
                                            style: "margin-left: 8px; padding: 1px 6px; \
                                                    background: var(--fsn-color-bg-overlay); \
                                                    border-radius: 999px; font-size: 10px;",
                                            "Dark variants"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
