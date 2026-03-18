/// Installed list — shows FSN systemd services and Store-installed packages.
///
/// Section 1: fsn-*.service units via systemctl --user.
/// Section 2: All packages registered in PackageRegistry (languages, themes, widgets, …).
use dioxus::prelude::*;
use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
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

/// Component that lists installed FSN services and Store packages with Remove buttons.
#[component]
pub fn InstalledList(catalog_versions: Vec<(String, String)>) -> Element {
    let mut entries: Signal<Vec<InstalledEntry>>      = use_signal(Vec::new);
    let mut error:   Signal<Option<String>>            = use_signal(|| None);
    let mut confirm: Signal<Option<InstalledEntry>>    = use_signal(|| None);
    // Registry packages (languages, themes, widgets, …)
    let mut reg_pkgs: Signal<Vec<InstalledPackage>>   = use_signal(|| PackageRegistry::load());
    let mut reg_confirm: Signal<Option<InstalledPackage>> = use_signal(|| None);

    // Fetch services every 10 seconds
    use_future(move || async move {
        let mgr = SystemctlManager::user();
        loop {
            let units = list_fsn_units().await;
            if units.is_empty() {
                error.set(Some(fsn_i18n::t("store.installed.no_services")));
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
            // Service remove confirm dialog
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

            // Registry package remove confirm dialog
            if let Some(pkg) = reg_confirm.read().clone() {
                RegRemoveDialog {
                    pkg: pkg.clone(),
                    on_confirm: move |_| {
                        let id = pkg.id.clone();
                        let _ = PackageRegistry::remove(&id);
                        reg_pkgs.set(PackageRegistry::load());
                        *reg_confirm.write() = None;
                    },
                    on_cancel: move |_| *reg_confirm.write() = None,
                }
            }

            // ── Section 1: Systemd services ──────────────────────────────────
            h3 {
                style: "font-size: 13px; font-weight: 600; text-transform: uppercase; \
                        letter-spacing: 0.07em; color: var(--fsn-color-text-muted); \
                        margin: 0 0 12px 0;",
                {fsn_i18n::t("store.tab.services")}
            }

            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-color-warning, #f59e0b); font-size: 13px; margin-bottom: 12px;",
                    "{err}"
                }
            }

            if entries.read().is_empty() && error.read().is_none() {
                div {
                    style: "color: var(--fsn-color-text-muted); font-size: 13px; padding: 12px 0 24px 0;",
                    {fsn_i18n::t("store.installed.no_services")}
                }
            } else if !entries.read().is_empty() {
                table {
                    style: "width: 100%; border-collapse: collapse; margin-bottom: 32px;",
                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fsn-color-border-default); \
                                    font-size: 12px; color: var(--fsn-color-text-muted);",
                            th { style: "text-align: left; padding: 8px;",  "SERVICE" }
                            th { style: "text-align: left; padding: 8px;",  "STATUS" }
                            th { style: "text-align: right; padding: 8px;", "ACTIONS" }
                        }
                    }
                    tbody {
                        for entry in entries.read().iter().cloned().collect::<Vec<_>>() {
                            InstalledRow {
                                key: "{entry.name}",
                                entry: entry.clone(),
                                on_remove: move |e: InstalledEntry| {
                                    *confirm.write() = Some(e);
                                },
                            }
                        }
                    }
                }
            }

            // ── Section 2: Store packages ────────────────────────────────────
            h3 {
                style: "font-size: 13px; font-weight: 600; text-transform: uppercase; \
                        letter-spacing: 0.07em; color: var(--fsn-color-text-muted); \
                        margin: 0 0 12px 0;",
                {fsn_i18n::t("store.section.store_packages")}
            }

            {
                let pkgs = reg_pkgs.read();
                if pkgs.is_empty() {
                    rsx! {
                        div {
                            style: "color: var(--fsn-color-text-muted); font-size: 13px; padding: 12px 0;",
                            {fsn_i18n::t("store.installed.no_packages")}
                        }
                    }
                } else {
                    rsx! {
                        table {
                            style: "width: 100%; border-collapse: collapse;",
                            thead {
                                tr {
                                    style: "border-bottom: 1px solid var(--fsn-color-border-default); \
                                            font-size: 12px; color: var(--fsn-color-text-muted);",
                                    th { style: "text-align: left; padding: 8px;",  "NAME" }
                                    th { style: "text-align: left; padding: 8px;",  "KIND" }
                                    th { style: "text-align: left; padding: 8px;",  "VERSION" }
                                    th { style: "text-align: right; padding: 8px;", "ACTIONS" }
                                }
                            }
                            tbody {
                                for pkg in pkgs.iter().cloned().collect::<Vec<_>>() {
                                    RegPackageRow {
                                        key: "{pkg.id}",
                                        pkg: pkg.clone(),
                                        on_remove: move |p: InstalledPackage| {
                                            *reg_confirm.write() = Some(p);
                                        },
                                    }
                                }
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
    let status_label = if entry.running {
        fsn_i18n::t("status.running")
    } else {
        fsn_i18n::t("status.stopped")
    };

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
                    {fsn_i18n::t("actions.remove")}
                }
            }
        }
    }
}

