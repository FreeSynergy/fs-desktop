/// Built-in app registry — pre-registers the Store at startup.
///
/// The Store is always present as the entry point for installing everything else.
/// Managers are NOT pre-registered — they must be installed explicitly via the Store,
/// just like any other package. This keeps the sidebar clean on fresh installs.
///
/// Call `ensure_registered()` once at startup — it is idempotent.

use fsd_db::package_registry::{InstalledPackage, PackageRegistry};
use crate::icons::ICON_STORE;

/// Metadata for one built-in package.
struct BuiltinPkg {
    id:      &'static str,
    name:    &'static str,
    kind:    &'static str,
    icon:    &'static str,
    version: &'static str,
}

// ── Registry ──────────────────────────────────────────────────────────────────

const BUILTIN_PKGS: &[BuiltinPkg] = &[
    // The Store is the only truly built-in entry point — always present.
    BuiltinPkg { id: "store", name: "Store", kind: "app", icon: ICON_STORE, version: env!("CARGO_PKG_VERSION") },
];

/// IDs to remove on startup — renamed entries and formerly auto-registered managers.
/// Managers are no longer built-in; they must be installed explicitly via the Store.
const LEGACY_IDS: &[&str] = &[
    // Old renamed IDs
    "manager-language",
    "manager-theme",
    "manager-icons",
    "manager-container-app",
    "manager-bots",
    // Previously auto-registered managers — remove so user must install explicitly
    "language-manager",
    "theme-manager",
    "icons-manager",
    "container",
    "bot-manager",
];

/// Pre-registers all built-in packages in the PackageRegistry (idempotent).
/// Should be called once at Desktop startup before the sidebar is rendered.
pub fn ensure_registered() {
    // Remove stale legacy entries so renamed IDs don't produce duplicates.
    for id in LEGACY_IDS {
        if PackageRegistry::is_installed(id) {
            let _ = PackageRegistry::remove(id);
        }
    }

    // Remove any non-builtin "app" entries whose binary no longer exists.
    // These are stale entries from previous auto-registration code. Keeps the
    // sidebar clean: only genuinely installed apps (with a file on disk) and
    // built-in apps (which are part of the Desktop binary) are shown.
    let builtin_ids: std::collections::HashSet<&str> =
        BUILTIN_PKGS.iter().map(|p| p.id).collect();
    for pkg in PackageRegistry::load() {
        if pkg.kind == "app" && !builtin_ids.contains(pkg.id.as_str()) {
            // Only remove if a binary path was recorded but the file is now gone (stale).
            // If file_path is None the package was installed without a local binary
            // (production stub, container-backed, etc.) — keep it.
            let has_binary = pkg.file_path.as_ref()
                .map_or(true, |p| std::path::Path::new(p).exists());
            if !has_binary {
                let _ = PackageRegistry::remove(&pkg.id);
            }
        }
    }

    // Always upsert built-in packages so icon changes in icons.rs are propagated
    // to packages.json on the next startup — icons are never lost or stale.
    for pkg in BUILTIN_PKGS {
        let entry = InstalledPackage {
            id:           pkg.id.to_string(),
            name:         pkg.name.to_string(),
            kind:         pkg.kind.to_string(),
            version:      pkg.version.to_string(),
            icon:         pkg.icon.to_string(),
            file_path:    None,
            installed_by: None,
        };
        let _ = PackageRegistry::install(entry);
    }
}
