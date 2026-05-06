use crate::{llm::LanguageModel, runtime::ModelRuntimeConfig};

use super::ModelRouter;

/// Primary local model.
pub const PRIMARY_OLLAMA_MODEL: &str = "llama3";

impl ModelRouter {
    /// Create the router with all supported backends.
    ///
    /// Claude Code and Codex are local terminal apps, so the router can choose
    /// them without provider credential checks.
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
                LanguageModel::claude_code(
                    config.claude_code().model_name(),
                    &[
                        "Claude Code terminal app",
                        "deep coding",
                        "long structured work",
                    ],
                    true,
                    None,
                ),
                LanguageModel::codex(
                    config.codex().model_name(),
                    &[
                        "Codex terminal app",
                        "general coding",
                        "implementation work",
                    ],
                    true,
                    None,
                ),
            ],
        }
    }
}
