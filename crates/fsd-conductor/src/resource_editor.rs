/// Resource editor — configure CPU, RAM, volumes, and environment for a service.
use dioxus::prelude::*;

/// Resource limits for a container.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceLimits {
    pub cpu_shares: u32,        // Relative CPU weight (default 1024)
    pub memory_mb: Option<u32>, // Memory limit in MiB (None = unlimited)
    pub memory_swap_mb: Option<u32>,
    pub pids_limit: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_shares: 1024,
            memory_mb: None,
            memory_swap_mb: None,
            pids_limit: None,
        }
    }
}

/// Resource editor component.
#[component]
pub fn ResourceEditor(service_name: String) -> Element {
    let limits = use_signal(ResourceLimits::default);

    rsx! {
        div {
            class: "fsd-resource-editor",
            style: "padding: 16px;",

            h3 { "Resources: {service_name}" }

            // CPU
            div { style: "margin-bottom: 16px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "CPU Weight" }
                input {
                    r#type: "range",
                    min: "128",
                    max: "4096",
                    value: "{limits.read().cpu_shares}",
                    style: "width: 100%;",
                }
                span { style: "font-size: 12px; color: var(--fsn-color-text-muted);",
                    "Weight: {limits.read().cpu_shares} (default: 1024)"
                }
            }

            // Memory
            div { style: "margin-bottom: 16px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Memory Limit" }
                input {
                    r#type: "number",
                    placeholder: "Unlimited",
                    style: "width: 100%; padding: 8px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                }
                span { style: "font-size: 12px; color: var(--fsn-color-text-muted);", "MiB — leave empty for no limit" }
            }

            // Buttons
            div { style: "display: flex; gap: 8px; margin-top: 24px;",
                button {
                    style: "background: var(--fsn-color-primary); color: white; border: none; padding: 8px 16px; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    "Apply"
                }
                button {
                    style: "background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); padding: 8px 16px; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    "Cancel"
                }
            }
        }
    }
}
