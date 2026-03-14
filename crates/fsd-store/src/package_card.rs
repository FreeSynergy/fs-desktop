/// Package card — displays a single package in the store browser.
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// A package entry from the FSN store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub icon: Option<String>,
    pub installed: bool,
    pub update_available: bool,
}

/// Package card component.
#[component]
pub fn PackageCard(package: PackageEntry) -> Element {
    rsx! {
        div {
            class: "fsd-package-card",
            style: "background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-lg); padding: 16px; display: flex; flex-direction: column; gap: 8px;",

            // Header
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                div {
                    style: "width: 40px; height: 40px; border-radius: var(--fsn-radius-md); background: var(--fsn-color-bg-overlay); display: flex; align-items: center; justify-content: center; font-size: 20px;",
                    if let Some(icon) = &package.icon {
                        img { src: "{icon}", width: "32", height: "32" }
                    } else {
                        "📦"
                    }
                }
                div {
                    strong { style: "display: block; font-size: 15px;", "{package.name}" }
                    span { style: "font-size: 12px; color: var(--fsn-color-text-muted);", "v{package.version} · {package.category}" }
                }
            }

            // Description
            p {
                style: "margin: 0; font-size: 13px; color: var(--fsn-color-text-secondary); line-height: 1.4;",
                "{package.description}"
            }

            // Install button
            div { style: "margin-top: auto;",
                if package.installed {
                    if package.update_available {
                        button {
                            style: "width: 100%; padding: 8px; background: var(--fsn-color-warning); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px;",
                            "Update available"
                        }
                    } else {
                        button {
                            style: "width: 100%; padding: 8px; background: var(--fsn-color-bg-overlay); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: default; font-size: 13px;",
                            disabled: true,
                            "Installed ✓"
                        }
                    }
                } else {
                    button {
                        style: "width: 100%; padding: 8px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px;",
                        // TODO: open install wizard
                        "Install"
                    }
                }
            }
        }
    }
}
