/// SystemContext — OOP source of truth for the running environment.
///
/// Knows what platform, architecture, and operating mode the current instance
/// is running in. Initialized once (via `SystemInfo::detect()`) and exposed
/// as a `GlobalSignal` so any component can read it.
///
/// When Init or Node starts, they call `SYSTEM_INFO.write().set_mode(...)` to
/// inform all other components about the full system context.
use dioxus::prelude::*;

// ── Platform ─────────────────────────────────────────────────────────────────

/// The host operating system.
#[derive(Clone, Debug, PartialEq)]
pub enum Platform {
    Linux,
    MacOs,
    Windows,
    Unknown,
}

impl Platform {
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]   { Platform::Linux }
        #[cfg(target_os = "macos")]   { Platform::MacOs }
        #[cfg(target_os = "windows")] { Platform::Windows }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        { Platform::Unknown }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Linux   => "Linux",
            Self::MacOs   => "macOS",
            Self::Windows => "Windows",
            Self::Unknown => "Unknown",
        }
    }

    /// True if the platform supports server packages (Container Apps, etc.).
    pub fn supports_server_packages(&self) -> bool {
        matches!(self, Self::Linux)
    }
}

// ── Architecture ─────────────────────────────────────────────────────────────

/// CPU architecture.
#[derive(Clone, Debug, PartialEq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Unknown,
}

impl Architecture {
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]  { Architecture::X86_64 }
        #[cfg(target_arch = "aarch64")] { Architecture::Aarch64 }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        { Architecture::Unknown }
    }

    pub fn label(&self) -> &'static str {
        match self { Self::X86_64 => "x86-64", Self::Aarch64 => "ARM64", Self::Unknown => "?" }
    }
}

// ── RunMode ──────────────────────────────────────────────────────────────────

/// How the current instance is operating.
///
/// Set by Init/Node after startup. Defaults to `DesktopOnly` until the
/// running Node reports its presence via `SYSTEM_INFO.write().set_mode(...)`.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum RunMode {
    /// Desktop with a connected local Node (server + desktop).
    Node,
    /// Desktop only — no local Node running.
    #[default]
    DesktopOnly,
    /// Headless server (Node without a desktop UI).
    Server,
}

// ── SystemInfo ───────────────────────────────────────────────────────────────

/// Complete description of the running system.
///
/// Available globally via [`SYSTEM_INFO`]. Any component or service that needs
/// to know "are we on a server?", "do we have a Node?", or "what OS is this?"
/// reads from this struct instead of running OS detection themselves.
///
/// # Typical usage
/// ```rust
/// let sys = SYSTEM_INFO.read();
/// if sys.can_install_server_packages() {
///     // show container-app install button
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SystemInfo {
    pub platform:     Platform,
    pub architecture: Architecture,
    pub mode:         RunMode,
    pub hostname:     String,
    /// Version string of the running FSN Node, if connected.
    pub node_version: Option<String>,
}

impl SystemInfo {
    /// Detect the current system at startup.
    pub fn detect() -> Self {
        Self {
            platform:     Platform::detect(),
            architecture: Architecture::detect(),
            mode:         RunMode::default(),
            hostname:     detect_hostname(),
            node_version: None,
        }
    }

    /// Called by the Node process after it starts to register its presence.
    pub fn set_mode(&mut self, mode: RunMode) {
        self.mode = mode;
    }

    /// Called by the Node process to register its version.
    pub fn set_node_version(&mut self, version: impl Into<String>) {
        self.node_version = Some(version.into());
        if self.mode == RunMode::DesktopOnly {
            self.mode = RunMode::Node;
        }
    }

    /// True if a local Node is running (server + desktop mode).
    pub fn has_node(&self) -> bool {
        matches!(self.mode, RunMode::Node)
    }

    /// True if this instance can install and run server packages.
    ///
    /// Requires: Linux platform + Node running.
    pub fn can_install_server_packages(&self) -> bool {
        self.platform.supports_server_packages() && self.has_node()
    }

    /// Short human-readable description of the current mode.
    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            RunMode::Node        => "Node + Desktop",
            RunMode::DesktopOnly => "Desktop",
            RunMode::Server      => "Server (headless)",
        }
    }
}

fn detect_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .or_else(|_| std::env::var("HOSTNAME"))
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

// ── Global signal ─────────────────────────────────────────────────────────────

/// Global system context — readable from any component.
///
/// Writable by Init/Node to set the run mode after startup.
pub static SYSTEM_INFO: GlobalSignal<SystemInfo> = Signal::global(SystemInfo::detect);
