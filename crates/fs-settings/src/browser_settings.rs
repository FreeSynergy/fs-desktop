// fs-settings/browser_settings.rs — Search engine selection for the browser.
//
// Follows the same pattern as DesktopSettings:
//   - BrowserConfig loaded from ~/.config/fsn/browser.toml
//   - Radio-button style list of SearchEngine choices
//   - Save button persists to disk

use dioxus::prelude::*;
use fs_browser::{BrowserConfig, SearchEngine, SearchEngineRegistry};
use fs_i18n;

// ── BrowserSettings ───────────────────────────────────────────────────────────

/// Browser settings component — search engine selection.
///
/// The user picks from a list of privacy-respecting search engines.
/// The selection is saved to `~/.config/fsn/browser.toml`.
#[component]
pub fn BrowserSettings() -> Element {
    let config = use_signal(BrowserConfig::load);

    rsx! {
        div {
            style: "padding: 24px; max-width: 500px;",

            h3 { style: "margin-top: 0; color: var(--fs-color-text-primary);",
                {fs_i18n::t("settings.browser.title")}
            }

            // Search engine section
            div { style: "margin-bottom: 32px;",
                label {
                    style: "display: block; font-weight: 500; margin-bottom: 4px; \
                            color: var(--fs-color-text-primary);",
                    {fs_i18n::t("settings.browser.search_engine")}
                }
                p {
                    style: "font-size: 13px; color: var(--fs-color-text-muted); margin: 0 0 12px;",
                    {fs_i18n::t("settings.browser.search_engine_hint")}
                }

                div { style: "display: flex; flex-direction: column; gap: 6px;",
                    for engine in SearchEngineRegistry::all() {
                        SearchEngineBtn { engine: engine.clone(), config }
                    }
                }
            }

            button {
                style: "padding: 8px 24px; \
                        background: var(--fs-color-primary, #06b6d4); color: white; \
                        border: none; border-radius: var(--fs-radius-md, 6px); cursor: pointer; \
                        font-size: 14px; font-weight: 500;",
                onclick: move |_| config.read().save(),
                {fs_i18n::t("actions.save")}
            }
        }
    }
}

// ── SearchEngineBtn ───────────────────────────────────────────────────────────

#[component]
fn SearchEngineBtn(engine: SearchEngine, config: Signal<BrowserConfig>) -> Element {
    let id = engine.id.clone();
    let is_active = config.read().search_engine == engine.id;
    let border = if is_active {
        "var(--fs-color-primary, #06b6d4)"
    } else {
        "var(--fs-color-border-default, #334155)"
    };
    let weight = if is_active { "600" } else { "400" };

    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; \
                    padding: 10px 14px; border-radius: var(--fs-radius-md, 6px); \
                    border: 2px solid {border}; cursor: pointer; \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    text-align: left; font-weight: {weight}; width: 100%;",
            onclick: move |_| config.write().search_engine.clone_from(&id),
            span { style: "font-size: 16px;", "🔍" }
            span {
                style: "font-size: 14px; color: var(--fs-color-text-primary);",
                "{engine.name}"
            }
        }
    }
}
