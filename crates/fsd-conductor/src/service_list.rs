/// Service list — shows all FSN systemd services with start/stop/restart actions.
///
/// Uses SystemctlManager (no Podman socket) to list and control services.
use dioxus::prelude::*;
use fsn_container::{SystemctlManager, UnitActiveState};

/// A single service entry displayed in the list.
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceEntry {
    pub name:        String,
    pub active:      UnitActiveState,
    pub sub_state:   String,
    pub description: String,
}

impl ServiceEntry {
    pub fn status_label(&self) -> &str {
        match self.active {
            UnitActiveState::Active       => "Running",
            UnitActiveState::Inactive     => "Stopped",
            UnitActiveState::Activating   => "Starting",
            UnitActiveState::Deactivating => "Stopping",
            UnitActiveState::Failed       => "Failed",
            UnitActiveState::Unknown      => "Unknown",
        }
    }

    pub fn status_color(&self) -> &str {
        match self.active {
            UnitActiveState::Active       => "var(--fsn-success)",
            UnitActiveState::Inactive     => "var(--fsn-text-muted)",
            UnitActiveState::Activating   => "var(--fsn-info)",
            UnitActiveState::Deactivating => "var(--fsn-warning)",
            UnitActiveState::Failed       => "var(--fsn-error)",
            UnitActiveState::Unknown      => "var(--fsn-text-muted)",
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

/// Fetch all fsn-*.service unit names via systemctl --user.
pub async fn list_fsn_units() -> Vec<String> {
    let Ok(out) = tokio::process::Command::new("systemctl")
        .args(["--user", "list-units", "--type=service", "--no-legend", "--plain", "--all"])
        .output()
        .await
    else {
        return vec![];
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let unit = line.split_whitespace().next()?;
            if unit.starts_with("fsn-") && unit.ends_with(".service") {
                Some(unit.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Service list component — renders all FSN services with start/stop/restart buttons.
#[component]
pub fn ServiceList(mut selected: Signal<Option<String>>) -> Element {
    let mut services: Signal<Vec<ServiceEntry>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Poll every 5 seconds
    use_future(move || async move {
        let mgr = SystemctlManager::user();
        loop {
            let units = list_fsn_units().await;
            if units.is_empty() {
                error.set(Some("No FSN services found. Deploy a project first.".into()));
            } else {
                let mut entries = Vec::new();
                for unit in &units {
                    match mgr.service_status(unit).await {
                        Ok(s) => entries.push(ServiceEntry {
                            name:        s.name.clone(),
                            active:      s.active_state.clone(),
                            sub_state:   s.sub_state.clone(),
                            description: s.description.clone(),
                        }),
                        Err(_) => entries.push(ServiceEntry {
                            name:        unit.clone(),
                            active:      UnitActiveState::Unknown,
                            sub_state:   String::new(),
                            description: String::new(),
                        }),
                    }
                }
                services.set(entries);
                error.set(None);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Action handler: start/stop/restart then refresh
    let on_action = move |(name, action): (String, ServiceAction)| {
        spawn(async move {
            let mgr = SystemctlManager::user();
            let _ = match action {
                ServiceAction::Start   => mgr.start(&name).await,
                ServiceAction::Stop    => mgr.stop(&name).await,
                ServiceAction::Restart => mgr.restart(&name).await,
            };
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
                    style: "background: var(--fsn-primary); color: white; border: none; padding: 8px 16px; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    "Install Service"
                }
            }

            // Connection error / empty state
            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-warning); background: var(--fsn-warning-bg); border: 1px solid var(--fsn-warning); border-radius: 6px; padding: 12px; margin-bottom: 16px; font-size: 13px;",
                    "{err}"
                }
            }

            // Table
            if services.read().is_empty() && error.read().is_none() {
                div {
                    style: "text-align: center; color: var(--fsn-text-muted); padding: 48px;",
                    p { "No services installed yet." }
                    p { "Open the Store to install your first service." }
                }
            } else if !services.read().is_empty() {
                table {
                    style: "width: 100%; border-collapse: collapse;",

                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fsn-border); font-size: 12px; color: var(--fsn-text-muted);",
                            th { style: "text-align: left; padding: 8px;", "NAME" }
                            th { style: "text-align: left; padding: 8px;", "STATUS" }
                            th { style: "text-align: left; padding: 8px;", "STATE" }
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
            style: "border-bottom: 1px solid var(--fsn-border); cursor: pointer;",
            onclick: move |_| *selected.write() = Some(name.clone()),

            td { style: "padding: 12px 8px; font-weight: 500; font-size: 13px;", "{service.name}" }
            td { style: "padding: 12px 8px;",
                span {
                    style: "color: {service.status_color()}; font-size: 13px;",
                    "{service.status_label()}"
                }
            }
            td { style: "padding: 12px 8px; font-size: 12px; color: var(--fsn-text-muted);",
                "{service.sub_state}"
            }
            td { style: "padding: 12px 8px; text-align: right;",
                ServiceActions {
                    name: service.name.clone(),
                    active: service.active.clone(),
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
    active: UnitActiveState,
    mut selected: Signal<Option<String>>,
    on_action: EventHandler<(String, ServiceAction)>,
) -> Element {
    let is_running = matches!(active, UnitActiveState::Active);
    rsx! {
        div {
            style: "display: flex; gap: 4px; justify-content: flex-end;",

            if !is_running {
                button {
                    style: "padding: 4px 8px; background: var(--fsn-success); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Start",
                    onclick: {
                        let n = name.clone();
                        move |_| on_action.call((n.clone(), ServiceAction::Start))
                    },
                    "▶"
                }
            }

            if is_running {
                button {
                    style: "padding: 4px 8px; background: var(--fsn-error); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Stop",
                    onclick: {
                        let n = name.clone();
                        move |_| on_action.call((n.clone(), ServiceAction::Stop))
                    },
                    "■"
                }
                button {
                    style: "padding: 4px 8px; background: var(--fsn-warning); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: "Restart",
                    onclick: {
                        let n = name.clone();
                        move |_| on_action.call((n.clone(), ServiceAction::Restart))
                    },
                    "↺"
                }
            }

            // Select service for log view
            button {
                style: "padding: 4px 8px; background: var(--fsn-bg-surface); border: 1px solid var(--fsn-border); border-radius: 4px; cursor: pointer; font-size: 12px;",
                title: "Logs",
                onclick: {
                    let n = name.clone();
                    move |_| *selected.write() = Some(n.clone())
                },
                "≡"
            }
        }
    }
}
