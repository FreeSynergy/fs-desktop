/// Module builder — converts Docker Compose / YAML into an FSN module definition.
use std::collections::HashMap;

use dioxus::prelude::*;
use serde::Deserialize;

use crate::ollama::{ModuleMetadataSuggestion, OllamaClient, OLLAMA_BASE_URL};

// ── Compose parsing ───────────────────────────────────────────────────────────

/// Minimal Docker Compose file shape for parsing.
#[derive(Deserialize)]
struct ComposeFile {
    services: HashMap<String, ComposeServiceDef>,
}

#[derive(Deserialize)]
struct ComposeServiceDef {
    image: Option<String>,
    #[serde(default)]
    ports: Vec<serde_json::Value>,
    #[serde(default)]
    volumes: Vec<serde_json::Value>,
    environment: Option<serde_json::Value>,
    healthcheck: Option<serde_json::Value>,
}

/// Normalised view of one parsed service.
#[derive(Clone, Debug, Default)]
pub struct ParsedService {
    pub name: String,
    pub image: String,
    pub image_tag: String,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub env: Vec<String>,
    pub has_healthcheck: bool,
}

/// Parse a Docker Compose YAML and return all services.
pub fn parse_compose(yaml: &str) -> Result<Vec<ParsedService>, String> {
    let compose: ComposeFile = serde_yaml::from_str(yaml)
        .map_err(|e| format!("YAML parse error: {e}"))?;

    let mut result = Vec::new();
    for (name, def) in compose.services {
        let (image, tag) = split_image(def.image.as_deref().unwrap_or(""));
        let ports = def.ports.iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        let volumes = def.volumes.iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        let env = match def.environment {
            Some(serde_json::Value::Object(m)) => m.keys().cloned().collect(),
            Some(serde_json::Value::Array(a)) => a.iter()
                .filter_map(|v| v.as_str().and_then(|s| s.split_once('=').map(|(k, _)| k.to_owned())))
                .collect(),
            _ => Vec::new(),
        };
        result.push(ParsedService {
            name,
            image,
            image_tag: tag,
            ports,
            volumes,
            env,
            has_healthcheck: def.healthcheck.is_some(),
        });
    }
    Ok(result)
}

fn split_image(image: &str) -> (String, String) {
    if let Some((img, tag)) = image.rsplit_once(':') {
        (img.to_owned(), tag.to_owned())
    } else {
        (image.to_owned(), "latest".to_owned())
    }
}

/// Detect service type from image name and port hints.
pub fn detect_service_type(service: &ParsedService) -> &'static str {
    let img = service.image.to_lowercase();
    let has_port = |p: u16| service.ports.iter().any(|s| s.contains(&p.to_string()));

    if img.contains("nginx") || img.contains("caddy") || img.contains("traefik") || img.contains("zentinel") {
        return "proxy";
    }
    if img.contains("kanidm") || img.contains("keycloak") || img.contains("authentik") || img.contains("dex") {
        return "iam";
    }
    if img.contains("stalwart") || img.contains("postfix") || img.contains("dovecot") || img.contains("maddy") {
        return "mail";
    }
    if img.contains("forgejo") || img.contains("gitea") || img.contains("gitlab") || img.contains("gogs") {
        return "git";
    }
    if img.contains("outline") || img.contains("bookstack") || img.contains("wiki") {
        return "wiki";
    }
    if img.contains("element") || img.contains("synapse") || img.contains("tuwunel") || img.contains("matrix") {
        return "chat";
    }
    if img.contains("cryptpad") || img.contains("nextcloud") || img.contains("onlyoffice") {
        return "collab";
    }
    if img.contains("vikunja") || img.contains("todoist") || img.contains("plane") {
        return "tasks";
    }
    if img.contains("pretix") || img.contains("ticketing") {
        return "tickets";
    }
    if img.contains("umap") || img.contains("osm") || img.contains("tile") {
        return "maps";
    }
    if img.contains("openobserve") || img.contains("grafana") || img.contains("prometheus") {
        return "monitoring";
    }
    if img.contains("postgres") || img.contains("mysql") || img.contains("mariadb") {
        return "database";
    }
    if img.contains("redis") || img.contains("dragonfly") || img.contains("memcached") {
        return "cache";
    }
    if has_port(80) || has_port(443) || has_port(8080) || has_port(3000) {
        return "custom";
    }
    "custom"
}

