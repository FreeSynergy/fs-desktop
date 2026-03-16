/// Installed list — shows FSN systemd services with Remove button.
///
/// Lists all fsn-*.service units via systemctl --user (no Podman socket).
use dioxus::prelude::*;
use fsn_container::SystemctlManager;

// ── InstalledEntry ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct InstalledEntry {
    pub name:    String,
    pub running: bool,
}

// ── list helper ───────────────────────────────────────────────────────────────

async fn list_fsn_units() -> Vec<String> {
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

// ── InstalledList ─────────────────────────────────────────────────────────────

/// Component that lists installed FSN services with a Remove button.
#[component]
pub fn InstalledList(catalog_versions: Vec<(String, String)>) -> Element {
    let mut entries: Signal<Vec<InstalledEntry>>  = use_signal(Vec::new);
    let mut error:   Signal<Option<String>>        = use_signal(|| None);
    let mut confirm: Signal<Option<InstalledEntry>> = use_signal(|| None);

    // Fetch services every 10 seconds
    use_future(move || async move {
        let mgr = SystemctlManager::user();
        loop {
            let units = list_fsn_units().await;
            if units.is_empty() {
                error.set(Some("No installed FSN services found.".into()));
            } else {
                let mut rows = Vec::new();
                for unit in &units {
                    let running = mgr.is_active(unit).await.unwrap_or(false);
                    rows.push(InstalledEntry { name: unit.clone(), running });
                }
                entries.set(rows);
                error.set(None);
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    rsx! {
        div {
            // Confirm dialog overlay
            if let Some(entry) = confirm.read().clone() {
                RemoveConfirmDialog {
                    entry: entry.clone(),
                    on_confirm: move |_| {
                        let entry = entry.clone();
                        spawn(async move {
                            let mgr = SystemctlManager::user();
                            let _ = mgr.stop(&entry.name).await;
                            let _ = mgr.disable(&entry.name).await;
                        });
                        *confirm.write() = None;
                    },
                    on_cancel: move |_| *confirm.write() = None,
                }
            }

            // Error
            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-warning); font-size: 13px; margin-bottom: 12px;",
                    "{err}"
                }
            }

            if entries.read().is_empty() && error.read().is_none() {
                div {
                    style: "text-align: center; color: var(--fsn-text-muted); padding: 48px;",
                    p { "No installed FreeSynergy services found." }
                    p { style: "font-size: 12px;",
                        "Deploy a project with "
                        code { "fsn deploy" }
                        " to install services."
                    }
                }
            } else if !entries.read().is_empty() {
                table {
                    style: "width: 100%; border-collapse: collapse;",
                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fsn-border); font-size: 12px; color: var(--fsn-text-muted);",
                            th { style: "text-align: left; padding: 8px;",  "SERVICE" }
                            th { style: "text-align: left; padding: 8px;",  "STATUS" }
                            th { style: "text-align: right; padding: 8px;", "ACTIONS" }
                        }
                    }
                    tbody {
                        for entry in entries.read().iter().cloned().collect::<Vec<_>>() {
                            InstalledRow {
                                entry: entry.clone(),
                                on_remove: move |e: InstalledEntry| {
                                    *confirm.write() = Some(e);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── InstalledRow ──────────────────────────────────────────────────────────────

#[component]
fn InstalledRow(
    entry: InstalledEntry,
    on_remove: EventHandler<InstalledEntry>,
) -> Element {
    let status_color = if entry.running { "var(--fsn-success)" } else { "var(--fsn-text-muted)" };
    let status_label = if entry.running { "Running" } else { "Stopped" };

    rsx! {
        tr {
            style: "border-bottom: 1px solid var(--fsn-border);",

            td { style: "padding: 10px 8px; font-weight: 500; font-size: 13px;", "{entry.name}" }
            td { style: "padding: 10px 8px;",
                span { style: "font-size: 13px; color: {status_color};", "{status_label}" }
            }
            td { style: "padding: 10px 8px; text-align: right;",
                button {
                    style: "padding: 4px 10px; background: var(--fsn-error); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    onclick: {
                        let e = entry.clone();
                        move |_| on_remove.call(e.clone())
                    },
                    "Remove"
                }
            }
        }
    }
}

// ── RemoveConfirmDialog ───────────────────────────────────────────────────────

#[component]
fn RemoveConfirmDialog(
    entry: InstalledEntry,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            div {
                style: "background: var(--fsn-bg-surface); border: 1px solid var(--fsn-border); border-radius: var(--fsn-radius-lg); padding: 24px; max-width: 400px; width: 100%;",
                h3 { style: "margin: 0 0 12px 0;", "Remove {entry.name}?" }
                p {
                    style: "color: var(--fsn-text-muted); font-size: 14px; margin-bottom: 20px;",
                    "This will stop the service and disable its systemd unit. "
                    "Data volumes will not be deleted."
                }
                div {
                    style: "display: flex; gap: 8px; justify-content: flex-end;",
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-bg-elevated); border: 1px solid var(--fsn-border); border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-error); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| on_confirm.call(()),
                        "Remove"
                    }
                }
            }
        }
    }
}
