/// Bot management — define and run automated container lifecycle tasks.
///
/// A Bot is a named rule that triggers a container action (start/stop/restart)
/// either on startup or on a simple interval. Bots are persisted in
/// `~/.config/fsn/bots.toml`.
use std::path::PathBuf;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// ── Data model ────────────────────────────────────────────────────────────────

/// When a bot fires.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BotTrigger {
    /// Fires once when the desktop starts.
    OnStartup,
    /// Fires every `interval_secs` seconds.
    Interval { interval_secs: u64 },
}

impl BotTrigger {
    fn label(&self) -> &'static str {
        match self {
            BotTrigger::OnStartup => "On startup",
            BotTrigger::Interval { .. } => "Interval",
        }
    }
}

/// What a bot does when it fires.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BotAction {
    /// Start a container by name.
    Start { service: String },
    /// Stop a container by name.
    Stop { service: String },
    /// Restart a container by name.
    Restart { service: String },
    /// Run a shell command (non-interactive, output to log).
    RunCommand { command: String },
}

impl BotAction {
    fn label(&self) -> String {
        match self {
            BotAction::Start { service } => format!("Start {service}"),
            BotAction::Stop { service } => format!("Stop {service}"),
            BotAction::Restart { service } => format!("Restart {service}"),
            BotAction::RunCommand { command } => format!("Run: {command}"),
        }
    }
}

/// A single bot definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    /// Unique name for this bot.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// When to fire.
    pub trigger: BotTrigger,
    /// What to do.
    pub action: BotAction,
    /// Whether this bot is active.
    pub enabled: bool,
}

/// Root config file structure.
#[derive(Default, Serialize, Deserialize)]
struct BotsConfig {
    #[serde(default)]
    bots: Vec<Bot>,
}

impl BotsConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("bots.toml")
    }

    fn load() -> Vec<Bot> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<BotsConfig>(&content)
            .unwrap_or_default()
            .bots
    }

    fn save(bots: &[Bot]) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let cfg = BotsConfig { bots: bots.to_vec() };
        let content = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

// ── Add-bot form state ────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct AddBotForm {
    name: String,
    description: String,
    trigger_kind: String,    // "startup" | "interval"
    interval_secs: String,
    action_kind: String,     // "start" | "stop" | "restart" | "command"
    service_or_cmd: String,
}

impl AddBotForm {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.service_or_cmd.trim().is_empty()
            && !self.action_kind.is_empty()
            && !self.trigger_kind.is_empty()
    }

    fn build_bot(&self) -> Option<Bot> {
        if !self.is_valid() {
            return None;
        }
        let trigger = match self.trigger_kind.as_str() {
            "interval" => {
                let secs = self.interval_secs.parse::<u64>().unwrap_or(300);
                BotTrigger::Interval { interval_secs: secs }
            }
            _ => BotTrigger::OnStartup,
        };
        let svc = self.service_or_cmd.trim().to_string();
        let action = match self.action_kind.as_str() {
            "stop" => BotAction::Stop { service: svc },
            "restart" => BotAction::Restart { service: svc },
            "command" => BotAction::RunCommand { command: svc },
            _ => BotAction::Start { service: svc },
        };
        Some(Bot {
            name: self.name.trim().to_string(),
            description: self.description.trim().to_string(),
            trigger,
            action,
            enabled: true,
        })
    }
}

// ── Bot row child component ───────────────────────────────────────────────────

#[component]
fn BotRow(
    idx: usize,
    bot: Bot,
    mut bots: Signal<Vec<Bot>>,
    mut status_msg: Signal<Option<String>>,
) -> Element {
    let trigger_label = bot.trigger.label();
    let action_label  = bot.action.label();
    let opacity       = if bot.enabled { "1" } else { "0.55" };

    rsx! {
        div {
            key: "{idx}",
            style: "display: flex; align-items: center; gap: 12px; padding: 12px 14px; \
                    background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); \
                    margin-bottom: 8px; border: 1px solid var(--fsn-color-border-default); \
                    opacity: {opacity};",

            // Toggle
            input {
                r#type: "checkbox",
                checked: bot.enabled,
                style: "cursor: pointer; width: 16px; height: 16px; flex-shrink: 0;",
                onchange: move |_| {
                    // `bot.enabled` is the value at render time — flip it.
                    bots.write()[idx].enabled = !bot.enabled;
                    save_bots(bots, status_msg);
                },
            }

            // Info
            div { style: "flex: 1; min-width: 0;",
                div { style: "font-weight: 500; font-size: 14px;", "{bot.name}" }
                if !bot.description.is_empty() {
                    div {
                        style: "font-size: 12px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                        "{bot.description}"
                    }
                }
                div { style: "display: flex; gap: 8px; margin-top: 4px;",
                    span {
                        style: "font-size: 11px; padding: 2px 8px; border-radius: 9999px; \
                                background: var(--fsn-color-bg-overlay); color: var(--fsn-color-text-muted);",
                        "⏱ {trigger_label}"
                    }
                    span {
                        style: "font-size: 11px; padding: 2px 8px; border-radius: 9999px; \
                                background: var(--fsn-color-bg-overlay); color: var(--fsn-color-text-muted);",
                        "▶ {action_label}"
                    }
                }
            }

            // Delete
            button {
                style: "color: var(--fsn-color-error); background: none; border: none; \
                        cursor: pointer; font-size: 18px; flex-shrink: 0;",
                onclick: move |_| {
                    bots.write().remove(idx);
                    save_bots(bots, status_msg);
                },
                "✕"
            }
        }
    }
}

