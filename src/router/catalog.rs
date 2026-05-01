use std::env;

use crate::{anthropic, llm::LanguageModel, openai, xai};

use super::ModelRouter;

/// Primary local model.
pub const PRIMARY_OLLAMA_MODEL: &str = "llama3";

/// Environment variable for the small local Ollama model.
pub const FAST_OLLAMA_MODEL_ENV: &str = "OLLAMA_FAST_MODEL";

/// Default fast local model.
///
/// This defaults to the same installed Llama 3 tag used by the primary local
/// fallback, so short prompts work on a fresh setup. Set `OLLAMA_FAST_MODEL` to
/// a smaller installed model if you want lower latency.
pub const DEFAULT_FAST_OLLAMA_MODEL: &str = "llama3:latest";

impl ModelRouter {
    /// Create the router with all supported backends.
    ///
    /// Cloud models are listed even when disabled so the TUI can show what must
    /// be configured.
    pub fn new() -> Self {
        let fast_ollama_model = env::var(FAST_OLLAMA_MODEL_ENV)
            .unwrap_or_else(|_| DEFAULT_FAST_OLLAMA_MODEL.to_string());

        Self {
            models: vec![
                LanguageModel::ollama(
                    PRIMARY_OLLAMA_MODEL,
                    &[
                        "primary local model",
                        "private/offline prompts",
                        "reliable fallback",
                    ],
                ),
                LanguageModel::ollama(
                    &fast_ollama_model,
                    &["fast local model", "short/simple prompts", "low latency"],
                ),
                LanguageModel::anthropic(
                    &anthropic::configured_model_name(),
                    &["deep coding", "careful reasoning", "long structured work"],
                    anthropic::is_configured(),
                    disabled_reason(
                        anthropic::is_configured(),
                        anthropic::missing_configuration_reason(),
                    ),
                ),
                LanguageModel::openai(
                    &openai::configured_model_name(),
                    &["balanced cloud model", "general tasks", "creative drafting"],
                    openai::is_configured(),
                    disabled_reason(
                        openai::is_configured(),
                        openai::missing_configuration_reason(),
                    ),
                ),
                LanguageModel::xai(
                    &xai::configured_model_name(),
                    &[
                        "Grok reasoning",
                        "public-discourse questions",
                        "fresh-context style prompts",
                    ],
                    xai::is_configured(),
                    disabled_reason(xai::is_configured(), xai::missing_configuration_reason()),
                ),
            ],
        }
    }
}

fn disabled_reason(enabled: bool, reason: String) -> Option<String> {
    if enabled { None } else { Some(reason) }
}
