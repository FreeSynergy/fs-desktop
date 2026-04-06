//! `CapabilityObserver` — detects runtime service capabilities.
//!
//! Design Pattern: Observer (stateful capability detection)
//!
//! For G1.5 the check is synchronous and reads the `FS_AI_CAPABILITY`
//! environment variable.  Full gRPC wiring to `fs-registry` is planned for G1.7.

/// Detects which optional capabilities are available at runtime.
///
/// Absent capabilities cause their UI elements to be hidden.  Example: the AI
/// corner menu is shown only when `ai_chat_available()` returns `true`.
pub struct CapabilityObserver {
    ai_chat: bool,
}

impl CapabilityObserver {
    /// Probe available capabilities on startup.
    ///
    /// Current heuristic: reads `FS_AI_CAPABILITY` env var.  When `fs-registry`
    /// gRPC is available (G1.7) this will perform a real capability lookup.
    #[must_use]
    pub fn detect() -> Self {
        let ai_chat = std::env::var("FS_AI_CAPABILITY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        Self { ai_chat }
    }

    /// Returns `true` when an `ai.chat` service is registered.
    #[must_use]
    pub fn ai_chat_available(&self) -> bool {
        self.ai_chat
    }

    /// Update the `ai.chat` availability flag.
    ///
    /// Called when a `DesktopMessage::AiCapabilityChanged` event arrives.
    pub fn set_ai_chat(&mut self, available: bool) {
        self.ai_chat = available;
    }
}

impl Default for CapabilityObserver {
    fn default() -> Self {
        Self::detect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_unavailable_when_env_missing() {
        // Remove the env var so the test is deterministic.
        std::env::remove_var("FS_AI_CAPABILITY");
        let obs = CapabilityObserver::detect();
        assert!(!obs.ai_chat_available());
    }

    #[test]
    fn set_ai_chat_updates_flag() {
        let mut obs = CapabilityObserver { ai_chat: false };
        obs.set_ai_chat(true);
        assert!(obs.ai_chat_available());
        obs.set_ai_chat(false);
        assert!(!obs.ai_chat_available());
    }
}
