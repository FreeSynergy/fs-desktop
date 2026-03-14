/// Log viewer — shows live or recent logs for a container.
use dioxus::prelude::*;

/// Log entry with timestamp and message.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

/// Log severity level.
#[derive(Clone, Debug, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    pub fn color(&self) -> &str {
        match self {
            Self::Info  => "var(--fsn-color-text-primary)",
            Self::Warn  => "var(--fsn-color-warning)",
            Self::Error => "var(--fsn-color-error)",
            Self::Debug => "var(--fsn-color-text-muted)",
        }
    }
}

/// Live log viewer component.
#[component]
pub fn LogViewer(service: String) -> Element {
    let entries = use_signal(Vec::<LogEntry>::new);
    let follow = use_signal(|| true);

    // TODO: stream logs from fsn-container via use_resource + tokio channel

    rsx! {
        div {
            class: "fsd-log-viewer",
            style: "display: flex; flex-direction: column; height: 100%;",

            // Toolbar
            div {
                style: "display: flex; align-items: center; gap: 12px; padding: 8px 0; border-bottom: 1px solid var(--fsn-color-border-default);",

                h3 { style: "margin: 0; flex: 1;", "Logs: {service}" }

                label { style: "display: flex; align-items: center; gap: 4px; font-size: 13px;",
                    input {
                        r#type: "checkbox",
                        checked: "{follow.read()}",
                    }
                    "Follow"
                }

                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: 4px; cursor: pointer; font-size: 12px;",
                    "Clear"
                }
            }

            // Log output
            div {
                style: "flex: 1; overflow-y: auto; padding: 8px 0; font-family: var(--fsn-font-mono); font-size: 12px;",

                if entries.read().is_empty() {
                    if service.is_empty() {
                        div { style: "color: var(--fsn-color-text-muted); padding: 16px;",
                            "Select a service to view its logs."
                        }
                    } else {
                        div { style: "color: var(--fsn-color-text-muted); padding: 16px;",
                            "No logs yet for {service}."
                        }
                    }
                }

                for entry in entries.read().iter() {
                    div {
                        style: "display: flex; gap: 12px; padding: 2px 0; color: {entry.level.color()};",
                        span { style: "color: var(--fsn-color-text-muted); white-space: nowrap;", "{entry.timestamp}" }
                        span { "{entry.message}" }
                    }
                }
            }
        }
    }
}
