/// Install wizard — step-by-step configuration before first start.
use dioxus::prelude::*;
use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
use fsn_store::StoreClient;

use crate::node_package::PackageKind;
use crate::package_card::PackageEntry;

#[derive(Clone, PartialEq, Debug)]
pub enum WizardStep {
    Overview,
    Configure,
    Confirm,
    Installing,
    Done,
    Error,
}

impl WizardStep {
    pub fn label(&self) -> &str {
        match self {
            Self::Overview   => "Overview",
            Self::Configure  => "Configure",
            Self::Confirm    => "Confirm",
            Self::Installing => "Installing",
            Self::Done       => "Done",
            Self::Error      => "Error",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Overview   => Some(Self::Configure),
            Self::Configure  => Some(Self::Confirm),
            Self::Confirm    => Some(Self::Installing),
            // Installing transitions via async callback, not next()
            _ => None,
        }
    }
}

// ── async install logic ────────────────────────────────────────────────────────

/// Downloads and registers a package. Returns Ok on success.
async fn do_install(package: PackageEntry, env_vars: String) -> Result<(), String> {
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
fn extract_env_var_names(yaml: &str) -> Vec<String> {
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

// ── InstallWizard component ────────────────────────────────────────────────────

/// Install wizard — guides the user through pre-install configuration.
#[component]
pub fn InstallWizard(package: PackageEntry, on_cancel: EventHandler<()>) -> Element {
    let mut step          = use_signal(|| WizardStep::Overview);
    let mut install_error = use_signal(|| Option::<String>::None);

    // Container-specific: env var keys detected from compose file
    let mut detected_vars: Signal<Vec<String>> = use_signal(Vec::new);
    // Container-specific: user-entered env var values (KEY=VALUE per line in textarea)
    let mut env_vars_text: Signal<String>      = use_signal(String::new);
    // Whether we are currently fetching env vars
    let mut loading_vars: Signal<bool>         = use_signal(|| false);

    let visible_steps = [
        WizardStep::Overview,
        WizardStep::Configure,
        WizardStep::Confirm,
    ];

    let current = step.read().clone();

    // When entering Configure step for a Container package, fetch env vars once.
    {
        let pkg = package.clone();
        use_effect(move || {
            if *step.read() == WizardStep::Configure
                && pkg.kind == PackageKind::Container
                && detected_vars.read().is_empty()
                && !*loading_vars.read()
            {
                loading_vars.set(true);
                let pkg2 = pkg.clone();
                spawn(async move {
                    let vars = fetch_container_env_vars(&pkg2).await;
                    // Pre-fill textarea with KEY= lines
                    let prefill: String = vars.iter()
                        .map(|k| format!("{k}=\n"))
                        .collect();
                    detected_vars.set(vars);
                    env_vars_text.set(prefill);
                    loading_vars.set(false);
                });
            }
        });
    }

    rsx! {
        div {
            class: "fsd-install-wizard",
            style: "display: flex; flex-direction: column; height: 100%;",

            // Step indicator
            div {
                style: "display: flex; align-items: center; padding: 16px; \
                        border-bottom: 1px solid var(--fsn-color-border-default);",
                for (i, s) in visible_steps.iter().enumerate() {
                    WizardStepDot {
                        key: "{i}",
                        index: i,
                        label: s.label().to_string(),
                        active: current == *s,
                        done: matches!((&current, s),
                            (WizardStep::Confirm, WizardStep::Overview)
                            | (WizardStep::Confirm, WizardStep::Configure)
                            | (WizardStep::Installing, _)
                            | (WizardStep::Done, _)),
                        last: i >= visible_steps.len() - 1,
                    }
                }
            }

            // Step content
            div {
                style: "flex: 1; overflow: auto; padding: 24px;",
                match &current {
                    WizardStep::Overview => rsx! {
                        h3 { style: "margin-top: 0;", "Install {package.name}" }
                        p { style: "color: var(--fsn-color-text-secondary);", "{package.description}" }
                        p { style: "color: var(--fsn-color-text-muted); font-size: 13px;",
                            "Version: {package.version} · Type: {package.kind.label()}"
                        }
                        if !package.tags.is_empty() {
                            div { style: "display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px;",
                                for tag in &package.tags {
                                    span {
                                        key: "{tag}",
                                        style: "font-size: 11px; padding: 2px 8px; border-radius: 999px; \
                                                background: var(--fsn-color-bg-overlay); \
                                                border: 1px solid var(--fsn-color-border-default); \
                                                color: var(--fsn-color-text-muted);",
                                        "{tag}"
                                    }
                                }
                            }
                        }
                    },
                    WizardStep::Configure => rsx! {
                        h3 { style: "margin-top: 0;", "Configure {package.name}" }
                        { match &package.kind {
                            PackageKind::Container => rsx! {
                                ContainerConfigureStep {
                                    loading: *loading_vars.read(),
                                    env_vars_text,
                                }
                            },
                            PackageKind::Language => rsx! {
                                p { style: "color: var(--fsn-color-text-muted);",
                                    "The language pack will be downloaded and saved locally. \
                                     Select it in Settings → Language after installation."
                                }
                            },
                            PackageKind::Theme => rsx! {
                                p { style: "color: var(--fsn-color-text-muted);",
                                    "The theme CSS will be downloaded and saved locally. \
                                     Activate it in Settings → Appearance after installation."
                                }
                            },
                            PackageKind::Widget => rsx! {
                                p { style: "color: var(--fsn-color-text-muted);",
                                    "The widget will be registered. Add it to your desktop \
                                     via Edit Desktop → Add Widget."
                                }
                            },
                            _ => rsx! {
                                p { style: "color: var(--fsn-color-text-muted);",
                                    "No additional configuration required for this package type."
                                }
                            },
                        }}
                    },
                    WizardStep::Confirm => rsx! {
                        h3 { style: "margin-top: 0;", "Ready to install" }
                        p { "Click Install to download and register {package.name} v{package.version}." }
                        div {
                            style: "margin-top: 12px; padding: 12px 16px; \
                                    background: var(--fsn-color-bg-surface); \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); font-size: 13px;",
                            div { "Package: {package.name}" }
                            div { "Version: {package.version}" }
                            div { "Type: {package.kind.label()}" }
                        }
                        if package.kind == PackageKind::Container {
                            div {
                                style: "margin-top: 12px; padding: 10px 14px; \
                                        background: var(--fsn-color-bg-surface); \
                                        border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); font-size: 12px; \
                                        color: var(--fsn-color-text-muted);",
                                p { style: "margin: 0 0 4px 0;",
                                    "The compose file will be saved to:"
                                }
                                code {
                                    style: "font-size: 11px;",
                                    "~/.local/share/fsn/services/{package.id}/"
                                }
                                p { style: "margin: 8px 0 0 0;",
                                    "Then "
                                    code { "fsn conductor install" }
                                    " will generate the Quadlet unit and start the service."
                                }
                            }
                        }
                    },
                    WizardStep::Installing => rsx! {
                        div { style: "text-align: center; padding: 48px;",
                            if let Some(err) = install_error.read().as_deref() {
                                div {
                                    style: "color: var(--fsn-color-error, #ef4444); \
                                            background: rgba(239,68,68,0.1); \
                                            border: 1px solid var(--fsn-color-error, #ef4444); \
                                            border-radius: var(--fsn-radius-md); \
                                            padding: 12px; font-size: 13px; text-align: left;",
                                    p { strong { "Installation failed" } }
                                    p { "{err}" }
                                }
                            } else {
                                p { style: "font-size: 32px; margin-bottom: 12px;", "⏳" }
                                p { "Installing {package.name}…" }
                            }
                        }
                    },
                    WizardStep::Done => rsx! {
                        div { style: "text-align: center; padding: 48px;",
                            p { style: "font-size: 48px; margin-bottom: 12px;", "✓" }
                            p { style: "font-size: 18px; font-weight: 600;",
                                "{package.name} installed"
                            }
                            p { style: "color: var(--fsn-color-text-muted); font-size: 13px; margin-top: 8px;",
                                { match &package.kind {
                                    PackageKind::Container => "Open Conductor to start and configure the service.",
                                    PackageKind::Language  => "Select it in Settings → Language.",
                                    PackageKind::Theme     => "Activate it in Settings → Appearance.",
                                    PackageKind::Widget    => "Add it via Edit Desktop → Add Widget.",
                                    _                      => "Package is ready to use.",
                                }}
                            }
                        }
                    },
                    WizardStep::Error => rsx! {
                        div { style: "text-align: center; padding: 48px; color: var(--fsn-color-error, #ef4444);",
                            p { style: "font-size: 32px;", "✗" }
                            p { "Installation failed." }
                        }
                    },
                }
            }

            // Navigation buttons
            div {
                style: "display: flex; justify-content: space-between; padding: 16px; \
                        border-top: 1px solid var(--fsn-color-border-default);",

                // Left: Cancel / Close
                button {
                    style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); \
                            border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); cursor: pointer;",
                    onclick: move |_| on_cancel.call(()),
                    { if matches!(*step.read(), WizardStep::Done | WizardStep::Error) {
                        "Close"
                    } else {
                        "Cancel"
                    }}
                }

                // Right: Next / Install (hidden when Installing or Done)
                { match &current {
                    WizardStep::Installing => rsx! {
                        span {} // placeholder — buttons hidden while installing
                    },
                    WizardStep::Done | WizardStep::Error => rsx! {
                        span {}
                    },
                    WizardStep::Confirm => rsx! {
                        button {
                            style: "padding: 8px 20px; background: var(--fsn-color-primary); \
                                    color: white; border: none; \
                                    border-radius: var(--fsn-radius-md); cursor: pointer; \
                                    font-weight: 600;",
                            onclick: move |_| {
                                let pkg   = package.clone();
                                let envs  = env_vars_text.read().clone();
                                step.set(WizardStep::Installing);
                                spawn(async move {
                                    match do_install(pkg, envs).await {
                                        Ok(()) => step.set(WizardStep::Done),
                                        Err(e) => {
                                            install_error.set(Some(e));
                                            // Stay on Installing step so error is visible
                                        }
                                    }
                                });
                            },
                            "Install"
                        }
                    },
                    _ => rsx! {
                        button {
                            style: "padding: 8px 20px; background: var(--fsn-color-primary); \
                                    color: white; border: none; \
                                    border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| {
                                let next = step.read().next();
                                if let Some(n) = next {
                                    step.set(n);
                                }
                            },
                            "Next →"
                        }
                    },
                }}
            }
        }
    }
}

