/// Service list — shows all running/stopped containers with actions.
use dioxus::prelude::*;

/// A single service entry displayed in the list.
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceEntry {
    pub name: String,
    pub image: String,
    pub status: ServiceStatus,
    pub health: HealthDisplay,
    pub ports: Vec<String>,
}

/// Container runtime status.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Restarting,
    Error(String),
}

impl ServiceStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Running    => "Running",
            Self::Stopped    => "Stopped",
            Self::Restarting => "Restarting",
            Self::Error(_)   => "Error",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Running    => "var(--fsn-color-success)",
            Self::Stopped    => "var(--fsn-color-text-muted)",
            Self::Restarting => "var(--fsn-color-warning)",
            Self::Error(_)   => "var(--fsn-color-error)",
        }
    }
}

/// Health check display state.
#[derive(Clone, Debug, PartialEq)]
pub enum HealthDisplay {
    Ok,
    Degraded,
    Failed,
    Unknown,
}

impl HealthDisplay {
    pub fn icon(&self) -> &str {
        match self {
            Self::Ok      => "✓",
            Self::Degraded=> "⚠",
            Self::Failed  => "✗",
            Self::Unknown => "?",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Ok      => "var(--fsn-color-success)",
            Self::Degraded=> "var(--fsn-color-warning)",
            Self::Failed  => "var(--fsn-color-error)",
            Self::Unknown => "var(--fsn-color-text-muted)",
        }
    }
}

/// Service list component — renders all services with start/stop/restart buttons.
#[component]
pub fn ServiceList(mut selected: Signal<Option<String>>) -> Element {
    // TODO: load from fsn-container via use_resource
    let services = use_signal(Vec::<ServiceEntry>::new);

    rsx! {
        div {
            class: "fsd-service-list",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                h2 { style: "margin: 0; font-size: 18px;", "Services" }
                button {
                    style: "background: var(--fsn-color-primary); color: white; border: none; padding: 8px 16px; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    "Install Service"
                }
            }

            // Table
            if services.read().is_empty() {
                div {
                    style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                    p { "No services installed yet." }
                    p { "Open the Store to install your first service." }
                }
            } else {
                table {
                    style: "width: 100%; border-collapse: collapse;",

                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fsn-color-border-default); font-size: 12px; color: var(--fsn-color-text-muted);",
                            th { style: "text-align: left; padding: 8px;", "NAME" }
                            th { style: "text-align: left; padding: 8px;", "STATUS" }
                            th { style: "text-align: left; padding: 8px;", "HEALTH" }
                            th { style: "text-align: left; padding: 8px;", "PORTS" }
                            th { style: "text-align: right; padding: 8px;", "ACTIONS" }
                        }
                    }

                    tbody {
                        for svc in services.read().iter() {
                            ServiceRow { service: svc.clone(), selected }
                        }
                    }
                }
            }
        }
    }
}

/// A single row in the service table.
#[component]
fn ServiceRow(service: ServiceEntry, mut selected: Signal<Option<String>>) -> Element {
    let name = service.name.clone();
    rsx! {
        tr {
            style: "border-bottom: 1px solid var(--fsn-color-border-default); cursor: pointer;",
            onclick: move |_| *selected.write() = Some(name.clone()),

            td { style: "padding: 12px 8px; font-weight: 500;", "{service.name}" }
            td { style: "padding: 12px 8px;",
                span {
                    style: "color: {service.status.color()}; font-size: 13px;",
                    "{service.status.label()}"
                }
            }
            td { style: "padding: 12px 8px;",
                span {
                    style: "color: {service.health.color()};",
                    "{service.health.icon()}"
                }
            }
            td { style: "padding: 12px 8px; font-size: 12px; color: var(--fsn-color-text-muted);",
                "{service.ports.join(\", \")}"
            }
            td { style: "padding: 12px 8px; text-align: right;",
                ServiceActions { name: service.name.clone(), status: service.status.clone() }
            }
        }
    }
}

/// Start/Stop/Restart buttons for a service.
#[component]
fn ServiceActions(name: String, status: ServiceStatus) -> Element {
    rsx! {
        div {
            style: "display: flex; gap: 4px; justify-content: flex-end;",

            if matches!(status, ServiceStatus::Stopped | ServiceStatus::Error(_)) {
                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-success); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Start",
                    // TODO: call fsn-container start
                    "▶"
                }
            }

            if matches!(status, ServiceStatus::Running | ServiceStatus::Restarting) {
                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-error); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Stop",
                    "■"
                }
                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-warning); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Restart",
                    "↺"
                }
            }

            button {
                style: "padding: 4px 8px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: 4px; cursor: pointer; font-size: 12px;",
                title: "Logs",
                "≡"
            }
        }
    }
}
