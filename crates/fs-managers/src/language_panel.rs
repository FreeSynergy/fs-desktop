/// Language Manager panel — shows active language, lists available, allows switching.
use dioxus::prelude::*;
use fs_i18n;
use fs_manager_language::LanguageManager;

#[component]
pub fn LanguageManagerPanel() -> Element {
    let mgr = LanguageManager::new();
    let available = mgr.available();
    let active    = mgr.active();

    let mut selected = use_signal(|| active.id.clone());
    let mut saved    = use_signal(|| false);

    rsx! {
        div {
            style: "padding: 24px; max-width: 480px;",

            h3 { style: "margin-top: 0; color: var(--fs-text-primary);",
                {fs_i18n::t("managers.language.title")}
            }
            p { style: "font-size: 13px; color: var(--fs-color-text-muted); margin-top: -8px;",
                {fs_i18n::t("managers.language.description")}
            }

            div {
                style: "border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md); overflow: hidden; margin-bottom: 20px;",

                for lang in &available {
                    {
                        let is_active = lang.id == *selected.read();
                        let lang_id   = lang.id.clone();
                        let flag_svg  = lang.flag_svg().to_string();
                        let bg = if is_active {
                            "background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15)); \
                             color: var(--fs-sidebar-active, #4d8bf5);"
                        } else {
                            "background: transparent; color: var(--fs-color-text-primary);"
                        };
                        rsx! {
                            div {
                                style: "display: flex; align-items: center; gap: 12px; \
                                        padding: 11px 16px; cursor: pointer; \
                                        border-bottom: 1px solid var(--fs-color-border-default); \
                                        transition: background 100ms; {bg}",
                                onclick: move |_| {
                                    selected.set(lang_id.clone());
                                    saved.set(false);
                                },
                                // Selection indicator
                                span { style: "font-size: 16px; flex-shrink: 0;",
                                    if is_active { "◉" } else { "○" }
                                }
                                // Flag icon
                                if !flag_svg.is_empty() {
                                    span {
                                        style: "flex-shrink: 0; width: 24px; height: 14px; \
                                                display: inline-flex; align-items: center; \
                                                border-radius: 2px; overflow: hidden;",
                                        dangerous_inner_html: "{flag_svg}",
                                    }
                                }
                                // Language name + locale
                                span { style: "font-size: 14px; font-weight: 500; flex: 1;",
                                    "{lang.display_name}"
                                }
                                span { style: "font-size: 12px; opacity: 0.55;",
                                    "{lang.locale}"
                                }
                            }
                        }
                    }
                }
            }

            div { style: "display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fs-color-primary, #06b6d4); \
                            color: white; border: none; border-radius: var(--fs-radius-md, 6px); \
                            cursor: pointer; font-size: 13px;",
                    onclick: move |_| {
                        let id = selected.read().clone();
                        let mgr = LanguageManager::new();
                        let _ = mgr.set_active(&id);
                        saved.set(true);
                    },
                    {fs_i18n::t("actions.apply")}
                }
                if *saved.read() {
                    span { style: "font-size: 12px; color: var(--fs-color-text-muted);",
                        {fs_i18n::t("managers.saved")}
                    }
                }
            }
        }
    }
}
