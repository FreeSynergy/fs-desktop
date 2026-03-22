//! Package registry — tracks Store-installed packages in `~/.local/share/fsn/packages.json`.
//!
//! Shared persistence layer for languages, themes, widgets and other packages
//! installed from the FreeSynergy Store. Uses plain JSON so every program
//! (fs-store, fs-settings, fs-shell) can read/write it without migrations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single installed package entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Package identifier — matches the catalog ID.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Package kind: `"language"`, `"theme"`, `"widget"`, `"plugin"`, `"app"`, etc.
    pub kind: String,
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

    /// Returns all packages of a given kind (`"language"`, `"theme"`, etc.).
    pub fn by_kind(kind: &str) -> Vec<InstalledPackage> {
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
