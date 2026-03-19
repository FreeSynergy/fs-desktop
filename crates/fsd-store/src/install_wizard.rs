/// Install logic and result popup — replaces the multi-step install wizard.
use dioxus::prelude::*;
use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
use fsn_store::StoreClient;

use crate::node_package::PackageKind;
use crate::package_card::PackageEntry;

// ── InstallResult ──────────────────────────────────────────────────────────────

/// Result of an install operation.
#[derive(Clone, PartialEq, Debug)]
pub enum InstallResult {
    Success,
    Failed(String),
}

// ── async install logic ────────────────────────────────────────────────────────

/// Downloads and registers a package. Returns Ok on success.
pub async fn do_install(package: PackageEntry, env_vars: String) -> Result<(), String> {
    let home    = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let fsn_dir = std::path::PathBuf::from(&home).join(".local/share/fsn");

    let store_path = package.store_path.as_deref()
        .map(|p| p.trim_end_matches('/').to_string());

    let file_path: Option<String> = match &package.kind {
        PackageKind::Container => {
            install_container(&package, store_path, &fsn_dir, &env_vars).await?
        }
        PackageKind::Language => {
            let base = store_path.unwrap_or_else(|| format!("shared/i18n/{}", package.id));
            let url  = format!("{base}/ui.toml");
            match StoreClient::node_store().fetch_raw(&url).await {
                Ok(content) => {
                    let dest_dir = fsn_dir.join("i18n").join(&package.id);
                    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
                    let dest = dest_dir.join("ui.toml");
                    std::fs::write(&dest, content).map_err(|e| e.to_string())?;
                    Some(dest.to_string_lossy().into_owned())
                }
                Err(e) => {
                    tracing::warn!("Language pack download failed (registering anyway): {e}");
                    None
                }
            }
        }
        PackageKind::Theme => {
            let base = store_path.unwrap_or_else(|| format!("shared/themes/{}", package.id));
            let url  = format!("{base}/theme.css");
            match StoreClient::node_store().fetch_raw(&url).await {
                Ok(content) => {
                    let dest_dir = fsn_dir.join("themes");
                    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
                    let dest = dest_dir.join(format!("{}.css", package.id));
                    std::fs::write(&dest, content).map_err(|e| e.to_string())?;
                    Some(dest.to_string_lossy().into_owned())
                }
                Err(e) => {
                    tracing::warn!("Theme download failed (registering anyway): {e}");
                    None
                }
            }
        }
        // Widget, Bot, Task, Bridge, Plugin — register without file download
        _ => None,
    };

    PackageRegistry::install(InstalledPackage {
        id:        package.id.clone(),
        name:      package.name.clone(),
        kind:      package.kind.kind_str(),
        version:   package.version.clone(),
        icon:      String::new(),
        file_path,
    })
    .map_err(|e| format!("Registry error: {e}"))
}

/// Full container install:
///  1. Fetch compose file from store
///  2. Write compose + .env to ~/.local/share/fsn/services/<id>/
///  3. Try `fsn conductor install <compose_path>` (adds Quadlet + systemd unit)
///  4. systemctl --user daemon-reload
async fn install_container(
    package:    &PackageEntry,
    store_path: Option<String>,
    fsn_dir:    &std::path::Path,
    env_vars:   &str,
) -> Result<Option<String>, String> {
    let base = store_path.unwrap_or_else(|| format!("node/modules/{}", package.id));

    // Try compose.yml first, then docker-compose.yml
    let compose_content = {
        let mut content = None;
        for name in &["compose.yml", "docker-compose.yml", "container.yml"] {
            let url = format!("{base}/{name}");
            if let Ok(c) = StoreClient::node_store().fetch_raw(&url).await {
                content = Some((c, *name));
                break;
            }
        }
        content
    };

    let service_dir = fsn_dir.join("services").join(&package.id);
    std::fs::create_dir_all(&service_dir).map_err(|e| e.to_string())?;

    let compose_path = match compose_content {
        Some((content, filename)) => {
            let dest = service_dir.join(filename);
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            dest
        }
        None => {
            // No compose file in store — create a placeholder so the user can
            // edit it manually and re-run `fsn conductor install`.
            let dest = service_dir.join("compose.yml");
            std::fs::write(&dest, format!(
                "# Compose file for {name}\n\
                 # Edit this file and run: fsn conductor install {path}\n\
                 services:\n\
                 #  {id}:\n\
                 #    image: ...\n",
                name = package.name,
                id   = package.id,
                path = dest.display(),
            )).map_err(|e| e.to_string())?;
            dest
        }
    };

    // Write .env file
    if !env_vars.trim().is_empty() {
        let env_path = service_dir.join(".env");
        std::fs::write(&env_path, env_vars).map_err(|e| e.to_string())?;
    }

    // Try `fsn conductor install <compose_path>`
    let compose_str = compose_path.to_string_lossy().into_owned();
    let conductor_result = tokio::process::Command::new("fsn")
        .args(["conductor", "install", &compose_str])
        .output()
        .await;

    match conductor_result {
        Ok(out) if out.status.success() => {
            // Reload systemd so the new Quadlet unit is picked up
            let _ = tokio::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
                .await;
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(
                "fsn conductor install returned non-zero: {stderr}. \
                 Compose file saved to {compose_str} — run manually to finish setup."
            );
        }
        Err(_) => {
            tracing::warn!(
                "`fsn` binary not found. Compose file saved to {compose_str}. \
                 Run `fsn conductor install {compose_str}` manually to activate the service."
            );
        }
    }

    Ok(Some(compose_str))
}

