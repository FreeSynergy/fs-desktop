/// Package card — displays a single package in the store browser.
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::node_package::PackageKind;

/// A package entry from the FSN store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub kind: PackageKind,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub icon: Option<String>,
    /// Store-relative path to the package directory (used for file downloads).
    pub store_path: Option<String>,
    pub installed: bool,
    pub update_available: bool,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub author: String,
}

/// Package card component.
#[component]
pub fn PackageCard(
    package:    PackageEntry,
    on_details: EventHandler<MouseEvent>,
    #[props(default)]
    on_install: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div {
            class: "fsd-package-card",
            style: "background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); \
                    border-radius: var(--fsn-radius-lg); padding: 16px; display: flex; flex-direction: column; \
                    gap: 8px; cursor: pointer; transition: border-color 0.15s;",
            onclick: on_details,

            // Header
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                div {
                    style: "width: 40px; height: 40px; border-radius: var(--fsn-radius-md); background: var(--fsn-color-bg-overlay); display: flex; align-items: center; justify-content: center; font-size: 20px;",
                    if let Some(icon) = &package.icon {
                        img { src: "{icon}", width: "32", height: "32" }
                    } else {
                        "{package.kind.icon()}"
                    }
                }
                div {
                    strong { style: "display: block; font-size: 15px;", "{package.name}" }
                    span { style: "font-size: 12px; color: var(--fsn-color-text-muted);", "v{package.version} · {package.category}" }
                }
                if package.installed {
                    span {
                        style: "margin-left: auto; font-size: 11px; color: var(--fsn-color-success, #22c55e); \
                                background: rgba(34,197,94,0.12); padding: 2px 8px; border-radius: 999px;",
                        {fsn_i18n::t("store.status.installed")}
                    }
                }
            }

            // Description
            p {
                style: "margin: 0; font-size: 13px; color: var(--fsn-color-text-secondary); line-height: 1.4;",
                "{package.description}"
            }

            // Tags
            if !package.tags.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 4px;",
                    for tag in package.tags.iter().take(5) {
                        span {
                            key: "{tag}",
                            style: "font-size: 10px; padding: 2px 6px; border-radius: 999px; \
                                    background: var(--fsn-color-bg-overlay); \
                                    color: var(--fsn-color-text-muted); \
                                    border: 1px solid var(--fsn-color-border-default);",
                            "{tag}"
                        }
                    }
                }
            }

            // Install / installed button
            div { style: "margin-top: auto;",
                if package.installed {
                    if package.update_available {
                        button {
                            style: "width: 100%; padding: 8px; background: var(--fsn-color-warning); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px;",
                            {fsn_i18n::t("store.status.update_available")}
                        }
                    } else {
                        button {
                            style: "width: 100%; padding: 8px; background: var(--fsn-color-bg-overlay); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: default; font-size: 13px;",
                            disabled: true,
                            {fsn_i18n::t("store.status.installed")}
                        }
                    }
                } else {
                    button {
                        style: "width: 100%; padding: 8px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px;",
                        onclick: move |evt| {
                            if let Some(ref handler) = on_install {
                                evt.stop_propagation();
                                handler.call(evt);
                            }
                        },
                        {fsn_i18n::t("actions.install")}
                    }
                }
            }
        }
    }
}