/// Detect a likely health check path from image name / known patterns.
pub fn detect_health_path(service: &ParsedService) -> &'static str {
    let img = service.image.to_lowercase();
    if img.contains("kanidm") { return "/status"; }
    if img.contains("forgejo") || img.contains("gitea") { return "/-/health"; }
    if img.contains("outline") { return "/_health"; }
    if img.contains("vikunja") { return "/health"; }
    if img.contains("pretix") { return "/healthcheck/"; }
    if img.contains("openobserve") { return "/healthz"; }
    "/health"
}

/// Generate FSN module TOML from a parsed service and optional AI metadata.
pub fn generate_module_toml(
    service: &ParsedService,
    service_type: &str,
    ai: Option<&ModuleMetadataSuggestion>,
) -> String {
    let name = ai.map(|a| a.name.clone()).unwrap_or_else(|| service.name.replace('-', "_"));
    let description = ai.map(|a| a.description.clone())
        .unwrap_or_else(|| format!("{} container", service.image));
    let health_path = ai.map(|a| a.health_path.clone())
        .unwrap_or_else(|| detect_health_path(service).to_owned());
    let tags_str = ai.map(|a| {
        a.tags.iter().map(|t| format!("{t:?}")).collect::<Vec<_>>().join(", ")
    }).unwrap_or_default();

    // Detect primary port from ports list (container side).
    let port = service.ports.first()
        .and_then(|p| p.split(':').last())
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let mut out = String::new();
    out.push_str("[module]\n");
    out.push_str(&format!("name        = {:?}\n", name));
    out.push_str(&format!("type        = {:?}\n", service_type));
    out.push_str(&format!("version     = \"0.1.0\"\n"));
    out.push_str(&format!("description = {:?}\n", description));
    if !tags_str.is_empty() {
        out.push_str(&format!("tags        = [{tags_str}]\n"));
    }
    out.push_str(&format!("port        = {port}\n"));
    if !health_path.is_empty() {
        out.push_str(&format!("health_path = {:?}\n", health_path));
    }
    out.push('\n');

    out.push_str("[container]\n");
    out.push_str(&format!("name      = {:?}\n", name));
    out.push_str(&format!("image     = {:?}\n", service.image));
    out.push_str(&format!("image_tag = {:?}\n", service.image_tag));

    if !service.volumes.is_empty() {
        out.push('\n');
        out.push_str("volumes = [\n");
        for v in &service.volumes {
            out.push_str(&format!("  {:?},\n", v));
        }
        out.push_str("]\n");
    }

    out.push('\n');
    out.push_str("[container.healthcheck]\n");
    out.push_str(&format!("cmd          = \"curl -f http://localhost:{port}{health_path} || exit 1\"\n"));
    out.push_str("interval     = \"30s\"\n");
    out.push_str("timeout      = \"10s\"\n");
    out.push_str("retries      = 3\n");
    out.push_str("start_period = \"60s\"\n");

    if !service.env.is_empty() {
        out.push('\n');
        out.push_str("[environment]\n");
        for key in &service.env {
            out.push_str(&format!("# {key} = \"\"\n"));
        }
    }

    out
}

// ── Builder wizard state ──────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Default)]
pub enum BuilderStep {
    #[default]
    Input,
    Review,
    Export,
}