// ── RegPackageRow ─────────────────────────────────────────────────────────────

#[component]
fn RegPackageRow(
    pkg: InstalledPackage,
    on_remove: EventHandler<InstalledPackage>,
) -> Element {
    rsx! {
        tr {
            style: "border-bottom: 1px solid var(--fsn-color-border-default);",
            td { style: "padding: 10px 8px; font-weight: 500; font-size: 13px;", "{pkg.name}" }
            td { style: "padding: 10px 8px;",
                span {
                    style: "font-size: 11px; padding: 2px 8px; border-radius: 999px; \
                            background: var(--fsn-color-bg-overlay); \
                            border: 1px solid var(--fsn-color-border-default); \
                            color: var(--fsn-color-text-muted);",
                    "{pkg.kind}"
                }
            }
            td { style: "padding: 10px 8px; font-size: 13px; color: var(--fsn-color-text-muted);",
                "v{pkg.version}"
            }
            td { style: "padding: 10px 8px; text-align: right;",
                button {
                    style: "padding: 4px 10px; background: var(--fsn-color-error, #ef4444); \
                            color: white; border: none; border-radius: 4px; cursor: pointer; \
                            font-size: 12px;",
                    onclick: {
                        let p = pkg.clone();
                        move |_| on_remove.call(p.clone())
                    },
                    {fsn_i18n::t("actions.remove")}
                }
            }
        }
    }
}

// ── RegRemoveDialog ───────────────────────────────────────────────────────────

#[component]
fn RegRemoveDialog(
    pkg: InstalledPackage,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); \
                    display: flex; align-items: center; justify-content: center; z-index: 1000;",
            div {
                style: "background: var(--fsn-color-bg-surface); \
                        border: 1px solid var(--fsn-color-border-default); \
                        border-radius: var(--fsn-radius-lg); padding: 24px; \
                        max-width: 400px; width: 100%;",
                h3 { style: "margin: 0 0 12px 0;",
                    {fsn_i18n::t_with("store.dialog.remove_title", &[("name", pkg.name.as_str())])}
                }
                p {
                    style: "color: var(--fsn-color-text-muted); font-size: 14px; margin-bottom: 20px;",
                    {fsn_i18n::t("store.dialog.remove_body")}
                }
                div {
                    style: "display: flex; gap: 8px; justify-content: flex-end;",
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| on_cancel.call(()),
                        {fsn_i18n::t("actions.cancel")}
                    }
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-color-error, #ef4444); \
                                color: white; border: none; \
                                border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| on_confirm.call(()),
                        {fsn_i18n::t("actions.remove")}
                    }
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
                style: "background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-lg); padding: 24px; max-width: 400px; width: 100%;",
                h3 { style: "margin: 0 0 12px 0;",
                    {fsn_i18n::t_with("store.dialog.remove_service_title", &[("name", entry.name.as_str())])}
                }
                p {
                    style: "color: var(--fsn-color-text-muted); font-size: 14px; margin-bottom: 20px;",
                    {fsn_i18n::t("store.dialog.remove_service_body")}
                }
                div {
                    style: "display: flex; gap: 8px; justify-content: flex-end;",
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| on_cancel.call(()),
                        {fsn_i18n::t("actions.cancel")}
                    }
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-color-error, #ef4444); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| on_confirm.call(()),
                        {fsn_i18n::t("actions.remove")}
                    }
                }
            }
        }
    }
}
