// Lens data model — L1.
//
// A Lens is a saved search query. When opened, it queries the bus for data
// from all services that match any of the configured roles, then displays
// results grouped by role.

use serde::{Deserialize, Serialize};

/// The service role a lens item was sourced from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LensRole {
    Wiki,
    Chat,
    Git,
    Map,
    Tasks,
    Iam,
    Other(String),
}

impl LensRole {
    pub fn icon(&self) -> &str {
        match self {
            Self::Wiki  => "📖",
            Self::Chat  => "💬",
            Self::Git   => "🔀",
            Self::Map   => "🗺",
            Self::Tasks => "✅",
            Self::Iam   => "🔑",
            Self::Other(_) => "📄",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Wiki  => fsn_i18n::t("lenses.item.role_wiki"),
            Self::Chat  => fsn_i18n::t("lenses.item.role_chat"),
            Self::Git   => fsn_i18n::t("lenses.item.role_git"),
            Self::Map   => fsn_i18n::t("lenses.item.role_map"),
            Self::Tasks => fsn_i18n::t("lenses.item.role_tasks"),
            Self::Iam   => fsn_i18n::t("lenses.item.role_iam"),
            Self::Other(r) => r.clone(),
        }
    }
}

/// A single result item within a lens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensItem {
    /// The service role this result came from.
    pub role: LensRole,
    /// Short human-readable summary of the result.
    pub summary: String,
    /// URL to open in the browser when the user clicks the item.
    pub link: Option<String>,
    /// Source service instance name.
    pub source: String,
}

/// A saved lens (search config + cached results).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lens {
    /// Unique identifier (timestamp-based).
    pub id: i64,
    /// Human-readable name (e.g. "Helfa Köln").
    pub name: String,
    /// The search query sent to services.
    pub query: String,
    /// Cached results from the last query.
    #[serde(default)]
    pub items: Vec<LensItem>,
    /// ISO 8601 timestamp of the last refresh.
    pub last_refreshed: Option<String>,
    /// Whether a refresh is currently in progress.
    #[serde(skip)]
    pub loading: bool,
}

impl Lens {
    pub fn new(name: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            id:             chrono::Utc::now().timestamp_millis(),
            name:           name.into(),
            query:          query.into(),
            items:          Vec::new(),
            last_refreshed: None,
            loading:        false,
        }
    }

    /// Group items by role for display.
    pub fn grouped(&self) -> Vec<(LensRole, Vec<&LensItem>)> {
        let mut map: std::collections::BTreeMap<String, (LensRole, Vec<&LensItem>)> =
            std::collections::BTreeMap::new();

        for item in &self.items {
            let key = format!("{:?}", item.role);
            map.entry(key)
                .or_insert_with(|| (item.role.clone(), Vec::new()))
                .1.push(item);
        }
        map.into_values().collect()
    }
}
