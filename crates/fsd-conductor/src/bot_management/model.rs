// model.rs — Bot data model, trigger/action enums, and TOML persistence.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── BotTrigger ────────────────────────────────────────────────────────────────

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
    pub fn label(&self) -> &'static str {
        match self {
            Self::OnStartup => "On startup",
            Self::Interval { .. } => "Interval",
        }
    }
}

// ── BotAction ─────────────────────────────────────────────────────────────────

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
    pub fn label(&self) -> String {
        match self {
            Self::Start { service }      => format!("Start {service}"),
            Self::Stop { service }       => format!("Stop {service}"),
            Self::Restart { service }    => format!("Restart {service}"),
            Self::RunCommand { command } => format!("Run: {command}"),
        }
    }
}

// ── Bot ───────────────────────────────────────────────────────────────────────

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

// ── BotsConfig ────────────────────────────────────────────────────────────────

/// Root structure of `~/.config/fsn/bots.toml`.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct BotsConfig {
    #[serde(default)]
    bots: Vec<Bot>,
}

impl BotsConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("bots.toml")
    }

    pub(super) fn load() -> Vec<Bot> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<BotsConfig>(&content)
            .unwrap_or_default()
            .bots
    }

    pub(super) fn save(bots: &[Bot]) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let cfg = BotsConfig { bots: bots.to_vec() };
        let content = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}
