/// NodePackage — package type for the FreeSynergy Node.Store catalog.
use fsn_store::manifest::Manifest;
use serde::{Deserialize, Serialize};

/// Kind of store package — used for type-filter tabs in the browser.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    #[default]
    Plugin,
    /// A Podman/Quadlet container service (e.g. Kanidm, Forgejo, Outline).
    Container,
    Language,
    Theme,
    Widget,
    BotCommand,
    Bridge,
    Task,
}

impl PackageKind {
    /// All selectable kinds in order.
    pub const ALL: &'static [PackageKind] = &[
        PackageKind::Plugin,
        PackageKind::Container,
        PackageKind::Language,
        PackageKind::Theme,
        PackageKind::Widget,
        PackageKind::BotCommand,
        PackageKind::Bridge,
        PackageKind::Task,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            PackageKind::Plugin     => "Plugin",
            PackageKind::Container  => "Service",
            PackageKind::Language   => "Language",
            PackageKind::Theme      => "Theme",
            PackageKind::Widget     => "Widget",
            PackageKind::BotCommand => "Bot Command",
            PackageKind::Bridge     => "Bridge",
            PackageKind::Task       => "Task",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PackageKind::Plugin     => "🔌",
            PackageKind::Container  => "📦",
            PackageKind::Language   => "🌐",
            PackageKind::Theme      => "🎨",
            PackageKind::Widget     => "🧩",
            PackageKind::BotCommand => "🤖",
            PackageKind::Bridge     => "🌉",
            PackageKind::Task       => "⚡",
        }
    }

    /// Lowercase string key for registry storage.
    pub fn kind_str(&self) -> String {
        match self {
            PackageKind::Plugin     => "plugin".into(),
            PackageKind::Container  => "container".into(),
            PackageKind::Language   => "language".into(),
            PackageKind::Theme      => "theme".into(),
            PackageKind::Widget     => "widget".into(),
            PackageKind::BotCommand => "bot".into(),
            PackageKind::Bridge     => "bridge".into(),
            PackageKind::Task       => "task".into(),
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
