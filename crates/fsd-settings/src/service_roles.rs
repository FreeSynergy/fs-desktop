/// Service Roles — extended MIME-like system for functions, not files.
///
/// Each role defines a function the system needs (e.g. auth, mail).
/// A container registers which roles it can fulfill.
/// Settings assigns exactly one container per role.
use std::collections::HashMap;
use std::path::PathBuf;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// ── Data types ────────────────────────────────────────────────────────────────

/// A known service role — like a MIME type but for system functions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceRole {
    /// Unique identifier (e.g. "auth", "mail").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what this role provides.
    pub description: String,
    /// Whether this role is required for the system to function.
    pub required: bool,
}

/// All known service roles in the FSN ecosystem.
pub const KNOWN_ROLES: &[(&str, &str, &str, bool)] = &[
    ("proxy",      "Reverse Proxy",   "HTTP reverse proxy with TLS termination (Zentinel)",  true),
    ("auth",       "Authentication",  "Handles user login and identity (OIDC/SCIM)",         true),
    ("mail",       "Mail",            "Sends and receives email",                             true),
    ("git",        "Git",             "Hosts Git repositories",                               false),
    ("wiki",       "Wiki",            "Documentation and knowledge base",                     false),
    ("chat",       "Chat",            "Real-time messaging",                                  false),
    ("tasks",      "Tasks",           "Task and project management",                          false),
    ("tickets",    "Tickets",         "Issue tracking and helpdesk",                          false),
    ("collab",     "Collaboration",   "Real-time document editing",                           false),
    ("maps",       "Maps",            "Mapping and geospatial data",                          false),
    ("monitoring", "Monitoring",      "Metrics, logs, and alerting",                          false),
    ("database",   "Database",        "Primary relational database",                          false),
    ("cache",      "Cache",           "Key-value cache",                                      false),
];

/// The currently configured service role assignments.
/// Maps role ID → container/service name.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServiceRoleConfig {
    pub assignments: HashMap<String, String>,
}

impl ServiceRoleConfig {
    pub fn get(&self, role: &str) -> Option<&str> {
        self.assignments.get(role).map(String::as_str)
    }

    pub fn set(&mut self, role: impl Into<String>, service: impl Into<String>) {
        self.assignments.insert(role.into(), service.into());
    }

    pub fn clear(&mut self, role: &str) {
        self.assignments.remove(role);
    }
}

// ── Persistence ───────────────────────────────────────────────────────────────

/// Minimal struct matching the `service_roles` section in `~/.config/fsn/settings.toml`.
#[derive(Deserialize, Default)]
struct PartialSettings {
    #[serde(default)]
    service_roles: HashMap<String, String>,
}

/// Load service role assignments from `~/.config/fsn/settings.toml`.
pub fn load_role_assignments() -> ServiceRoleConfig {
    let path = crate::config_path("settings.toml");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let parsed: PartialSettings = toml::from_str(&content).unwrap_or_default();
    ServiceRoleConfig { assignments: parsed.service_roles }
}

/// Persist service role assignments into `~/.config/fsn/settings.toml`.
///
/// Only updates the `[service_roles]` section; all other keys are preserved
/// by a round-trip through a generic TOML value.
pub fn save_role_assignments(config: &ServiceRoleConfig) -> Result<(), String> {
    let path = crate::config_path("settings.toml");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Load existing TOML to preserve all other settings.
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: toml::Value = toml::from_str(&existing).unwrap_or(toml::Value::Table(Default::default()));

    if let toml::Value::Table(ref mut root) = doc {
        let mut role_table = toml::value::Table::new();
        for (k, v) in &config.assignments {
            role_table.insert(k.clone(), toml::Value::String(v.clone()));
        }
        root.insert("service_roles".to_string(), toml::Value::Table(role_table));
    }

    let out = toml::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&path, out).map_err(|e| e.to_string())
}

// ── ServiceRoleRegistry ───────────────────────────────────────────────────────

/// Scans all module TOML files and builds a map of role → providers.
#[derive(Debug, Default, Clone)]
pub struct ServiceRoleRegistry {
    /// Maps role ID → list of module names that provide it.
    pub providers: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct MinimalModuleFile {
    #[serde(rename = "module")]
    meta: MinimalModuleMeta,
}

#[derive(Deserialize)]
struct MinimalModuleMeta {
    name: String,
    #[serde(default)]
    roles: MinimalRoles,
}

#[derive(Deserialize, Default)]
struct MinimalRoles {
    #[serde(default)]
    provides: Vec<String>,
}

impl ServiceRoleRegistry {
    /// Build the registry from the standard FSN modules directory.
    ///
    /// Checks (in order):
    ///   1. `FSN_PLUGINS_DIR` env var
    ///   2. First enabled store `local_path` in settings
    ///   3. `~/.local/share/fsn/modules`
    pub fn build() -> Self {
        let dir = modules_dir();
        Self::build_from_dir(&dir)
    }

