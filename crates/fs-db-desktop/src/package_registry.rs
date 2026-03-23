//! Package registry — tracks Store-installed packages in `~/.local/share/fsn/packages.json`.
//!
//! Shared persistence layer for languages, themes, widgets and other packages
//! installed from the FreeSynergy Store. Uses plain JSON so every program
//! (fs-store, fs-settings, fs-gui-workspace) can read/write it without migrations.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Kind of a FreeSynergy package — used in both the store catalog and the local registry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    #[default]
    App,
    /// A Podman/Quadlet container app (e.g. Kanidm, Forgejo, Outline).
    Container,
    /// Built-in desktop manager (Language, Theme, Icons, Container, Bots).
    Manager,
    Language,
    Theme,
    Widget,
    #[serde(rename = "bot")]
    BotCommand,
    /// A Store package implementing the Channel trait for one messenger platform.
    #[serde(rename = "messenger-adapter")]
    MessengerAdapter,
    Bridge,
    Task,
    /// A native binary service (non-container, e.g. Mistral.rs).
    Binary,
    /// A bundle of multiple packages installed together.
    Bundle,
}

impl PackageKind {
    /// All selectable kinds in order.
    pub const ALL: &'static [PackageKind] = &[
        PackageKind::App,
        PackageKind::Container,
        PackageKind::Binary,
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
            PackageKind::App              => "App",
            PackageKind::Container        => "Container-App",
            PackageKind::Binary           => "Binary",
            PackageKind::Manager          => "Manager",
            PackageKind::Language         => "Language",
            PackageKind::Theme            => "Theme",
            PackageKind::Widget           => "Widget",
            PackageKind::BotCommand       => "Bot Command",
            PackageKind::MessengerAdapter => "Messenger Adapter",
            PackageKind::Bridge           => "Bridge",
            PackageKind::Task             => "Task",
            PackageKind::Bundle           => "Bundle",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PackageKind::App        => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 9h6"/><path d="M9 12h6"/><path d="M9 15h4"/></svg>"#,
            PackageKind::Container  => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="4" rx="1"/><rect x="2" y="10" width="20" height="4" rx="1"/><rect x="2" y="17" width="20" height="4" rx="1"/><circle cx="6" cy="5" r="1" fill="currentColor"/><circle cx="6" cy="12" r="1" fill="currentColor"/><circle cx="6" cy="19" r="1" fill="currentColor"/></svg>"#,
            PackageKind::Binary     => r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6l4-2 4 2 4-2 4 2"/><path d="M4 12l4-2 4 2 4-2 4 2"/><path d="M4 18l4-2 4 2 4-2 4 2"/></svg>"#,
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
}

impl fmt::Display for PackageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PackageKind::App              => "app",
            PackageKind::Container        => "container",
            PackageKind::Binary           => "binary",
            PackageKind::Manager          => "manager",
            PackageKind::Language         => "language",
            PackageKind::Theme            => "theme",
            PackageKind::Widget           => "widget",
            PackageKind::BotCommand       => "bot",
            PackageKind::MessengerAdapter => "messenger-adapter",
            PackageKind::Bridge           => "bridge",
            PackageKind::Task             => "task",
            PackageKind::Bundle           => "bundle",
        })
    }
}

/// A single installed package entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Package identifier — matches the catalog ID.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Package kind.
    pub kind: PackageKind,
    /// Installed version (SemVer).
    pub version: String,
    /// Emoji or icon identifier for sidebar display (e.g. `"🌐"`).
    #[serde(default)]
    pub icon: String,
    /// Absolute path to the downloaded file on disk, or `None` if no file was saved.
    pub file_path: Option<String>,
    /// If set: this package was installed as a member of a bundle.
    /// Contains the bundle's package ID. The package cannot be removed individually.
    #[serde(default)]
    pub installed_by: Option<String>,
    /// If `true`, this package is pinned in the sidebar's pinned section.
    #[serde(default)]
    pub pinned: bool,
}

/// Simple JSON-based registry at `~/.local/share/fsn/packages.json`.
///
/// All methods are synchronous and operate on the file directly — suitable
/// for use in Dioxus component initializers and `tokio::task::spawn_blocking`.
pub struct PackageRegistry;

impl PackageRegistry {
    fn registry_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".local/share/fsn/packages.json")
    }

    /// Load all installed packages from disk.
    pub fn load() -> Vec<InstalledPackage> {
        let path = Self::registry_path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    }

    /// Returns `true` if a package with `id` is registered.
    pub fn is_installed(id: &str) -> bool {
        Self::load().iter().any(|p| p.id == id)
    }

    /// Returns all packages of a given kind.
    pub fn by_kind(kind: PackageKind) -> Vec<InstalledPackage> {
        Self::load().into_iter().filter(|p| p.kind == kind).collect()
    }

    /// Register (or update) a package. Upserts by ID.
    pub fn install(pkg: InstalledPackage) -> Result<(), String> {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut packages = Self::load();
        packages.retain(|p| p.id != pkg.id);
        packages.push(pkg);
        let json = serde_json::to_string_pretty(&packages).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Remove a package by ID. Also removes the downloaded file from disk if present.
    pub fn remove(id: &str) -> Result<(), String> {
        let path = Self::registry_path();
        let mut packages = Self::load();
        // Delete local file if it exists
        if let Some(pkg) = packages.iter().find(|p| p.id == id) {
            if let Some(ref file) = pkg.file_path {
                let _ = std::fs::remove_file(file);
            }
        }
        packages.retain(|p| p.id != id);
        let json = serde_json::to_string_pretty(&packages).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Toggle the pinned state of a package by ID.
    pub fn set_pinned(id: &str, pinned: bool) -> Result<(), String> {
        let path = Self::registry_path();
        let mut packages = Self::load();
        if let Some(pkg) = packages.iter_mut().find(|p| p.id == id) {
            pkg.pinned = pinned;
        }
        let json = serde_json::to_string_pretty(&packages).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Remove a bundle and all packages that were installed as members of that bundle.
    pub fn remove_bundle(bundle_id: &str) -> Result<(), String> {
        let path = Self::registry_path();
        let mut packages = Self::load();
        // Delete local files for the bundle and all its members
        for pkg in packages.iter().filter(|p| p.id == bundle_id || p.installed_by.as_deref() == Some(bundle_id)) {
            if let Some(ref file) = pkg.file_path {
                let _ = std::fs::remove_file(file);
            }
        }
        packages.retain(|p| p.id != bundle_id && p.installed_by.as_deref() != Some(bundle_id));
        let json = serde_json::to_string_pretty(&packages).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }
}
