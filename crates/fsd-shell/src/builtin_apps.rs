/// Built-in app registry — pre-registers all compiled-in apps in PackageRegistry.
///
/// Built-in apps are compiled into the desktop binary and are always _runnable_,
/// but they still need to be registered as `kind = "app"` entries in the
/// PackageRegistry so that:
///  - they appear in the Store's "Installed" section with proper metadata
///  - the sidebar reads them dynamically (only registered apps are shown)
///  - users can uninstall (hide) or reinstall (re-show) them via the Store
///
/// Call `ensure_registered()` once at startup — it is idempotent (skips
/// apps that are already in the registry).

use fsd_db::package_registry::{InstalledPackage, PackageRegistry};

/// Metadata for one built-in app.
struct BuiltinApp {
    id:      &'static str,
    name:    &'static str,
    icon:    &'static str,
    version: &'static str,
}

const BUILTIN_APPS: &[BuiltinApp] = &[
    BuiltinApp { id: "browser",       name: "Browser",           icon: "🌐", version: env!("CARGO_PKG_VERSION") },
    BuiltinApp { id: "lenses",        name: "Lenses",            icon: "🔍", version: env!("CARGO_PKG_VERSION") },
    BuiltinApp { id: "tasks",         name: "Tasks",             icon: "📋", version: env!("CARGO_PKG_VERSION") },
    BuiltinApp { id: "store",         name: "Store",             icon: "📦", version: env!("CARGO_PKG_VERSION") },
    BuiltinApp { id: "container",     name: "Container Manager", icon: "📦", version: env!("CARGO_PKG_VERSION") },
    BuiltinApp { id: "theme-manager", name: "Theme Manager",     icon: "🎨", version: env!("CARGO_PKG_VERSION") },
    BuiltinApp { id: "bot-manager",   name: "Bot Manager",       icon: "🤖", version: env!("CARGO_PKG_VERSION") },
];

/// Pre-registers all built-in apps in the PackageRegistry (idempotent).
/// Should be called once at Desktop startup before the sidebar is rendered.
pub fn ensure_registered() {
    for app in BUILTIN_APPS {
        if !PackageRegistry::is_installed(app.id) {
            let pkg = InstalledPackage {
                id:        app.id.to_string(),
                name:      app.name.to_string(),
                kind:      "app".to_string(),
                version:   app.version.to_string(),
                icon:      app.icon.to_string(),
                file_path: None,
            };
            // Ignore write errors at startup — the app still runs, just won't persist.
            let _ = PackageRegistry::install(pkg);
        }
    }
}
