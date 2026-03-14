/// Service Roles — extended MIME-like system for functions, not files.
///
/// Each role defines a function the system needs (e.g. auth, mail).
/// A container registers which roles it can fulfill.
/// Settings assigns exactly one container per role.
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    ("auth",       "Authentication",  "Handles user login and identity (OIDC/SCIM)", true),
    ("mail",       "Mail",            "Sends and receives email",                    true),
    ("git",        "Git",             "Hosts Git repositories",                      false),
    ("wiki",       "Wiki",            "Documentation and knowledge base",            false),
    ("chat",       "Chat",            "Real-time messaging",                         false),
    ("storage",    "Storage",         "File storage and sharing",                    false),
    ("tasks",      "Tasks",           "Task and project management",                 false),
    ("calendar",   "Calendar",        "Shared calendar",                             false),
    ("monitoring", "Monitoring",      "Metrics, logs, and alerting",                 false),
    ("collab",     "Collaboration",   "Real-time document editing",                  false),
    ("tickets",    "Tickets",         "Issue tracking and helpdesk",                 false),
    ("maps",       "Maps",            "Mapping and geospatial data",                 false),
    ("database",   "Database",        "Primary relational database",                 false),
    ("cache",      "Cache",           "Key-value cache",                             false),
    ("proxy",      "Reverse Proxy",   "HTTP reverse proxy (Zentinel)",               true),
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

/// Service Roles settings component.
#[component]
pub fn ServiceRoles() -> Element {
    let config = use_signal(ServiceRoleConfig::default);
    // TODO: load available services from fsn-container

    rsx! {
        div {
            class: "fsd-service-roles",
            style: "padding: 24px;",

            h3 { style: "margin-top: 0;", "Service Roles" }
            p { style: "color: var(--fsn-color-text-muted); margin-bottom: 24px;",
                "Assign which installed service handles each system function. Similar to MIME types — but for capabilities."
            }

            div {
                style: "display: flex; flex-direction: column; gap: 12px;",

                for (id, name, description, required) in KNOWN_ROLES {
                    RoleRow {
                        key: "{id}",
                        role_id: id.to_string(),
                        name: name.to_string(),
                        description: description.to_string(),
                        required: *required,
                        assigned: config.read().get(id).unwrap_or("").to_string(),
                        config,
                    }
                }
            }

            div { style: "margin-top: 24px;",
                button {
                    style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    "Save"
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
    mut config: Signal<ServiceRoleConfig>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 16px; padding: 12px 16px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); border: 1px solid var(--fsn-color-border-default);",

            // Role info
            div { style: "flex: 1;",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    strong { "{name}" }
                    if required {
                        span {
                            style: "font-size: 11px; background: var(--fsn-color-error); color: white; padding: 1px 6px; border-radius: 4px;",
                            "required"
                        }
                    }
                }
                div { style: "font-size: 12px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                    "{description}"
                }
            }

            // Service selector
            select {
                style: "padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px; min-width: 180px;",
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
                // TODO: populate from installed services that provide this role
            }
        }
    }
}