/// Module builder component.
#[component]
pub fn ModuleBuilder() -> Element {
    let mut step            = use_signal(BuilderStep::default);
    let mut yaml_input      = use_signal(String::new);
    let mut parse_error     = use_signal(|| Option::<String>::None);
    let mut parsed          = use_signal(|| Vec::<ParsedService>::new());
    let mut selected_idx    = use_signal(|| 0usize);
    let mut generated_toml  = use_signal(String::new);
    let mut ai_status       = use_signal(|| Option::<String>::None);
    let mut ai_suggestion   = use_signal(|| Option::<ModuleMetadataSuggestion>::None);
    let mut ai_prompt       = use_signal(String::new);
    let mut export_path     = use_signal(String::new);
    let mut export_msg      = use_signal(|| Option::<String>::None);

    rsx! {
        div {
            class: "fsd-module-builder",
            style: "padding: 24px;",

            match *step.read() {
                // ── Step 1: Input ─────────────────────────────────────────────
                BuilderStep::Input => rsx! {
                    h3 { "Module Builder" }
                    p { style: "color: var(--fsn-color-text-muted); margin-bottom: 16px;",
                        "Paste a Docker Compose file or container YAML. \
                         The builder converts it to an FSN module definition."
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "YAML / Compose Input" }
                        textarea {
                            style: "width: 100%; height: 280px; padding: 8px 12px; \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); font-family: var(--fsn-font-mono); \
                                    font-size: 13px; resize: vertical;",
                            placeholder: "version: '3'\nservices:\n  myapp:\n    image: myapp:latest\n    ports:\n      - '8080:8080'",
                            oninput: move |e| {
                                *yaml_input.write() = e.value();
                                *parse_error.write() = None;
                            },
                            "{yaml_input.read()}"
                        }
                    }

                    if let Some(err) = parse_error.read().as_deref() {
                        div { style: "color: var(--fsn-color-error); font-size: 13px; margin-bottom: 8px;", "{err}" }
                    }

                    button {
                        style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; \
                                border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                        disabled: yaml_input.read().is_empty(),
                        onclick: move |_| {
                            match parse_compose(&yaml_input.read()) {
                                Ok(services) if !services.is_empty() => {
                                    // Pre-generate TOML for first service.
                                    let svc = &services[0];
                                    let stype = detect_service_type(svc);
                                    *generated_toml.write() = generate_module_toml(svc, stype, None);
                                    *parsed.write() = services;
                                    *selected_idx.write() = 0;
                                    *step.write() = BuilderStep::Review;
                                }
                                Ok(_) => *parse_error.write() = Some("No services found in YAML.".into()),
                                Err(e) => *parse_error.write() = Some(e),
                            }
                        },
                        "Analyse →"
                    }
                },

                // ── Step 2: Review ────────────────────────────────────────────
                BuilderStep::Review => rsx! {
                    h3 { "Review Generated Module" }

                    // Service selector (if multiple services)
                    if parsed.read().len() > 1 {
                        div { style: "margin-bottom: 12px;",
                            label { style: "font-weight: 500; margin-right: 8px;", "Service:" }
                            select {
                                style: "padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                                onchange: move |e: Event<FormData>| {
                                    let idx: usize = e.value().parse().unwrap_or(0);
                                    *selected_idx.write() = idx;
                                    let svc = &parsed.read()[idx];
                                    let stype = detect_service_type(svc);
                                    *generated_toml.write() = generate_module_toml(svc, stype, ai_suggestion.read().as_ref());
                                },
                                for (i, svc) in parsed.read().iter().enumerate() {
                                    option { value: "{i}", "{svc.name}" }
                                }
                            }
                        }
                    }

                    // Generated TOML (editable)
                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;",
                            "Generated module.toml"
                        }
                        textarea {
                            style: "width: 100%; height: 360px; padding: 8px 12px; \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); font-family: var(--fsn-font-mono); \
                                    font-size: 12px; resize: vertical;",
                            oninput: move |e| *generated_toml.write() = e.value(),
                            "{generated_toml.read()}"
                        }
                    }

                    // AI Assist section
                    div {
                        style: "margin-bottom: 16px; padding: 12px 16px; \
                                background: var(--fsn-color-bg-surface); \
                                border-radius: var(--fsn-radius-md); \
                                border: 1px solid var(--fsn-color-border-default);",

                        div { style: "font-weight: 500; margin-bottom: 8px;", "AI Assist (Ollama)" }
                        p { style: "font-size: 12px; color: var(--fsn-color-text-muted); margin-bottom: 8px;",
                            "Describe the service in plain language and let Ollama enrich the metadata."
                        }
                        div { style: "display: flex; gap: 8px; margin-bottom: 8px;",
                            input {
                                r#type: "text",
                                placeholder: "e.g. Self-hosted Git server with CI and code review",
                                value: "{ai_prompt.read()}",
                                style: "flex: 1; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| *ai_prompt.write() = e.value(),
                            }
                            button {
                                style: "padding: 6px 16px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px; white-space: nowrap;",
                                disabled: ai_prompt.read().is_empty(),
                                onclick: move |_| {
                                    let prompt = ai_prompt.read().clone();
                                    let idx = *selected_idx.read();
                                    *ai_status.write() = Some("Asking Ollama…".into());
                                    spawn(async move {
                                        let client = OllamaClient::new(OLLAMA_BASE_URL, "llama3.2");
                                        if !client.is_available().await {
                                            *ai_status.write() = Some("Ollama not reachable at localhost:11434".into());
                                            return;
                                        }
                                        match client.suggest_metadata(&prompt).await {
                                            Ok(suggestion) => {
                                                let svc_snapshot = parsed.read()[idx].clone();
                                                let stype = suggestion.service_type.clone();
                                                *generated_toml.write() = generate_module_toml(
                                                    &svc_snapshot, &stype, Some(&suggestion)
                                                );
                                                *ai_suggestion.write() = Some(suggestion);
                                                *ai_status.write() = Some("Metadata enriched by AI.".into());
                                            }
                                            Err(e) => *ai_status.write() = Some(format!("AI error: {e}")),
                                        }
                                    });
                                },
                                "Ask AI"
                            }
                        }
                        if let Some(msg) = ai_status.read().as_deref() {
                            div { style: "font-size: 12px; color: var(--fsn-color-text-muted);", "{msg}" }
                        }
                    }

                    div { style: "display: flex; gap: 8px;",
                        button {
                            style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| *step.write() = BuilderStep::Input,
                            "← Back"
                        }
                        button {
                            style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; \
                                    border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| {
                                let svc = &parsed.read()[*selected_idx.read()];
                                let default_path = format!(
                                    "{}/modules/{}/{}/{}.toml",
                                    std::env::var("HOME").unwrap_or_else(|_| ".".into()),
                                    detect_service_type(svc), svc.name, svc.name
                                );
                                *export_path.write() = default_path;
                                *step.write() = BuilderStep::Export;
                            },
                            "Export →"
                        }
                    }
                },

                // ── Step 3: Export ────────────────────────────────────────────
                BuilderStep::Export => rsx! {
                    h3 { "Export Module" }
                    p { style: "color: var(--fsn-color-text-muted); margin-bottom: 16px;",
                        "Save the module to your local modules directory."
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Save to path" }
                        input {
                            r#type: "text",
                            value: "{export_path.read()}",
                            style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-family: var(--fsn-font-mono); font-size: 13px;",
                            oninput: move |e| *export_path.write() = e.value(),
                        }
                    }

                    if let Some(msg) = export_msg.read().as_deref() {
                        div { style: "font-size: 13px; color: var(--fsn-color-text-muted); margin-bottom: 12px;", "{msg}" }
                    }

                    div { style: "display: flex; gap: 8px;",
                        button {
                            style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| *step.write() = BuilderStep::Review,
                            "← Back"
                        }
                        button {
                            style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; \
                                    border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| {
                                let path = std::path::PathBuf::from(export_path.read().clone());
                                let toml = generated_toml.read().clone();
                                if let Some(parent) = path.parent() {
                                    if let Err(e) = std::fs::create_dir_all(parent) {
                                        *export_msg.write() = Some(format!("Error creating dirs: {e}"));
                                        return;
                                    }
                                }
                                match std::fs::write(&path, &toml) {
                                    Ok(()) => *export_msg.write() = Some(format!("Saved to {}", path.display())),
                                    Err(e) => *export_msg.write() = Some(format!("Error: {e}")),
                                }
                            },
                            "Save"
                        }
                    }
                },
            }
        }
    }
}
