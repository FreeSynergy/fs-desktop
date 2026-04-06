/// Service Roles — extended MIME-like system for functions, not files.
///
/// Each role defines a function the system needs (e.g. auth, mail).
/// A container registers which roles it can fulfill.
/// Settings assigns exactly one container per role.
use std::collections::HashMap;
use std::path::PathBuf;

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
    (
        "proxy",
        "Reverse Proxy",
        "HTTP reverse proxy with TLS termination (Zentinel)",
        true,
    ),
    (
        "auth",
        "Authentication",
        "Handles user login and identity (OIDC/SCIM)",
        true,
    ),
    ("mail", "Mail", "Sends and receives email", true),
    ("git", "Git", "Hosts Git repositories", false),
    ("wiki", "Wiki", "Documentation and knowledge base", false),
    ("chat", "Chat", "Real-time messaging", false),
    ("tasks", "Tasks", "Task and project management", false),
    ("tickets", "Tickets", "Issue tracking and helpdesk", false),
    (
        "collab",
        "Collaboration",
        "Real-time document editing",
        false,
    ),
    ("maps", "Maps", "Mapping and geospatial data", false),
    (
        "monitoring",
        "Monitoring",
        "Metrics, logs, and alerting",
        false,
    ),
    ("database", "Database", "Primary relational database", false),
    ("cache", "Cache", "Key-value cache", false),
    (
        "llm",
        "AI / LLM",
        "Local AI language model for text generation",
        false,
    ),
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
#[must_use]
pub fn load_role_assignments() -> ServiceRoleConfig {
    let path = crate::config_path("settings.toml");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let parsed: PartialSettings = toml::from_str(&content).unwrap_or_default();
    ServiceRoleConfig {
        assignments: parsed.service_roles,
    }
}

/// Persist service role assignments into `~/.config/fsn/settings.toml`.
///
/// Only updates the `[service_roles]` section; all other keys are preserved
/// by a round-trip through a generic TOML value.
///
/// # Errors
///
/// Returns an error if the config directory cannot be created or the file cannot be written.
pub fn save_role_assignments(config: &ServiceRoleConfig) -> Result<(), String> {
    let path = crate::config_path("settings.toml");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Load existing TOML to preserve all other settings.
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: toml::Value =
        toml::from_str(&existing).unwrap_or(toml::Value::Table(toml::map::Map::default()));

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
    ///   1. `FS_PLUGINS_DIR` env var
    ///   2. First enabled store `local_path` in settings
    ///   3. `~/.local/share/fsn/modules`
    #[must_use]
    pub fn build() -> Self {
        let dir = modules_dir();
        Self::build_from_dir(&dir)
    }

    /// Build the registry by walking `modules_dir` recursively.
    #[must_use]
    pub fn build_from_dir(modules_dir: &std::path::Path) -> Self {
        let mut providers: HashMap<String, Vec<String>> = HashMap::new();

        if !modules_dir.exists() {
            return Self { providers };
        }

        for path in walkdir_toml(modules_dir) {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(parsed) = toml::from_str::<MinimalModuleFile>(&content) else {
                continue;
            };
            for role in parsed.meta.roles.provides {
                providers
                    .entry(role)
                    .or_default()
                    .push(parsed.meta.name.clone());
            }
        }

        Self { providers }
    }

    /// All module names that claim to provide `role_id`.
    pub fn providers_for(&self, role_id: &str) -> &[String] {
        self.providers.get(role_id).map_or(&[], Vec::as_slice)
    }
}

fn modules_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FS_PLUGINS_DIR") {
        return PathBuf::from(dir);
    }
    // Check settings for local store path.
    let settings_content =
        std::fs::read_to_string(crate::config_path("settings.toml")).unwrap_or_default();
    if let Ok(v) = toml::from_str::<toml::Value>(&settings_content) {
        if let Some(stores) = v.get("stores").and_then(|s| s.as_array()) {
            for store in stores {
                let enabled = store
                    .get("enabled")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(true);
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
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("fsn")
        .join("modules")
}

fn walkdir_toml(dir: &std::path::Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            result.extend(walkdir_toml(&path));
        } else if path.extension().is_some_and(|ext| ext == "toml") {
            result.push(path);
        }
    }
    result
}

// ── ServiceRoles (public re-export type) ──────────────────────────────────────

/// Public type alias kept for backwards compatibility with `lib.rs` re-exports.
pub struct ServiceRoles;

// ── ServiceRolesState ─────────────────────────────────────────────────────────

/// Runtime state for the Service Roles settings section.
#[derive(Debug, Clone)]
pub struct ServiceRolesState {
    pub config: ServiceRoleConfig,
    pub registry: ServiceRoleRegistry,
}

impl ServiceRolesState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: load_role_assignments(),
            registry: ServiceRoleRegistry::build(),
        }
    }
}

impl Default for ServiceRolesState {
    fn default() -> Self {
        Self::new()
    }
}

// ── view_service_roles ────────────────────────────────────────────────────────

use fs_gui_engine_iced::iced::{
    widget::{button, column, row, scrollable, text},
    Alignment, Element, Length,
};
use fs_i18n;

use crate::app::{Message, SettingsApp};

/// Render the Service Roles settings section.
#[must_use]
pub fn view_service_roles(app: &SettingsApp) -> Element<'_, Message> {
    let state = &app.service_roles;

    let rows: Vec<Element<Message>> = KNOWN_ROLES
        .iter()
        .map(|(id, name, _desc, required)| {
            let current = state.config.get(id).unwrap_or("(none)");
            let providers = state.registry.providers_for(id);

            let req_label = if *required { " *" } else { "" };
            let role_id = (*id).to_string();

            let provider_btns: Vec<Element<Message>> = providers
                .iter()
                .map(|p| {
                    let is_active = current == p.as_str();
                    let rid = role_id.clone();
                    let pname = p.clone();
                    button(text(p.as_str()).size(11))
                        .padding([4, 8])
                        .style(if is_active {
                            fs_gui_engine_iced::iced::widget::button::primary
                        } else {
                            fs_gui_engine_iced::iced::widget::button::secondary
                        })
                        .on_press(Message::ServiceRoleChanged(rid, pname))
                        .into()
                })
                .collect();

            let providers_row: Element<Message> = if provider_btns.is_empty() {
                text(fs_i18n::t("settings-roles-no-providers").to_string())
                    .size(11)
                    .into()
            } else {
                row(provider_btns).spacing(4).into()
            };

            column![
                row![
                    text(format!("{name}{req_label}"))
                        .size(13)
                        .width(Length::Fill),
                    text(format!("= {current}")).size(12),
                ]
                .align_y(Alignment::Center)
                .spacing(8),
                providers_row,
            ]
            .spacing(4)
            .padding([8, 0])
            .into()
        })
        .collect();

    let save_btn = button(text(fs_i18n::t("actions.save").to_string()).size(13))
        .padding([8, 20])
        .on_press(Message::SaveServiceRoles);

    let content = column![
        text(fs_i18n::t("settings-section-roles").to_string()).size(16),
        column(rows).spacing(4),
        save_btn,
    ]
    .spacing(16)
    .width(Length::Fill);

    scrollable(content).into()
}