// ── ContainerConfigureStep ────────────────────────────────────────────────────

/// Configure step for container packages: editable KEY=VALUE textarea.
/// Pre-filled with detected variable names from the compose file.
#[component]
fn ContainerConfigureStep(loading: bool, mut env_vars_text: Signal<String>) -> Element {
    rsx! {
        div {
            p { style: "color: var(--fsn-color-text-secondary); margin-bottom: 16px;",
                "Enter environment variables for this service (one per line, KEY=VALUE)."
            }
            p { style: "color: var(--fsn-color-text-muted); font-size: 12px; margin-bottom: 12px;",
                "These will be saved to "
                code { ".env" }
                " next to the compose file. Leave values empty to fill in later."
            }

            if loading {
                p { style: "color: var(--fsn-color-text-muted); font-size: 13px;",
                    "Detecting variables…"
                }
            } else {
                textarea {
                    style: "width: 100%; min-height: 200px; padding: 10px 12px; \
                            font-family: monospace; font-size: 13px; \
                            background: var(--fsn-color-bg-elevated); \
                            border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); \
                            color: var(--fsn-color-text-primary); \
                            resize: vertical; box-sizing: border-box;",
                    placeholder: "KEY=value\nANOTHER_KEY=value",
                    value: "{env_vars_text.read()}",
                    oninput: move |e| env_vars_text.set(e.value()),
                }
                p { style: "color: var(--fsn-color-text-muted); font-size: 11px; margin-top: 6px;",
                    "Tip: You can also edit this file later at "
                    code { "~/.local/share/fsn/services/<name>/.env" }
                }
            }
        }
    }
}

// ── WizardStepDot ─────────────────────────────────────────────────────────────

#[component]
fn WizardStepDot(
    index: usize,
    label: String,
    active: bool,
    done: bool,
    last: bool,
) -> Element {
    let bg    = if active { "var(--fsn-color-primary)" }
                else if done { "var(--fsn-color-success, #22c55e)" }
                else { "var(--fsn-color-bg-overlay)" };
    let color = if active || done { "white" } else { "var(--fsn-color-text-muted)" };
    let text  = if active { "var(--fsn-color-text-primary)" } else { "var(--fsn-color-text-muted)" };
    let num   = index + 1;
    let inner = if done { "✓".to_string() } else { num.to_string() };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 4px;",
            div {
                style: "width: 24px; height: 24px; border-radius: 50%; \
                        display: flex; align-items: center; justify-content: center; \
                        font-size: 12px; background: {bg}; color: {color};",
                "{inner}"
            }
            span {
                style: "font-size: 13px; color: {text};",
                "{label}"
            }
            if !last {
                span { style: "margin: 0 8px; color: var(--fsn-color-text-muted);", "›" }
            }
        }
    }
}
