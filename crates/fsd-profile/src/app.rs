/// Profile — user profile, avatar, SSH keys, and display preferences.
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

/// Profile app component.
#[component]
pub fn ProfileApp() -> Element {
    let profile = use_signal(UserProfile::default);

    rsx! {
        div {
            class: "fsd-profile",
            style: "padding: 24px; max-width: 600px;",

            h2 { style: "margin-top: 0;", "Profile" }

            // Avatar
            div { style: "display: flex; align-items: center; gap: 24px; margin-bottom: 32px;",
                div {
                    style: "width: 80px; height: 80px; border-radius: 50%; background: var(--fsn-color-bg-overlay); display: flex; align-items: center; justify-content: center; font-size: 32px; border: 2px solid var(--fsn-color-border-default);",
                    if let Some(url) = &profile.read().avatar_url {
                        img { src: "{url}", width: "80", height: "80", style: "border-radius: 50%;" }
                    } else {
                        "👤"
                    }
                }
                div {
                    button {
                        style: "display: block; padding: 6px 12px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer; margin-bottom: 8px;",
                        "Upload photo"
                    }
                    button {
                        style: "display: block; padding: 6px 12px; background: none; border: none; cursor: pointer; color: var(--fsn-color-error); font-size: 13px;",
                        "Remove"
                    }
                }
            }

            // Name + Email
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Display Name" }
                    input {
                        r#type: "text",
                        value: "{profile.read().display_name}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                    }
                }
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Email" }
                    input {
                        r#type: "email",
                        value: "{profile.read().email}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                    }
                }
            }

            // Bio
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Bio" }
                textarea {
                    style: "width: 100%; height: 80px; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); resize: vertical;",
                    placeholder: "A short description…",
                    "{profile.read().bio}"
                }
            }

            // SSH Keys
            div { style: "margin-bottom: 24px;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                    label { style: "font-weight: 500;", "SSH Keys" }
                    button {
                        style: "padding: 4px 10px; background: var(--fsn-color-primary); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px;",
                        "+ Add Key"
                    }
                }
                if profile.read().ssh_keys.is_empty() {
                    div {
                        style: "padding: 12px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); color: var(--fsn-color-text-muted); font-size: 13px;",
                        "No SSH keys added yet."
                    }
                }
                for key in profile.read().ssh_keys.iter() {
                    div {
                        style: "display: flex; align-items: center; gap: 8px; padding: 10px 12px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); margin-bottom: 4px;",
                        span { style: "flex: 1; font-family: var(--fsn-font-mono); font-size: 12px; overflow: hidden; text-overflow: ellipsis;",
                            "{key.label}: {&key.public_key[..key.public_key.len().min(40)]}…"
                        }
                        button {
                            style: "color: var(--fsn-color-error); background: none; border: none; cursor: pointer;",
                            "✕"
                        }
                    }
                }
            }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                "Save Profile"
            }
        }
    }
}