// ── Fetch detected env vars from compose file ─────────────────────────────────

/// Fetches a compose file from the store and extracts environment variable
/// names so the configure step can show pre-populated input fields.
pub async fn fetch_container_env_vars(package: &PackageEntry) -> Vec<String> {
    let base = package.store_path.as_deref()
        .map(|p| p.trim_end_matches('/').to_string())
        .unwrap_or_else(|| format!("node/modules/{}", package.id));

    for name in &["compose.yml", "docker-compose.yml", "container.yml"] {
        let url = format!("{base}/{name}");
        if let Ok(content) = StoreClient::node_store().fetch_raw(&url).await {
            return extract_env_var_names(&content);
        }
    }
    vec![]
}

/// Simple line-by-line extraction of `KEY` or `KEY: ...` or `KEY=...` from a
/// YAML environment section. Good enough for showing input fields; the
/// Conductor will do the proper analysis when installing.
pub fn extract_env_var_names(yaml: &str) -> Vec<String> {
    let mut in_env = false;
    let mut vars   = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim();

        // Detect `environment:` section start
        if trimmed == "environment:" || trimmed.starts_with("environment:") {
            in_env = true;
            continue;
        }

        // Detect next top-level key (ends env section)
        if in_env && !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
            in_env = false;
        }

        if in_env {
            // `  - KEY=value`  or  `  - KEY`
            let entry = trimmed.trim_start_matches("- ");
            // `  KEY: value`
            let key = if let Some(pos) = entry.find('=') {
                &entry[..pos]
            } else if let Some(pos) = entry.find(':') {
                &entry[..pos]
            } else {
                entry
            };
            let key = key.trim();
            if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                vars.push(key.to_string());
            }
        }
    }
    vars
}

// ── InstallPopup ───────────────────────────────────────────────────────────────

/// Simple centered modal overlay shown after an install attempt.
#[component]
pub fn InstallPopup(result: InstallResult, on_close: EventHandler<()>) -> Element {
    let (icon, title, detail) = match &result {
        InstallResult::Success => (
            "✅",
            fsn_i18n::t("store.install.success"),
            String::new(),
        ),
        InstallResult::Failed(err) => (
            "❌",
            fsn_i18n::t("store.install.failed"),
            err.clone(),
        ),
    };

    rsx! {
        // Overlay backdrop
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.55); \
                    display: flex; align-items: center; justify-content: center; z-index: 2000;",
            onclick: move |_| on_close.call(()),

            // Modal card — stop propagation so clicking inside doesn't close
            div {
                style: "background: var(--fsn-color-bg-surface); \
                        border: 1px solid var(--fsn-color-border-default); \
                        border-radius: var(--fsn-radius-lg); \
                        padding: 40px 32px; max-width: 360px; width: 100%; \
                        text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.4);",
                onclick: move |e| e.stop_propagation(),

                p { style: "font-size: 48px; margin: 0 0 16px 0;", "{icon}" }
                p { style: "font-size: 18px; font-weight: 600; margin: 0 0 12px 0;",
                    "{title}"
                }
                if !detail.is_empty() {
                    p {
                        style: "font-size: 13px; color: var(--fsn-color-error, #ef4444); \
                                background: rgba(239,68,68,0.08); \
                                border: 1px solid var(--fsn-color-error, #ef4444); \
                                border-radius: var(--fsn-radius-md); \
                                padding: 10px 12px; margin: 0 0 20px 0; \
                                text-align: left; word-break: break-word;",
                        "{detail}"
                    }
                }
                button {
                    style: "padding: 10px 32px; background: var(--fsn-color-primary); \
                            color: white; border: none; \
                            border-radius: var(--fsn-radius-md); cursor: pointer; \
                            font-size: 14px; font-weight: 600;",
                    onclick: move |_| on_close.call(()),
                    {fsn_i18n::t("actions.close")}
                }
            }
        }
    }
}
