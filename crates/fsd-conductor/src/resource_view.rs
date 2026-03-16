/// Resource view — service status overview via SystemctlManager.
///
/// CPU/RAM stats require the Podman socket (removed).
/// Use `podman stats` or system monitoring tools for resource metrics.
use dioxus::prelude::*;
use fsn_container::{SystemctlManager, UnitActiveState};

use crate::service_list::list_fsn_units;

// ── ResourceEntry ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct ResourceEntry {
    name:      String,
    active:    UnitActiveState,
    sub_state: String,
}

// ── ResourceView ──────────────────────────────────────────────────────────────

/// Service status table (replaces CPU/RAM stats — use system monitoring for metrics).
#[component]
pub fn ResourceView() -> Element {
    let mut entries: Signal<Vec<ResourceEntry>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>>       = use_signal(|| None);

    // Poll every 5 seconds
    use_future(move || async move {
        let mgr = SystemctlManager::user();
        loop {
            let units = list_fsn_units().await;
            if units.is_empty() {
                error.set(Some("No FSN services found.".into()));
            } else {
                let mut rows = Vec::new();
                for unit in &units {
                    let (active, sub) = match mgr.service_status(unit).await {
                        Ok(s)  => (s.active_state, s.sub_state),
                        Err(_) => (UnitActiveState::Unknown, String::new()),
                    };
                    rows.push(ResourceEntry { name: unit.clone(), active, sub_state: sub });
                }
                entries.set(rows);
                error.set(None);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    rsx! {
        div {
            class: "fsd-resource-view",

            // Header
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;",
                h2 { style: "margin: 0; font-size: 18px;", "Resources" }
                span {
                    style: "font-size: 12px; color: var(--fsn-text-muted);",
                    "For CPU/RAM metrics use: "
                    code { "podman stats" }
                }
            }

            // Error / empty state
            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-warning); background: var(--fsn-warning-bg); border: 1px solid var(--fsn-warning); border-radius: 6px; padding: 12px; margin-bottom: 16px; font-size: 13px;",
                    "{err}"
                }
            }

            if entries.read().is_empty() && error.read().is_none() {
                div {
                    style: "text-align: center; color: var(--fsn-text-muted); padding: 48px;",
                    "No running containers."
                }
            } else if !entries.read().is_empty() {
                table {
                    style: "width: 100%; border-collapse: collapse;",

                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fsn-border); font-size: 12px; color: var(--fsn-text-muted);",
                            th { style: "text-align: left; padding: 8px;",  "SERVICE" }
                            th { style: "text-align: left; padding: 8px;",  "STATE" }
                            th { style: "text-align: left; padding: 8px;",  "SUB-STATE" }
                        }
                    }

                    tbody {
                        for e in entries.read().iter().cloned().collect::<Vec<_>>() {
                            ResourceRow { entry: e }
                        }
                    }
                }
            }
        }
    }
}

// ── ResourceRow ───────────────────────────────────────────────────────────────

#[component]
fn ResourceRow(entry: ResourceEntry) -> Element {
    let color = match entry.active {
        UnitActiveState::Active       => "var(--fsn-success)",
        UnitActiveState::Failed       => "var(--fsn-error)",
        UnitActiveState::Activating
        | UnitActiveState::Deactivating => "var(--fsn-warning)",
        _                             => "var(--fsn-text-muted)",
    };
    rsx! {
        tr {
            style: "border-bottom: 1px solid var(--fsn-border);",
            td { style: "padding: 10px 8px; font-weight: 500; font-size: 13px;", "{entry.name}" }
            td { style: "padding: 10px 8px;",
                span { style: "font-size: 13px; color: {color};", "{entry.active}" }
            }
            td { style: "padding: 10px 8px; font-size: 12px; color: var(--fsn-text-muted);",
                "{entry.sub_state}"
            }
        }
    }
}
