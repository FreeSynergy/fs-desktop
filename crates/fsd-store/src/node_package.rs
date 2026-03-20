/// NodePackage — package type for the FreeSynergy Node.Store catalog.
use fsn_store::manifest::Manifest;
use serde::{Deserialize, Serialize};

/// Kind of store package — used for type-filter tabs in the browser.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    #[default]
    App,
    /// A Podman/Quadlet container app (e.g. Kanidm, Forgejo, Outline).
    Container,
    /// Built-in desktop manager (Language, Theme, Icons, ContainerApp, Bots).
    Manager,
    Language,
    Theme,
    Widget,
    BotCommand,
    /// A Store package implementing the Channel trait for one messenger platform.
    MessengerAdapter,
    Bridge,
    Task,
    /// A bundle of multiple packages installed together.
    Bundle,
}

impl PackageKind {
    /// All selectable kinds in order.
    pub const ALL: &'static [PackageKind] = &[
        PackageKind::App,
        PackageKind::Container,
        PackageKind::Manager,
        PackageKind::Language,
        PackageKind::Theme,
        PackageKind::Widget,
        PackageKind::BotCommand,
        PackageKind::MessengerAdapter,
        PackageKind::Bridge,
        PackageKind::Task,
        PackageKind::Bundle,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            PackageKind::App             => "App",
            PackageKind::Container       => "Container-App",
            PackageKind::Manager         => "Manager",
            PackageKind::Language        => "Language",
            PackageKind::Theme           => "Theme",
            PackageKind::Widget          => "Widget",
            PackageKind::BotCommand      => "Bot Command",
            PackageKind::MessengerAdapter => "Messenger Adapter",
            PackageKind::Bridge          => "Bridge",
            PackageKind::Task            => "Task",
            PackageKind::Bundle          => "Bundle",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PackageKind::App        => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 9h6"/><path d="M9 12h6"/><path d="M9 15h4"/></svg>"#,
            PackageKind::Container  => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="4" rx="1"/><rect x="2" y="10" width="20" height="4" rx="1"/><rect x="2" y="17" width="20" height="4" rx="1"/><circle cx="6" cy="5" r="1" fill="currentColor"/><circle cx="6" cy="12" r="1" fill="currentColor"/><circle cx="6" cy="19" r="1" fill="currentColor"/></svg>"#,
            PackageKind::Manager    => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>"#,
            PackageKind::Language   => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>"#,
            PackageKind::Theme      => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r="0.5" fill="currentColor"/><circle cx="17.5" cy="10.5" r="0.5" fill="currentColor"/><circle cx="8.5" cy="7.5" r="0.5" fill="currentColor"/><circle cx="6.5" cy="12.5" r="0.5" fill="currentColor"/><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10c.19 0 .37-.01.56-.02a1 1 0 0 0 .94-1V19a2 2 0 0 1 2-2h3a2 2 0 0 0 2-2v-1c0-5.52-4.48-10-10-10z"/></svg>"#,
            PackageKind::Widget     => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>"#,
            PackageKind::BotCommand       => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M12 11V3"/><circle cx="12" cy="3" r="1" fill="currentColor"/><line x1="8" y1="16" x2="8" y2="16" stroke-width="3"/><line x1="16" y1="16" x2="16" y2="16" stroke-width="3"/><path d="M9 20h6"/></svg>"#,
            PackageKind::MessengerAdapter => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/><path d="M8 10h8"/><path d="M8 14h5"/></svg>"#,
            PackageKind::Bridge           => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>"#,
            PackageKind::Task       => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><polyline points="3 6 4 7 6 5"/><polyline points="3 12 4 13 6 11"/><polyline points="3 18 4 19 6 17"/></svg>"#,
            PackageKind::Bundle     => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>"#,
        }
    }

    /// Lowercase string key for registry storage.
    pub fn kind_str(&self) -> String {
        match self {
            PackageKind::App              => "app".into(),
            PackageKind::Container        => "container".into(),
            PackageKind::Manager          => "manager".into(),
            PackageKind::Language         => "language".into(),
            PackageKind::Theme            => "theme".into(),
            PackageKind::Widget           => "widget".into(),
            PackageKind::BotCommand       => "bot".into(),
            PackageKind::MessengerAdapter => "messenger-adapter".into(),
            PackageKind::Bridge           => "bridge".into(),
            PackageKind::Task             => "task".into(),
            PackageKind::Bundle           => "bundle".into(),
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
