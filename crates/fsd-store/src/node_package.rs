/// NodePackage — package type for the FreeSynergy Node.Store catalog.
use fsn_store::manifest::Manifest;
use serde::{Deserialize, Serialize};

/// Kind of store package — used for type-filter tabs in the browser.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    #[default]
    Plugin,
    Language,
    Theme,
    Widget,
    BotCommand,
    Bridge,
}

impl PackageKind {
    /// All selectable kinds in order.
    pub const ALL: &'static [PackageKind] = &[
        PackageKind::Plugin,
        PackageKind::Language,
        PackageKind::Theme,
        PackageKind::Widget,
        PackageKind::BotCommand,
        PackageKind::Bridge,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            PackageKind::Plugin     => "Plugin",
            PackageKind::Language   => "Language",
            PackageKind::Theme      => "Theme",
            PackageKind::Widget     => "Widget",
            PackageKind::BotCommand => "Bot Command",
            PackageKind::Bridge     => "Bridge",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PackageKind::Plugin     => "🔌",
            PackageKind::Language   => "🌐",
            PackageKind::Theme      => "🎨",
            PackageKind::Widget     => "🧩",
            PackageKind::BotCommand => "🤖",
            PackageKind::Bridge     => "🌉",
        }
    }
}

/// A package entry in the `Node/catalog.toml`.
///
/// Extends `PackageMeta` with Node-Store-specific fields (`icon`, `path`, `kind`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePackage {
    pub id:          String,
    pub name:        String,
    pub version:     String,
    pub category:    String,
    pub description: String,
    #[serde(default)]
    pub license:     String,
    #[serde(default)]
    pub author:      String,
    #[serde(default)]
    pub tags:        Vec<String>,
    #[serde(default)]
    pub kind:         PackageKind,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub icon:         Option<String>,
    /// Store-relative path to the module directory.
    #[serde(default)]
    pub path:         Option<String>,
}

impl Manifest for NodePackage {
    fn id(&self)       -> &str { &self.id }
    fn version(&self)  -> &str { &self.version }
    fn category(&self) -> &str { &self.category }
    fn name(&self)     -> &str { &self.name }
}
