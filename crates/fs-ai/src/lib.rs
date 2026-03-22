pub mod app;

pub use app::AiManagerApp;

pub fn register_i18n() {
    const EN: &str = include_str!("../assets/i18n/en.toml");
    const DE: &str = include_str!("../assets/i18n/de.toml");
    let _ = fs_i18n::add_toml_lang("en", EN);
    let _ = fs_i18n::add_toml_lang("de", DE);
}

// ── AI status helpers (used by shell / help sidebar) ─────────────────────────

use fs_manager_ai::{AiEngine, LlmConfig, LlmEngine, LlmModel};

fn default_engine() -> LlmEngine {
    LlmEngine::new(
        LlmConfig { model: LlmModel::Qwen3_4B, ..LlmConfig::default() },
        LlmEngine::default_binary(),
        LlmEngine::default_data_dir(),
    )
}

/// Returns `true` if the LLM engine binary is installed.
pub fn is_ai_installed() -> bool {
    default_engine().is_installed()
}

/// Returns the OpenAI-compatible API base URL if the engine is running,
/// e.g. `"http://127.0.0.1:1234/v1"`.
pub fn ai_api_url() -> Option<String> {
    match default_engine().status() {
        fs_manager_ai::EngineStatus::Running { port } =>
            Some(format!("http://127.0.0.1:{port}/v1")),
        _ => None,
    }
}