    /// Build the registry by walking `modules_dir` recursively.
    pub fn build_from_dir(modules_dir: &std::path::Path) -> Self {
        let mut providers: HashMap<String, Vec<String>> = HashMap::new();

        if !modules_dir.exists() {
            return Self { providers };
        }

        for path in walkdir_toml(modules_dir) {
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let Ok(parsed) = toml::from_str::<MinimalModuleFile>(&content) else { continue };
            for role in parsed.meta.roles.provides {
                providers.entry(role).or_default().push(parsed.meta.name.clone());
            }
        }

        Self { providers }
    }

    /// All module names that claim to provide `role_id`.
    pub fn providers_for(&self, role_id: &str) -> &[String] {
        self.providers.get(role_id).map(Vec::as_slice).unwrap_or(&[])
    }
}

fn modules_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FSN_PLUGINS_DIR") {
        return PathBuf::from(dir);
    }
    // Check settings for local store path.
    let settings_content = std::fs::read_to_string(crate::config_path("settings.toml")).unwrap_or_default();
    if let Ok(v) = toml::from_str::<toml::Value>(&settings_content) {
        if let Some(stores) = v.get("stores").and_then(|s| s.as_array()) {
            for store in stores {
                let enabled = store.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true);
                if enabled {
                    if let Some(local) = store.get("local_path").and_then(|p| p.as_str()) {
                        return PathBuf::from(local).join("Node").join("modules");
                    }
                }
            }
        }
    }
    // XDG fallback.
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local").join("share").join("fsn").join("modules")
}

fn walkdir_toml(dir: &std::path::Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut result = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            result.extend(walkdir_toml(&path));
        } else if path.extension().map_or(false, |ext| ext == "toml") {
            result.push(path);
        }
    }
    result
}

// ── Service Roles component ───────────────────────────────────────────────────

/// Service Roles settings component.
#[component]
pub fn ServiceRoles() -> Element {
    let config   = use_signal(load_role_assignments);
    let registry     = use_signal(ServiceRoleRegistry::build);
    let mut save_msg = use_signal(|| Option::<String>::None);

    rsx! {
        div {
            class: "fsd-service-roles",
            style: "padding: 24px;",

            h3 { style: "margin-top: 0;", "Service Roles" }
            p { style: "color: var(--fsn-color-text-muted); margin-bottom: 24px;",
                "Assign which installed service handles each system function. \
                 Similar to MIME types — but for capabilities."
            }

            div {
                style: "display: flex; flex-direction: column; gap: 12px;",

                for (id, name, description, required) in KNOWN_ROLES {
                    {
                        let providers = registry.read().providers_for(id).to_vec();
                        rsx! {
                            RoleRow {
                                key: "{id}",
                                role_id: id.to_string(),
                                name: name.to_string(),
                                description: description.to_string(),
                                required: *required,
                                assigned: config.read().get(id).unwrap_or("").to_string(),
                                providers,
                                config,
                            }
                        }
                    }
                }
            }

            div { style: "margin-top: 24px; display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        match save_role_assignments(&config.read()) {
                            Ok(()) => *save_msg.write() = Some("Saved.".into()),
                            Err(e) => *save_msg.write() = Some(format!("Error: {e}")),
                        }
                    },
                    "Save"
                }
                if let Some(msg) = save_msg.read().as_deref() {
                    span { style: "font-size: 13px; color: var(--fsn-color-text-muted);", "{msg}" }
                }
            }
        }
    }
}

/// A single role assignment row.
#[component]
fn RoleRow(
    role_id: String,
    name: String,
    description: String,
    required: bool,
    assigned: String,
    providers: Vec<String>,
    mut config: Signal<ServiceRoleConfig>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 16px; padding: 12px 16px; \
                    background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); \
                    border: 1px solid var(--fsn-color-border-default);",

            div { style: "flex: 1;",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    strong { "{name}" }
                    if required {
                        span {
                            style: "font-size: 11px; background: var(--fsn-color-error); color: white; \
                                    padding: 1px 6px; border-radius: 4px;",
                            "required"
                        }
                    }
                }
                div { style: "font-size: 12px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                    "{description}"
                }
            }

            select {
                style: "padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                        border-radius: var(--fsn-radius-md); font-size: 13px; min-width: 180px;",
                value: "{assigned}",
                onchange: {
                    let role_id = role_id.clone();
                    move |e: Event<FormData>| {
                        let val = e.value();
                        if val.is_empty() {
                            config.write().clear(&role_id);
                        } else {
                            config.write().set(role_id.clone(), val);
                        }
                    }
                },
                option { value: "", "— not assigned —" }
                for provider in &providers {
                    option { value: "{provider}", selected: *provider == assigned, "{provider}" }
                }
            }
        }
    }
}
