use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── BotKind ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotKind {
    Broadcast,
    Gatekeeper,
    Monitor,
    Digest,
    UserBot,
}

impl BotKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Broadcast  => "Broadcast",
            Self::Gatekeeper => "Gatekeeper",
            Self::Monitor    => "Monitor",
            Self::Digest     => "Digest",
            Self::UserBot    => "Personal Assistant",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Broadcast  => "📢",
            Self::Gatekeeper => "🔒",
            Self::Monitor    => "📊",
            Self::Digest     => "📋",
            Self::UserBot    => "🤖",
        }
    }
}

// ── Platform ──────────────────────────────────────────────────────────────────

/// A credential field definition: (name, placeholder, is_secret).
pub struct CredentialField {
    pub name:        &'static str,
    pub placeholder: &'static str,
    pub is_secret:   bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Telegram,
    Matrix,
    Discord,
    RocketChat,
    Mattermost,
    Slack,
    XMPP,
}

impl Platform {
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Telegram,
            Platform::Matrix,
            Platform::Discord,
            Platform::RocketChat,
            Platform::Mattermost,
            Platform::Slack,
            Platform::XMPP,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Telegram   => "Telegram",
            Self::Matrix     => "Matrix",
            Self::Discord    => "Discord",
            Self::RocketChat => "Rocket.Chat",
            Self::Mattermost => "Mattermost",
            Self::Slack      => "Slack",
            Self::XMPP       => "XMPP",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Telegram   => "✈",
            Self::Matrix     => "🔷",
            Self::Discord    => "🎮",
            Self::RocketChat => "🚀",
            Self::Mattermost => "📡",
            Self::Slack      => "💼",
            Self::XMPP       => "💬",
        }
    }

    pub fn credential_fields(&self) -> Vec<CredentialField> {
        match self {
            Self::Telegram => vec![
                CredentialField { name: "Bot Token",       placeholder: "1234567890:ABCdef...", is_secret: true  },
                CredentialField { name: "Phone (UserBot)", placeholder: "+49123456789",         is_secret: false },
            ],
            Self::Matrix => vec![
                CredentialField { name: "Homeserver",   placeholder: "https://matrix.example.com", is_secret: false },
                CredentialField { name: "Access Token", placeholder: "syt_...",                    is_secret: true  },
            ],
            Self::Discord => vec![
                CredentialField { name: "Bot Token", placeholder: "MTx....", is_secret: true },
            ],
            Self::RocketChat => vec![
                CredentialField { name: "Server URL", placeholder: "https://chat.example.com", is_secret: false },
                CredentialField { name: "Username",   placeholder: "admin_bot",                is_secret: false },
                CredentialField { name: "Password",   placeholder: "...",                      is_secret: true  },
            ],
            Self::Mattermost => vec![
                CredentialField { name: "Server URL",  placeholder: "https://mattermost.example.com", is_secret: false },
                CredentialField { name: "Bot Token",   placeholder: "token...",                       is_secret: true  },
            ],
            Self::Slack => vec![
                CredentialField { name: "Bot Token",     placeholder: "xoxb-...", is_secret: true  },
                CredentialField { name: "App Token",     placeholder: "xapp-...", is_secret: true  },
            ],
            Self::XMPP => vec![
                CredentialField { name: "JID",      placeholder: "bot@example.com", is_secret: false },
                CredentialField { name: "Password", placeholder: "...",             is_secret: true  },
            ],
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Matrix"     => Self::Matrix,
            "Discord"    => Self::Discord,
            "RocketChat" | "Rocket.Chat" => Self::RocketChat,
            "Mattermost" => Self::Mattermost,
            "Slack"      => Self::Slack,
            "XMPP"       => Self::XMPP,
            _            => Self::Telegram,
        }
    }
}

// ── ControlBotAccount ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlBotAccount {
    pub id:          String,
    pub platform:    Platform,
    pub label:       String,
    pub credentials: Vec<(String, String)>, // (field_name, value)
    pub connected:   bool,
}

// ── ControlBotConfig ──────────────────────────────────────────────────────────

#[derive(Default, Serialize, Deserialize)]
pub struct ControlBotConfig {
    #[serde(default)]
    pub accounts: Vec<ControlBotAccount>,
}

impl ControlBotConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("control_bot.toml")
    }

    pub fn load() -> Vec<ControlBotAccount> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let cfg: Self = toml::from_str(&content).unwrap_or_default();
        cfg.accounts
    }

    pub fn save(accounts: &[ControlBotAccount]) -> Result<(), String> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let cfg = Self { accounts: accounts.to_vec() };
        let content = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

// ── MessagingBot ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelTarget {
    pub platform: String,
    pub name:     String,
    pub id:       String,
    pub enabled:  bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BroadcastRecord {
    pub message:      String,
    pub sent_at:      DateTime<Utc>,
    pub target_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id:            String,
    pub username:      String,
    pub platform:      String,
    pub waiting_since: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessagingBot {
    pub id:      String,
    pub name:    String,
    pub kind:    BotKind,
    pub enabled: bool,
    pub targets: Vec<ChannelTarget>,
    #[serde(default)]
    pub recent_broadcasts: Vec<BroadcastRecord>,
    #[serde(skip)]
    pub pending_approvals: Vec<PendingApproval>,
}

impl MessagingBot {
    pub fn demo_bots() -> Vec<Self> {
        vec![
            Self {
                id: "broadcast".into(),
                name: "Broadcast Bot".into(),
                kind: BotKind::Broadcast,
                enabled: true,
                targets: vec![
                    ChannelTarget { platform: "Telegram".into(), name: "FreeSynergy Community".into(), id: "-100123".into(), enabled: true },
                    ChannelTarget { platform: "Matrix".into(),   name: "#general:example.com".into(),  id: "!abc:example.com".into(), enabled: true },
                ],
                recent_broadcasts: vec![
                    BroadcastRecord {
                        message: "New update available…".into(),
                        sent_at: Utc::now() - chrono::Duration::hours(2),
                        target_count: 4,
                    },
                ],
                pending_approvals: vec![],
            },
            Self {
                id: "gatekeeper".into(),
                name: "Gatekeeper Bot".into(),
                kind: BotKind::Gatekeeper,
                enabled: true,
                targets: vec![
                    ChannelTarget { platform: "Telegram".into(), name: "FreeSynergy Community".into(), id: "-100123".into(), enabled: true },
                ],
                recent_broadcasts: vec![],
                pending_approvals: vec![
                    PendingApproval {
                        id: "1".into(),
                        username: "@alice_t".into(),
                        platform: "Telegram".into(),
                        waiting_since: Utc::now() - chrono::Duration::minutes(2),
                    },
                    PendingApproval {
                        id: "2".into(),
                        username: "@bob_99".into(),
                        platform: "Telegram".into(),
                        waiting_since: Utc::now() - chrono::Duration::minutes(4),
                    },
                ],
            },
        ]
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct MessagingBotsConfig {
    #[serde(default)]
    pub bots: Vec<MessagingBot>,
}

impl MessagingBotsConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("messaging_bots.toml")
    }

    pub fn load() -> Vec<MessagingBot> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let cfg: Self = toml::from_str(&content).unwrap_or_default();
        if cfg.bots.is_empty() { MessagingBot::demo_bots() } else { cfg.bots }
    }

    pub fn save(bots: &[MessagingBot]) -> Result<(), String> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let cfg = Self { bots: bots.to_vec() };
        let content = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}
