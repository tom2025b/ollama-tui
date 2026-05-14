use crate::llm::LanguageModel;
use crate::runtime::ModelRuntimeConfig;

use super::ModelRouter;

/// Primary local model name.
pub const PRIMARY_OLLAMA_MODEL: &str = "llama3";

impl ModelRouter {
    /// Create the router with the configured Ollama models.
    pub(crate) fn new(config: &ModelRuntimeConfig) -> Self {
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
                    config.fast_ollama_model(),
                    &["fast local model", "short/simple prompts", "low latency"],
                ),
            ],
        }
    }
}
