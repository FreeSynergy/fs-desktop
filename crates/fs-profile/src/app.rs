/// Profile — user profile, avatar, SSH keys, linked OIDC accounts, and personal capabilities.
use std::path::PathBuf;

use dioxus::prelude::*;
use fs_i18n;
use serde::{Deserialize, Serialize};

// ── PersonalCapability ────────────────────────────────────────────────────────

/// A personal resource this user has connected to the system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PersonalCapability {
    /// User has a Telegram account → can receive personal bot messages.
    MessengerAccount {
        platform: String, // "telegram", "matrix", "signal"
        username: String,
        #[serde(default)]
        verified: bool,
    },
    /// User has personal tasks in a task service (Vikunja).
    TaskManager { service_id: String },
    /// User has a personal mail inbox.
    Mailbox {
        service_id: String,
        address: String,
    },
    /// User has a personal LLM assistant configured.
    LlmAssistant {
        provider: String, // "ollama", "claude", "openai-compatible"
        model: String,
    },
}

impl PersonalCapability {
    pub fn label(&self) -> String {
        match self {
            Self::MessengerAccount { platform, username, .. } => {
                format!("{} (@{})", capitalize(platform), username)
            }
            Self::TaskManager { service_id } => format!("Task Manager ({})", service_id),
            Self::Mailbox { address, .. } => format!("Mailbox ({})", address),
            Self::LlmAssistant { provider, model } => format!("LLM: {} / {}", provider, model),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::MessengerAccount { .. } => "💬",
            Self::TaskManager { .. } => "✅",
            Self::Mailbox { .. } => "📬",
            Self::LlmAssistant { .. } => "🤖",
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::MessengerAccount { .. } => "Messenger",
            Self::TaskManager { .. } => "Task Manager",
            Self::Mailbox { .. } => "Mailbox",
            Self::LlmAssistant { .. } => "LLM Assistant",
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// ── Token-Flow ────────────────────────────────────────────────────────────────

/// Active account-linking token (6-char, expires after 5 minutes).
#[derive(Clone, Debug)]
struct LinkToken {
    token: String,
    platform: String,
    created_at: std::time::Instant,
}

impl LinkToken {
    fn generate(platform: &str) -> Self {
        // Generate a 6-char alphanumeric token using SystemTime — no rand crate needed.
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let chars = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // no ambiguous chars
        let token: String = (0..6)
            .map(|i| chars[((secs >> (i * 5)) & 0x1F) as usize] as char)
            .collect();
        Self {
            token,
            platform: platform.to_string(),
            created_at: std::time::Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() > 300
    }
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A linked OIDC identity from an external provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkedAccount {
    /// Provider display name (e.g. "Kanidm").
    pub provider: String,
    /// OIDC subject identifier (`sub` claim).
    pub subject: String,
    /// Username or email on the remote provider.
    pub username: String,
}

/// User profile data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserProfile {
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub ssh_keys: Vec<SshKey>,
    pub timezone: String,
    #[serde(default)]
    pub linked_accounts: Vec<LinkedAccount>,
    #[serde(default)]
    pub personal_capabilities: Vec<PersonalCapability>,
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

// ── ProfileApp ────────────────────────────────────────────────────────────────

/// Profile app component.
#[component]
pub fn ProfileApp() -> Element {
    let mut profile  = use_signal(UserProfile::load);
    let mut save_msg = use_signal(|| Option::<String>::None);
    let mut new_key_label  = use_signal(String::new);
    let mut new_key_value  = use_signal(String::new);
    let mut show_add_key   = use_signal(|| false);

    // Linked account form state
    let mut show_link     = use_signal(|| false);
    let mut link_provider = use_signal(String::new);
    let mut link_subject  = use_signal(String::new);
    let mut link_username = use_signal(String::new);

    // Personal capabilities form state
    // cap_type: None = hidden, Some("messenger"|"task"|"mailbox"|"llm") = form visible
    let mut cap_type = use_signal(|| Option::<String>::None);

    // Messenger form fields
    let mut cap_msg_platform = use_signal(|| "telegram".to_string());
    let mut cap_msg_username = use_signal(String::new);
    let mut link_token: Signal<Option<LinkToken>> = use_signal(|| None);

    // Task Manager form fields
    let mut cap_task_service = use_signal(|| "vikunja".to_string());

    // Mailbox form fields
    let mut cap_mail_service = use_signal(|| "stalwart".to_string());
    let mut cap_mail_address = use_signal(String::new);

    // LLM Assistant form fields
    let mut cap_llm_provider = use_signal(|| "ollama".to_string());
    let mut cap_llm_model    = use_signal(|| "llama3".to_string());

    rsx! {
        div {
            class: "fs-profile",
            style: "padding: 24px; max-width: 600px;",

            h2 { style: "margin-top: 0;", {fs_i18n::t("profile.title")} }

            // Avatar
            div { style: "display: flex; align-items: center; gap: 24px; margin-bottom: 32px;",
                div {
                    style: "width: 80px; height: 80px; border-radius: 50%; background: var(--fs-color-bg-overlay); \
                            display: flex; align-items: center; justify-content: center; font-size: 32px; \
                            border: 2px solid var(--fs-color-border-default);",
                    if let Some(url) = profile.read().avatar_url.clone() {
                        img { src: "{url}", width: "80", height: "80", style: "border-radius: 50%;" }
                    } else {
                        "👤"
                    }
                }
                div {
                    button {
                        style: "display: block; padding: 6px 12px; background: var(--fs-color-bg-surface); \
                                border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); \
                                cursor: pointer; margin-bottom: 8px;",
                        {fs_i18n::t("profile.btn.upload_photo")}
                    }
                    button {
                        style: "display: block; padding: 6px 12px; background: none; border: none; \
                                cursor: pointer; color: var(--fs-color-error); font-size: 13px;",
                        onclick: move |_| profile.write().avatar_url = None,
                        {fs_i18n::t("actions.remove")}
                    }
                }
            }

            // Display Name + Email
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.display_name")} }
                    input {
                        r#type: "text",
                        value: "{profile.read().display_name}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md);",
                        oninput: move |e| profile.write().display_name = e.value(),
                    }
                }
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.email")} }
                    input {
                        r#type: "email",
                        value: "{profile.read().email}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md);",
                        oninput: move |e| profile.write().email = e.value(),
                    }
                }
            }

            // Bio
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.bio")} }
                textarea {
                    style: "width: 100%; height: 80px; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); resize: vertical;",
                    placeholder: "A short description…",
                    oninput: move |e| profile.write().bio = e.value(),
                    "{profile.read().bio}"
                }
            }

            // SSH Keys
            div { style: "margin-bottom: 24px;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                    label { style: "font-weight: 500;", {fs_i18n::t("profile.section.ssh_keys")} }
                    button {
                        style: "padding: 4px 10px; background: var(--fs-color-primary); color: white; \
                                border: none; border-radius: 4px; cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            let cur = *show_add_key.read();
                            *show_add_key.write() = !cur;
                        },
                        if *show_add_key.read() { {fs_i18n::t("actions.cancel")} } else { {fs_i18n::t("profile.btn.add_key")} }
                    }
                }

                // Add Key Form
                if *show_add_key.read() {
                    div {
                        style: "padding: 12px; background: var(--fs-color-bg-surface); \
                                border-radius: var(--fs-radius-md); margin-bottom: 8px; \
                                border: 1px solid var(--fs-color-border-default);",
                        div { style: "margin-bottom: 8px;",
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.key_label")} }
                            input {
                                r#type: "text",
                                placeholder: "e.g. Work Laptop",
                                value: "{new_key_label.read()}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                oninput: move |e| *new_key_label.write() = e.value(),
                            }
                        }
                        div { style: "margin-bottom: 8px;",
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.public_key")} }
                            textarea {
                                style: "width: 100%; height: 60px; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-family: var(--fs-font-mono); font-size: 12px; resize: vertical;",
                                placeholder: "ssh-ed25519 AAAA…",
                                oninput: move |e| *new_key_value.write() = e.value(),
                                "{new_key_value.read()}"
                            }
                        }
                        button {
                            style: "padding: 6px 14px; background: var(--fs-color-primary); color: white; border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
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
                            {fs_i18n::t("actions.add")}
                        }
                    }
                }

                if profile.read().ssh_keys.is_empty() {
                    div {
                        style: "padding: 12px; background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); color: var(--fs-color-text-muted); font-size: 13px;",
                        "No SSH keys added yet."
                    }
                }

                for (idx, key) in profile.read().ssh_keys.iter().enumerate() {
                    div {
                        key: "{idx}",
                        style: "display: flex; align-items: center; gap: 8px; padding: 10px 12px; \
                                background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); \
                                margin-bottom: 4px; border: 1px solid var(--fs-color-border-default);",
                        div { style: "flex: 1;",
                            div { style: "font-size: 13px; font-weight: 500;", "{key.label}" }
                            div { style: "font-family: var(--fs-font-mono); font-size: 11px; color: var(--fs-color-text-muted); overflow: hidden; text-overflow: ellipsis;",
                                "{&key.public_key[..key.public_key.len().min(60)]}…"
                            }
                        }
                        button {
                            style: "color: var(--fs-color-error); background: none; border: none; cursor: pointer; font-size: 16px;",
                            onclick: move |_| { profile.write().ssh_keys.remove(idx); },
                            "✕"
                        }
                    }
                }
            }

            // ── Linked OIDC Accounts ───────────────────────────────────────────────
            div { style: "margin-bottom: 24px;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                    div {
                        label { style: "font-weight: 500; display: block;", {fs_i18n::t("profile.section.linked_accounts")} }
                        p { style: "margin: 2px 0 0; font-size: 12px; color: var(--fs-color-text-muted);",
                            "OIDC identities linked to this profile."
                        }
                    }
                    button {
                        style: "padding: 4px 10px; background: var(--fs-color-primary); color: white; \
                                border: none; border-radius: 4px; cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            let cur = *show_link.read();
                            *show_link.write() = !cur;
                            *link_provider.write() = String::new();
                            *link_subject.write() = String::new();
                            *link_username.write() = String::new();
                        },
                        if *show_link.read() { {fs_i18n::t("actions.cancel")} } else { {fs_i18n::t("profile.btn.link_account")} }
                    }
                }

                if *show_link.read() {
                    div {
                        style: "padding: 12px; background: var(--fs-color-bg-surface); \
                                border-radius: var(--fs-radius-md); margin-bottom: 8px; \
                                border: 1px solid var(--fs-color-border-default);",
                        div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; margin-bottom: 8px;",
                            div {
                                label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.provider")} }
                                input {
                                    r#type: "text", placeholder: "e.g. Kanidm",
                                    value: "{link_provider.read()}",
                                    style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                    oninput: move |e| *link_provider.write() = e.value(),
                                }
                            }
                            div {
                                label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("labels.name")} }
                                input {
                                    r#type: "text", placeholder: "e.g. alice",
                                    value: "{link_username.read()}",
                                    style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                    oninput: move |e| *link_username.write() = e.value(),
                                }
                            }
                            div {
                                label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", {fs_i18n::t("profile.label.subject")} }
                                input {
                                    r#type: "text", placeholder: "OIDC sub claim",
                                    value: "{link_subject.read()}",
                                    style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                    oninput: move |e| *link_subject.write() = e.value(),
                                }
                            }
                        }
                        button {
                            disabled: link_provider.read().trim().is_empty() || link_username.read().trim().is_empty(),
                            style: "padding: 6px 14px; background: var(--fs-color-primary); color: white; border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                            onclick: move |_| {
                                let provider = link_provider.read().trim().to_string();
                                let username = link_username.read().trim().to_string();
                                let subject  = link_subject.read().trim().to_string();
                                if !provider.is_empty() && !username.is_empty() {
                                    profile.write().linked_accounts.push(LinkedAccount {
                                        provider, username, subject,
                                    });
                                    *show_link.write() = false;
                                }
                            },
                            {fs_i18n::t("actions.add")}
                        }
                    }
                }

                if profile.read().linked_accounts.is_empty() {
                    div {
                        style: "padding: 12px; background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); color: var(--fs-color-text-muted); font-size: 13px;",
                        "No accounts linked yet."
                    }
                }

                for (idx, acct) in profile.read().linked_accounts.iter().enumerate() {
                    div {
                        key: "{idx}",
                        style: "display: flex; align-items: center; gap: 8px; padding: 10px 12px; \
                                background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); \
                                margin-bottom: 4px; border: 1px solid var(--fs-color-border-default);",
                        div {
                            style: "width: 28px; height: 28px; border-radius: 50%; background: var(--fs-color-bg-overlay); \
                                    display: flex; align-items: center; justify-content: center; font-size: 14px; flex-shrink: 0;",
                            "🔐"
                        }
                        div { style: "flex: 1;",
                            div { style: "font-size: 13px; font-weight: 500;",
                                "{acct.provider}  ·  {acct.username}"
                            }
                            if !acct.subject.is_empty() {
                                div { style: "font-family: var(--fs-font-mono); font-size: 11px; color: var(--fs-color-text-muted);",
                                    "sub: {acct.subject}"
                                }
                            }
                        }
                        button {
                            style: "color: var(--fs-color-error); background: none; border: none; cursor: pointer; font-size: 16px;",
                            title: "Unlink",
                            onclick: move |_| { profile.write().linked_accounts.remove(idx); },
                            "✕"
                        }
                    }
                }
            }

            // ── Personal Capabilities ──────────────────────────────────────────────
            div { style: "margin-bottom: 24px;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                    div {
                        label { style: "font-weight: 500; display: block;", {fs_i18n::t("profile.section.capabilities")} }
                        p { style: "margin: 2px 0 0; font-size: 12px; color: var(--fs-color-text-muted);",
                            "Messengers, task managers, mailboxes, and AI assistants."
                        }
                    }
                    // Show "+ Add Capability" only when no form is open
                    if cap_type.read().is_none() {
                        div { style: "position: relative;",
                            select {
                                style: "padding: 4px 10px; background: var(--fs-color-primary); color: white; \
                                        border: none; border-radius: 4px; cursor: pointer; font-size: 13px; \
                                        appearance: none; -webkit-appearance: none;",
                                onchange: move |e| {
                                    let v = e.value();
                                    if v != "__placeholder__" {
                                        *cap_type.write() = Some(v.clone());
                                        // Reset form fields when type changes
                                        *cap_msg_platform.write() = "telegram".to_string();
                                        *cap_msg_username.write() = String::new();
                                        *link_token.write() = None;
                                        *cap_task_service.write() = "vikunja".to_string();
                                        *cap_mail_service.write() = "stalwart".to_string();
                                        *cap_mail_address.write() = String::new();
                                        *cap_llm_provider.write() = "ollama".to_string();
                                        *cap_llm_model.write() = "llama3".to_string();
                                    }
                                },
                                option { value: "__placeholder__", {fs_i18n::t("profile.btn.add_capability")} }
                                option { value: "messenger", {fs_i18n::t("profile.capability.messenger_account")} }
                                option { value: "task", {fs_i18n::t("profile.capability.task_manager")} }
                                option { value: "mailbox", {fs_i18n::t("profile.capability.mailbox")} }
                                option { value: "llm", {fs_i18n::t("profile.capability.llm_assistant")} }
                            }
                        }
                    }
                    if cap_type.read().is_some() {
                        button {
                            style: "padding: 4px 10px; background: none; border: 1px solid var(--fs-color-border-default); \
                                    border-radius: 4px; cursor: pointer; font-size: 13px;",
                            onclick: move |_| {
                                *cap_type.write() = None;
                                *link_token.write() = None;
                            },
                            {fs_i18n::t("actions.cancel")}
                        }
                    }
                }

                // ── Capability form (shown when a type is selected) ────────────────
                if let Some(ct) = cap_type.read().clone() {
                    div {
                        style: "padding: 12px; background: var(--fs-color-bg-surface); \
                                border-radius: var(--fs-radius-md); margin-bottom: 8px; \
                                border: 1px solid var(--fs-color-border-default);",

                        // ── Messenger Account form ─────────────────────────────────
                        if ct == "messenger" {
                            div {
                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 12px;",
                                    div {
                                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Platform" }
                                        select {
                                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                            onchange: move |e| {
                                                *cap_msg_platform.write() = e.value();
                                                *link_token.write() = None;
                                            },
                                            option { value: "telegram", selected: cap_msg_platform.read().as_str() == "telegram", "Telegram" }
                                            option { value: "matrix",   selected: cap_msg_platform.read().as_str() == "matrix",   "Matrix" }
                                            option { value: "signal",   selected: cap_msg_platform.read().as_str() == "signal",   "Signal" }
                                            option { value: "discord",  selected: cap_msg_platform.read().as_str() == "discord",  "Discord" }
                                        }
                                    }
                                    div {
                                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Username" }
                                        input {
                                            r#type: "text",
                                            placeholder: "e.g. myuser",
                                            value: "{cap_msg_username.read()}",
                                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                            oninput: move |e| {
                                                *cap_msg_username.write() = e.value();
                                                *link_token.write() = None;
                                            },
                                        }
                                    }
                                }

                                // Token-flow: generate button (only before token is generated)
                                if link_token.read().is_none() {
                                    button {
                                        style: "padding: 6px 14px; background: var(--fs-color-primary); color: white; \
                                                border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                                        disabled: cap_msg_username.read().trim().is_empty(),
                                        onclick: move |_| {
                                            let platform = cap_msg_platform.read().clone();
                                            *link_token.write() = Some(LinkToken::generate(&platform));
                                        },
                                        "Generate Link Token"
                                    }
                                }

                                // Token-flow panel (shown after token is generated)
                                if let Some(token) = link_token.read().clone() {
                                    div {
                                        style: "margin-top: 12px; padding: 14px 16px; \
                                                background: var(--fs-color-bg-overlay); \
                                                border: 1px solid var(--fs-color-border-default); \
                                                border-radius: var(--fs-radius-md); font-size: 13px;",
                                        if token.is_expired() {
                                            p { style: "color: var(--fs-color-error); margin: 0 0 8px;",
                                                "⚠ Token expired. Please regenerate."
                                            }
                                        } else {
                                            p { style: "margin: 0 0 6px; color: var(--fs-color-text-muted);",
                                                "Send this message to "
                                                strong { "@FreeSynergyBot" }
                                                " on "
                                                strong { "{capitalize(&token.platform)}" }
                                                ":"
                                            }
                                            div {
                                                style: "font-family: var(--fs-font-mono); font-size: 16px; font-weight: 700; \
                                                        letter-spacing: 2px; padding: 8px 12px; \
                                                        background: var(--fs-color-bg-surface); \
                                                        border-radius: var(--fs-radius-md); margin-bottom: 8px; \
                                                        color: var(--fs-color-primary);",
                                                "/verify {token.token}"
                                            }
                                            p { style: "margin: 0 0 12px; font-size: 12px; color: var(--fs-color-text-muted);",
                                                "Token expires in 5 minutes."
                                            }
                                        }
                                        div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                            button {
                                                style: "padding: 5px 12px; border: 1px solid var(--fs-color-border-default); \
                                                        background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); \
                                                        cursor: pointer; font-size: 13px;",
                                                onclick: move |_| {
                                                    let platform = cap_msg_platform.read().clone();
                                                    *link_token.write() = Some(LinkToken::generate(&platform));
                                                },
                                                "Regenerate"
                                            }
                                            // "Mark as Verified" — user manually confirms after sending /verify
                                            button {
                                                style: "padding: 5px 12px; background: var(--fs-color-primary); color: white; \
                                                        border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                                                onclick: move |_| {
                                                    let platform = cap_msg_platform.read().trim().to_string();
                                                    let username = cap_msg_username.read().trim().to_string();
                                                    if !username.is_empty() {
                                                        profile.write().personal_capabilities.push(
                                                            PersonalCapability::MessengerAccount {
                                                                platform,
                                                                username,
                                                                verified: true,
                                                            }
                                                        );
                                                        *cap_type.write() = None;
                                                        *link_token.write() = None;
                                                    }
                                                },
                                                "Mark as Verified"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // ── Task Manager form ──────────────────────────────────────
                        if ct == "task" {
                            div {
                                div { style: "margin-bottom: 8px;",
                                    label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Service ID" }
                                    input {
                                        r#type: "text",
                                        placeholder: "vikunja",
                                        value: "{cap_task_service.read()}",
                                        style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                        oninput: move |e| *cap_task_service.write() = e.value(),
                                    }
                                }
                                button {
                                    style: "padding: 6px 14px; background: var(--fs-color-primary); color: white; border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                                    disabled: cap_task_service.read().trim().is_empty(),
                                    onclick: move |_| {
                                        let service_id = cap_task_service.read().trim().to_string();
                                        if !service_id.is_empty() {
                                            profile.write().personal_capabilities.push(
                                                PersonalCapability::TaskManager { service_id }
                                            );
                                            *cap_type.write() = None;
                                        }
                                    },
                                    {fs_i18n::t("actions.add")}
                                }
                            }
                        }

                        // ── Mailbox form ───────────────────────────────────────────
                        if ct == "mailbox" {
                            div {
                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 8px;",
                                    div {
                                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Service ID" }
                                        input {
                                            r#type: "text",
                                            placeholder: "stalwart",
                                            value: "{cap_mail_service.read()}",
                                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                            oninput: move |e| *cap_mail_service.write() = e.value(),
                                        }
                                    }
                                    div {
                                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Email Address" }
                                        input {
                                            r#type: "email",
                                            placeholder: "user@example.com",
                                            value: "{cap_mail_address.read()}",
                                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                            oninput: move |e| *cap_mail_address.write() = e.value(),
                                        }
                                    }
                                }
                                button {
                                    style: "padding: 6px 14px; background: var(--fs-color-primary); color: white; border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                                    disabled: cap_mail_service.read().trim().is_empty() || cap_mail_address.read().trim().is_empty(),
                                    onclick: move |_| {
                                        let service_id = cap_mail_service.read().trim().to_string();
                                        let address    = cap_mail_address.read().trim().to_string();
                                        if !service_id.is_empty() && !address.is_empty() {
                                            profile.write().personal_capabilities.push(
                                                PersonalCapability::Mailbox { service_id, address }
                                            );
                                            *cap_type.write() = None;
                                        }
                                    },
                                    {fs_i18n::t("actions.add")}
                                }
                            }
                        }

                        // ── LLM Assistant form ─────────────────────────────────────
                        if ct == "llm" {
                            div {
                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 8px;",
                                    div {
                                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Provider" }
                                        select {
                                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                            onchange: move |e| *cap_llm_provider.write() = e.value(),
                                            option { value: "ollama",             selected: cap_llm_provider.read().as_str() == "ollama",             "Ollama" }
                                            option { value: "claude",             selected: cap_llm_provider.read().as_str() == "claude",             "Claude" }
                                            option { value: "openai-compatible",  selected: cap_llm_provider.read().as_str() == "openai-compatible",  "OpenAI-compatible" }
                                        }
                                    }
                                    div {
                                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Model" }
                                        input {
                                            r#type: "text",
                                            placeholder: "llama3",
                                            value: "{cap_llm_model.read()}",
                                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); border-radius: var(--fs-radius-md); font-size: 13px;",
                                            oninput: move |e| *cap_llm_model.write() = e.value(),
                                        }
                                    }
                                }
                                button {
                                    style: "padding: 6px 14px; background: var(--fs-color-primary); color: white; border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                                    disabled: cap_llm_provider.read().trim().is_empty() || cap_llm_model.read().trim().is_empty(),
                                    onclick: move |_| {
                                        let provider = cap_llm_provider.read().trim().to_string();
                                        let model    = cap_llm_model.read().trim().to_string();
                                        if !provider.is_empty() && !model.is_empty() {
                                            profile.write().personal_capabilities.push(
                                                PersonalCapability::LlmAssistant { provider, model }
                                            );
                                            *cap_type.write() = None;
                                        }
                                    },
                                    {fs_i18n::t("actions.add")}
                                }
                            }
                        }
                    }
                }

                // ── Capabilities list ──────────────────────────────────────────────
                if profile.read().personal_capabilities.is_empty() {
                    div {
                        style: "padding: 12px; background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); color: var(--fs-color-text-muted); font-size: 13px;",
                        "No personal capabilities added yet."
                    }
                }

                for (idx, cap) in profile.read().personal_capabilities.iter().enumerate() {
                    div {
                        key: "{idx}",
                        style: "display: flex; align-items: center; gap: 8px; padding: 10px 12px; \
                                background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); \
                                margin-bottom: 4px; border: 1px solid var(--fs-color-border-default);",
                        div {
                            style: "width: 28px; height: 28px; border-radius: 50%; background: var(--fs-color-bg-overlay); \
                                    display: flex; align-items: center; justify-content: center; font-size: 14px; flex-shrink: 0;",
                            "{cap.icon()}"
                        }
                        div { style: "flex: 1;",
                            div { style: "font-size: 12px; color: var(--fs-color-text-muted);", "{cap.kind_label()}" }
                            div { style: "font-size: 13px; font-weight: 500;", "{cap.label()}" }
                        }
                        button {
                            style: "color: var(--fs-color-error); background: none; border: none; cursor: pointer; font-size: 16px;",
                            title: "Remove",
                            onclick: move |_| { profile.write().personal_capabilities.remove(idx); },
                            "✕"
                        }
                    }
                }
            }

            div { style: "display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fs-color-primary); color: white; border: none; border-radius: var(--fs-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        match profile.read().save() {
                            Ok(()) => *save_msg.write() = Some(fs_i18n::t("profile.msg.saved").to_string()),
                            Err(e) => *save_msg.write() = Some(fs_i18n::t_with("profile.msg.save_failed", &[("error", e.as_str())]).to_string()),
                        }
                    },
                    {fs_i18n::t("profile.btn.save")}
                }
                if let Some(msg) = save_msg.read().as_deref() {
                    span { style: "font-size: 13px; color: var(--fs-color-text-muted);", "{msg}" }
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
