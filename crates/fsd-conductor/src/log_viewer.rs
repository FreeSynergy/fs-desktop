/// Log viewer — shows live or recent logs for a container.
use dioxus::prelude::*;
use fsn_container::PodmanClient;

/// Log entry with severity level.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

/// Log severity level — detected from common log prefixes.
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

    fn detect(line: &str) -> Self {
        let l = line.to_ascii_lowercase();
        if l.contains("error") || l.contains("err]") || l.contains("fatal") || l.contains("panic") {
            Self::Error
        } else if l.contains("warn") || l.contains("wrn]") {
            Self::Warn
        } else if l.contains("debug") || l.contains("dbg]") || l.contains("trace") {
            Self::Debug
        } else {
            Self::Info
        }
    }
}

/// Live log viewer component — polls the last 200 lines every 3 seconds.
#[component]
pub fn LogViewer(service: String) -> Element {
    let mut entries = use_signal(Vec::<LogEntry>::new);
    let mut follow  = use_signal(|| true);

    use_future({
        let service = service.clone();
        move || {
            let service = service.clone();
            async move {
                if service.is_empty() {
                    return;
                }
                loop {
                    if let Ok(client) = PodmanClient::new() {
                        if let Ok(lines) = client.logs(&service, Some(200)).await {
                            let new_entries: Vec<LogEntry> = lines
                                .into_iter()
                                .filter(|l| !l.is_empty())
                                .map(|line| LogEntry {
                                    level: LogLevel::detect(&line),
                                    message: line,
                                })
                                .collect();
                            entries.set(new_entries);
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        }
    });

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
                        checked: *follow.read(),
                        oninput: move |e| follow.set(e.checked()),
                    }
                    "Follow"
                }

                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: 4px; cursor: pointer; font-size: 12px;",
                    onclick: move |_| entries.set(vec![]),
                    "Clear"
                }
            }

            // Log output
            div {
                style: "flex: 1; overflow-y: auto; padding: 8px 0; font-family: var(--fsn-font-mono); font-size: 12px;",

                if entries.read().is_empty() {
                    div {
                        style: "color: var(--fsn-color-text-muted); padding: 16px;",
                        if service.is_empty() {
                            "Select a service to view its logs."
                        } else {
                            "No logs yet for {service}."
                        }
                    }
                }

                for entry in entries.read().iter() {
                    div {
                        style: "padding: 2px 0; color: {entry.level.color()}; word-break: break-all;",
                        "{entry.message}"
                    }
                }
            }
        }
    }
}