fn save_bots(bots: Signal<Vec<Bot>>, mut status_msg: Signal<Option<String>>) {
    match BotsConfig::save(&*bots.read()) {
        Ok(()) => status_msg.set(None),
        Err(e) => status_msg.set(Some(format!("Save error: {e}"))),
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Bot management tab — list, add, toggle, and remove bots.
#[component]
pub fn BotManagement() -> Element {
    let mut bots       = use_signal(BotsConfig::load);
    let mut show_add   = use_signal(|| false);
    let mut form       = use_signal(AddBotForm::default);
    let mut status_msg = use_signal(|| Option::<String>::None);

    let showing_add   = *show_add.read();
    let is_empty      = bots.read().is_empty();
    let add_btn_label = if showing_add { "Cancel" } else { "+ Add Bot" };
    let show_interval = form.read().trigger_kind == "interval";
    let is_command    = form.read().action_kind == "command";
    let form_valid    = form.read().is_valid();
    let svc_label     = if is_command { "Command" } else { "Service name" };
    let svc_hint      = if is_command { "e.g. /usr/bin/fsn sync" } else { "e.g. zentinel" };
    let save_err      = status_msg.read().clone();

    // Collect snapshot so the for loop doesn't hold a signal read guard.
    let bot_list: Vec<Bot> = bots.read().clone();

    rsx! {
        div {
            class: "fsd-bots",
            style: "padding: 0;",

            // Header row
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                div {
                    h3 { style: "margin: 0;", "Bots" }
                    p {
                        style: "margin: 4px 0 0; font-size: 13px; color: var(--fsn-color-text-muted);",
                        "Automated container lifecycle rules — triggered on startup or on an interval."
                    }
                }
                button {
                    style: "padding: 8px 16px; background: var(--fsn-color-primary); color: white; \
                            border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        show_add.set(!showing_add);
                        form.set(AddBotForm::default());
                    },
                    "{add_btn_label}"
                }
            }

            // Add-bot form
            if showing_add {
                div {
                    style: "padding: 16px; background: var(--fsn-color-bg-surface); \
                            border-radius: var(--fsn-radius-md); border: 1px solid var(--fsn-color-border-default); \
                            margin-bottom: 16px;",

                    h4 { style: "margin: 0 0 12px;", "New Bot" }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;",
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Name" }
                            input {
                                r#type: "text",
                                placeholder: "e.g. auto-restart-proxy",
                                value: "{form.read().name}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| form.write().name = e.value(),
                            }
                        }
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Description" }
                            input {
                                r#type: "text",
                                placeholder: "Optional",
                                value: "{form.read().description}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| form.write().description = e.value(),
                            }
                        }
                    }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;",
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Trigger" }
                            select {
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); font-size: 13px;",
                                onchange: move |e| form.write().trigger_kind = e.value(),
                                option { value: "", "— select —" }
                                option { value: "startup", "On startup" }
                                option { value: "interval", "Interval" }
                            }
                        }
                        if show_interval {
                            div {
                                label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Every (seconds)" }
                                input {
                                    r#type: "number",
                                    placeholder: "300",
                                    value: "{form.read().interval_secs}",
                                    style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                                            border-radius: var(--fsn-radius-md); font-size: 13px;",
                                    oninput: move |e| form.write().interval_secs = e.value(),
                                }
                            }
                        }
                    }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 16px;",
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Action" }
                            select {
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); font-size: 13px;",
                                onchange: move |e| form.write().action_kind = e.value(),
                                option { value: "", "— select —" }
                                option { value: "start", "Start container" }
                                option { value: "stop", "Stop container" }
                                option { value: "restart", "Restart container" }
                                option { value: "command", "Run command" }
                            }
                        }
                        div {
                            label {
                                style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;",
                                "{svc_label}"
                            }
                            input {
                                r#type: "text",
                                placeholder: "{svc_hint}",
                                value: "{form.read().service_or_cmd}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| form.write().service_or_cmd = e.value(),
                            }
                        }
                    }

                    button {
                        disabled: !form_valid,
                        style: "padding: 8px 20px; background: var(--fsn-color-primary); color: white; \
                                border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| {
                            let built = form.read().build_bot();
                            if let Some(bot) = built {
                                bots.write().push(bot);
                                save_bots(bots, status_msg);
                                show_add.set(false);
                                form.set(AddBotForm::default());
                            }
                        },
                        "Add Bot"
                    }
                }
            }

            // Bot list — empty state
            if is_empty {
                div {
                    style: "text-align: center; padding: 40px; background: var(--fsn-color-bg-surface); \
                            border-radius: var(--fsn-radius-md); border: 1px dashed var(--fsn-color-border-default);",
                    p { style: "color: var(--fsn-color-text-muted); margin: 0;", "No bots defined yet." }
                    p { style: "font-size: 12px; color: var(--fsn-color-text-muted); margin: 8px 0 0;",
                        "Add a bot to automate container start/stop/restart actions."
                    }
                }
            }

            // Bot list — rows
            for (idx, bot) in bot_list.into_iter().enumerate() {
                BotRow {
                    key: "{idx}",
                    idx,
                    bot,
                    bots,
                    status_msg,
                }
            }

            // Save error
            if let Some(msg) = save_err {
                p { style: "font-size: 12px; color: var(--fsn-color-error); margin-top: 8px;", "{msg}" }
            }
        }
    }
}
