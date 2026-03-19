/// Built-in app and manager registry — pre-registers built-in packages at startup.
///
/// The Store is always present as the entry point for installing everything else.
/// The five built-in managers (Language, Theme, Icons, ContainerApp, Bots) are
/// desktop components that exist without installation.
///
/// Built-in items are registered as PackageRegistry entries so that:
///  - they appear in the sidebar (only registered packages are shown)
///  - they show up as "Installed" in the Store browser
///
/// Call `ensure_registered()` once at startup — it is idempotent.

use fsd_db::package_registry::{InstalledPackage, PackageRegistry};

/// Metadata for one built-in package.
struct BuiltinPkg {
    id:      &'static str,
    name:    &'static str,
    kind:    &'static str,
    icon:    &'static str,
    version: &'static str,
}

// ── Icons ─────────────────────────────────────────────────────────────────────

const ICON_STORE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"/><line x1="3" y1="6" x2="21" y2="6"/><path d="M16 10a4 4 0 0 1-8 0"/></svg>"#;

/// Globe — Language Manager
const ICON_LANGUAGE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>"#;

/// Palette — Theme Manager
const ICON_THEME: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125A1.64 1.64 0 0 1 14.441 18h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/><circle cx="8" cy="8" r="1" fill="currentColor" stroke="none"/><circle cx="12" cy="6" r="1" fill="currentColor" stroke="none"/><circle cx="16" cy="9" r="1" fill="currentColor" stroke="none"/></svg>"#;

/// Image frame — Icons Manager
const ICON_ICONS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>"#;

/// 3-D box — Container App Manager
const ICON_CONTAINER: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>"#;

/// CPU chip — Bots Manager
const ICON_BOTS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>"#;

// ── Registry ──────────────────────────────────────────────────────────────────

const BUILTIN_PKGS: &[BuiltinPkg] = &[
    // Built-in app — always the entry point
    BuiltinPkg { id: "store",            name: "Store",             kind: "app",     icon: ICON_STORE,     version: env!("CARGO_PKG_VERSION") },
    // Built-in managers — IDs must match AppWindowContent cases ("app-{id}")
    BuiltinPkg { id: "language-manager", name: "Language Manager",  kind: "manager", icon: ICON_LANGUAGE,  version: env!("CARGO_PKG_VERSION") },
    BuiltinPkg { id: "theme-manager",    name: "Theme Manager",     kind: "manager", icon: ICON_THEME,     version: env!("CARGO_PKG_VERSION") },
    BuiltinPkg { id: "icons-manager",    name: "Icons Manager",     kind: "manager", icon: ICON_ICONS,     version: env!("CARGO_PKG_VERSION") },
    BuiltinPkg { id: "container",        name: "Container Manager", kind: "manager", icon: ICON_CONTAINER, version: env!("CARGO_PKG_VERSION") },
    BuiltinPkg { id: "bot-manager",      name: "Bots Manager",      kind: "manager", icon: ICON_BOTS,      version: env!("CARGO_PKG_VERSION") },
];

/// Old IDs that were renamed — remove these on startup to avoid stale sidebar entries.
const LEGACY_IDS: &[&str] = &[
    "manager-language",
    "manager-theme",
    "manager-icons",
    "manager-container-app",
    "manager-bots",
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

    for pkg in BUILTIN_PKGS {
        if !PackageRegistry::is_installed(pkg.id) {
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
}
