use crate::{
    llm::LanguageModel,
    runtime::{CloudProviderRuntimeConfig, ModelRuntimeConfig},
};

use super::ModelRouter;

/// Primary local model.
pub const PRIMARY_OLLAMA_MODEL: &str = "llama3";

impl ModelRouter {
    /// Create the router with all supported backends.
    ///
    /// Cloud models are listed even when disabled so the TUI can show what must
    /// be configured.
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
                LanguageModel::anthropic(
                    config.anthropic().model_name(),
                    &["deep coding", "careful reasoning", "long structured work"],
                    config.anthropic().configured(),
                    disabled_reason(config.anthropic()),
                ),
                LanguageModel::openai(
                    config.openai().model_name(),
                    &["balanced cloud model", "general tasks", "creative drafting"],
                    config.openai().configured(),
                    disabled_reason(config.openai()),
                ),
                LanguageModel::xai(
                    config.xai().model_name(),
                    &[
                        "Grok reasoning",
                        "public-discourse questions",
                        "fresh-context style prompts",
                    ],
                    config.xai().configured(),
                    disabled_reason(config.xai()),
                ),
            ],
        }
    }
}

fn disabled_reason(config: &CloudProviderRuntimeConfig) -> Option<String> {
    if config.configured() {
        None
    } else {
        Some(config.missing_configuration_reason().to_string())
    }
}
