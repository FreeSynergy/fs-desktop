/// Service list — shows all running/stopped containers with actions.
use dioxus::prelude::*;
use fsn_container::{ContainerInfo, HealthStatus, PodmanClient, RunState};

/// A single service entry displayed in the list.
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceEntry {
    pub name: String,
    pub image: String,
    pub status: ServiceStatus,
    pub health: HealthDisplay,
    pub ports: Vec<String>,
}

impl From<ContainerInfo> for ServiceEntry {
    fn from(c: ContainerInfo) -> Self {
        Self {
            name: c.name,
            image: c.image,
            status: ServiceStatus::from(&c.state),
            health: HealthDisplay::from(&c.health),
            ports: vec![],
        }
    }
}

/// Container runtime status.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Restarting,
    Error(String),
}

impl From<&RunState> for ServiceStatus {
    fn from(s: &RunState) -> Self {
        match s {
            RunState::Running             => Self::Running,
            RunState::Exited | RunState::Stopped | RunState::Created | RunState::Paused => Self::Stopped,
            RunState::Unknown             => Self::Error("unknown".into()),
        }
    }
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

impl From<&HealthStatus> for HealthDisplay {
    fn from(h: &HealthStatus) -> Self {
        match h {
            HealthStatus::Healthy   => Self::Ok,
            HealthStatus::Unhealthy => Self::Failed,
            HealthStatus::Starting  => Self::Degraded,
            HealthStatus::None      => Self::Unknown,
        }
    }
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

/// Container lifecycle action.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
}

/// Service list component — renders all services with start/stop/restart buttons.
#[component]
pub fn ServiceList(mut selected: Signal<Option<String>>) -> Element {
    let mut services: Signal<Vec<ServiceEntry>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Poll Podman every 5 seconds
    use_future(move || async move {
        loop {
            match PodmanClient::new() {
                Ok(client) => match client.list(true).await {
                    Ok(list) => {
                        services.set(list.into_iter().map(ServiceEntry::from).collect());
                        error.set(None);
                    }
                    Err(e) => error.set(Some(format!("Failed to list containers: {e}"))),
                },
                Err(e) => error.set(Some(format!("Cannot connect to Podman: {e}"))),
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Action handler: start/stop/restart then refresh
    let on_action = move |(name, action): (String, ServiceAction)| {
        spawn(async move {
            let Ok(client) = PodmanClient::new() else { return };
            match action {
                ServiceAction::Start   => { let _ = client.start(&name).await; }
                ServiceAction::Stop    => { let _ = client.stop(&name, None).await; }
                ServiceAction::Restart => { let _ = client.restart(&name).await; }
            }
            // Refresh immediately after action
            if let Ok(list) = client.list(true).await {
                services.set(list.into_iter().map(ServiceEntry::from).collect());
            }
        });
    };

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

            // Connection error
            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-color-error); background: rgba(239,68,68,0.1); border: 1px solid var(--fsn-color-error); border-radius: 6px; padding: 12px; margin-bottom: 16px; font-size: 13px;",
                    "{err}"
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
                        for svc in services.read().iter().cloned().collect::<Vec<_>>() {
                            ServiceRow {
                                key: "{svc.name}",
                                service: svc,
                                selected,
                                on_action,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A single row in the service table.
#[component]
fn ServiceRow(
    service: ServiceEntry,
    mut selected: Signal<Option<String>>,
    on_action: EventHandler<(String, ServiceAction)>,
) -> Element {
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
                ServiceActions {
                    name: service.name.clone(),
                    status: service.status.clone(),
                    selected,
                    on_action,
                }
            }
        }
    }
}

/// Start/Stop/Restart + Logs buttons for a service.
#[component]
fn ServiceActions(
    name: String,
    status: ServiceStatus,
    mut selected: Signal<Option<String>>,
    on_action: EventHandler<(String, ServiceAction)>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; gap: 4px; justify-content: flex-end;",

            if matches!(status, ServiceStatus::Stopped | ServiceStatus::Error(_)) {
                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-success); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Start",
                    onclick: {
                        let name = name.clone();
                        move |_| on_action.call((name.clone(), ServiceAction::Start))
                    },
                    "▶"
                }
            }

            if matches!(status, ServiceStatus::Running | ServiceStatus::Restarting) {
                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-error); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Stop",
                    onclick: {
                        let name = name.clone();
                        move |_| on_action.call((name.clone(), ServiceAction::Stop))
                    },
                    "■"
                }
                button {
                    style: "padding: 4px 8px; background: var(--fsn-color-warning); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Restart",
                    onclick: {
                        let name = name.clone();
                        move |_| on_action.call((name.clone(), ServiceAction::Restart))
                    },
                    "↺"
                }
            }

            // Select service for log view
            button {
                style: "padding: 4px 8px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: 4px; cursor: pointer; font-size: 12px;",
                title: "Logs",
                onclick: {
                    let name = name.clone();
                    move |_| *selected.write() = Some(name.clone())
                },
                "≡"
            }
        }
    }
}
