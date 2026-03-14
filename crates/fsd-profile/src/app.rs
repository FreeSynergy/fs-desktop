/// Profile — user profile, avatar, SSH keys, and display preferences.
use std::path::PathBuf;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// User profile data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserProfile {
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub ssh_keys: Vec<SshKey>,
    pub timezone: String,
}

/// An SSH public key entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SshKey {
    pub label: String,
    pub public_key: String,
    pub added_at: String,
}

impl UserProfile {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("profile.toml")
    }

    /// Load profile from `~/.config/fsn/profile.toml`. Returns default if absent.
    pub fn load() -> Self {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    /// Save profile to `~/.config/fsn/profile.toml`.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

/// Profile app component.
#[component]
pub fn ProfileApp() -> Element {
    let mut profile  = use_signal(UserProfile::load);
    let mut save_msg = use_signal(|| Option::<String>::None);
    let mut new_key_label  = use_signal(String::new);
    let mut new_key_value  = use_signal(String::new);
    let mut show_add_key   = use_signal(|| false);

    rsx! {
        div {
            class: "fsd-profile",
            style: "padding: 24px; max-width: 600px;",

            h2 { style: "margin-top: 0;", "Profile" }

            // Avatar
            div { style: "display: flex; align-items: center; gap: 24px; margin-bottom: 32px;",
                div {
                    style: "width: 80px; height: 80px; border-radius: 50%; background: var(--fsn-color-bg-overlay); \
                            display: flex; align-items: center; justify-content: center; font-size: 32px; \
                            border: 2px solid var(--fsn-color-border-default);",
                    if let Some(url) = profile.read().avatar_url.clone() {
                        img { src: "{url}", width: "80", height: "80", style: "border-radius: 50%;" }
                    } else {
                        "👤"
                    }
                }
                div {
                    button {
                        style: "display: block; padding: 6px 12px; background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); \
                                cursor: pointer; margin-bottom: 8px;",
                        "Upload photo"
                    }
                    button {
                        style: "display: block; padding: 6px 12px; background: none; border: none; \
                                cursor: pointer; color: var(--fsn-color-error); font-size: 13px;",
                        onclick: move |_| profile.write().avatar_url = None,
                        "Remove"
                    }
                }
            }

            // Display Name + Email
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Display Name" }
                    input {
                        r#type: "text",
                        value: "{profile.read().display_name}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                        oninput: move |e| profile.write().display_name = e.value(),
                    }
                }
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Email" }
                    input {
                        r#type: "email",
                        value: "{profile.read().email}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                        oninput: move |e| profile.write().email = e.value(),
                    }
                }
            }

            // Bio
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Bio" }
                textarea {
                    style: "width: 100%; height: 80px; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); resize: vertical;",
                    placeholder: "A short description…",
                    oninput: move |e| profile.write().bio = e.value(),
                    "{profile.read().bio}"
                }
            }

            // SSH Keys
            div { style: "margin-bottom: 24px;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                    label { style: "font-weight: 500;", "SSH Keys" }
                    button {
                        style: "padding: 4px 10px; background: var(--fsn-color-primary); color: white; \
                                border: none; border-radius: 4px; cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            let cur = *show_add_key.read();
                            *show_add_key.write() = !cur;
                        },
                        if *show_add_key.read() { "Cancel" } else { "+ Add Key" }
                    }
                }

                // Add Key Form
                if *show_add_key.read() {
                    div {
                        style: "padding: 12px; background: var(--fsn-color-bg-surface); \
                                border-radius: var(--fsn-radius-md); margin-bottom: 8px; \
                                border: 1px solid var(--fsn-color-border-default);",
                        div { style: "margin-bottom: 8px;",
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Label" }
                            input {
                                r#type: "text",
                                placeholder: "e.g. Work Laptop",
                                value: "{new_key_label.read()}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| *new_key_label.write() = e.value(),
                            }
                        }
                        div { style: "margin-bottom: 8px;",
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Public Key" }
                            textarea {
                                style: "width: 100%; height: 60px; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-family: var(--fsn-font-mono); font-size: 12px; resize: vertical;",
                                placeholder: "ssh-ed25519 AAAA…",
                                oninput: move |e| *new_key_value.write() = e.value(),
                                "{new_key_value.read()}"
                            }
                        }
                        button {
                            style: "padding: 6px 14px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px;",
                            disabled: new_key_label.read().is_empty() || new_key_value.read().is_empty(),
                            onclick: move |_| {
                                let label = new_key_label.read().trim().to_string();
                                let key   = new_key_value.read().trim().to_string();
                                if !label.is_empty() && !key.is_empty() {
                                    profile.write().ssh_keys.push(SshKey {
                                        label,
                                        public_key: key,
                                        added_at: chrono_now(),
                                    });
                                    *new_key_label.write() = String::new();
                                    *new_key_value.write() = String::new();
                                    *show_add_key.write() = false;
                                }
                            },
                            "Add"
                        }
                    }
                }

                if profile.read().ssh_keys.is_empty() {
                    div {
                        style: "padding: 12px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); color: var(--fsn-color-text-muted); font-size: 13px;",
                        "No SSH keys added yet."
                    }
                }

                for (idx, key) in profile.read().ssh_keys.iter().enumerate() {
                    div {
                        key: "{idx}",
                        style: "display: flex; align-items: center; gap: 8px; padding: 10px 12px; \
                                background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); \
                                margin-bottom: 4px; border: 1px solid var(--fsn-color-border-default);",
                        div { style: "flex: 1;",
                            div { style: "font-size: 13px; font-weight: 500;", "{key.label}" }
                            div { style: "font-family: var(--fsn-font-mono); font-size: 11px; color: var(--fsn-color-text-muted); overflow: hidden; text-overflow: ellipsis;",
                                "{&key.public_key[..key.public_key.len().min(60)]}…"
                            }
                        }
                        button {
                            style: "color: var(--fsn-color-error); background: none; border: none; cursor: pointer; font-size: 16px;",
                            onclick: move |_| { profile.write().ssh_keys.remove(idx); },
                            "✕"
                        }
                    }
                }
            }

            div { style: "display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        match profile.read().save() {
                            Ok(()) => *save_msg.write() = Some("Profile saved.".into()),
                            Err(e) => *save_msg.write() = Some(format!("Error: {e}")),
                        }
                    },
                    "Save Profile"
                }
                if let Some(msg) = save_msg.read().as_deref() {
                    span { style: "font-size: 13px; color: var(--fsn-color-text-muted);", "{msg}" }
                }
            }
        }
    }
}

fn chrono_now() -> String {
    // ISO 8601 date without external dependency.
    // Uses file-system mtime as a proxy — good enough for "added_at" display.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            // Very simple ISO-8601: 1970-01-01T00:00:00Z arithmetic
            let s = secs % 60;
            let m = (secs / 60) % 60;
            let h = (secs / 3600) % 24;
            let days = secs / 86400;
            // Approximate year (ignoring leap seconds / exact months — for display only)
            let year = 1970 + days / 365;
            let day_of_year = days % 365;
            let month = day_of_year / 30 + 1;
            let day = day_of_year % 30 + 1;
            format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
        })
        .unwrap_or_else(|_| "unknown".into())
}
