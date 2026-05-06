use super::environment::RuntimeEnvironment;

/// Environment variable for the small local Ollama model.
pub(crate) const FAST_OLLAMA_MODEL_ENV: &str = "OLLAMA_FAST_MODEL";
pub(crate) const CLAUDE_CODE_MODEL_ENV: &str = "CLAUDE_CODE_MODEL";
pub(crate) const CODEX_MODEL_ENV: &str = "CODEX_MODEL";

/// Default fast local model.
///
/// This defaults to the same installed Llama 3 tag used by the primary local
/// fallback, so short prompts work on a fresh setup. Set `OLLAMA_FAST_MODEL` to
/// a smaller installed model if you want lower latency.
pub(crate) const DEFAULT_FAST_OLLAMA_MODEL: &str = "llama3:latest";
pub(crate) const DEFAULT_CLAUDE_CODE_MODEL: &str = "claude-sonnet-4-20250514";
pub(crate) const DEFAULT_CODEX_MODEL: &str = "codex";

#[derive(Clone, Debug)]
pub(crate) struct RuntimeConfig {
    models: ModelRuntimeConfig,
}

impl RuntimeConfig {
    pub(super) fn from_environment<E>(environment: &E) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            models: ModelRuntimeConfig::from_environment(environment),
        }
    }

    pub(crate) fn models(&self) -> &ModelRuntimeConfig {
        &self.models
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ModelRuntimeConfig {
    fast_ollama_model: String,
    claude_code: TerminalAppRuntimeConfig,
    codex: TerminalAppRuntimeConfig,
}

impl ModelRuntimeConfig {
    fn from_environment<E>(environment: &E) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            fast_ollama_model: environment
                .var(FAST_OLLAMA_MODEL_ENV)
                .unwrap_or_else(|| DEFAULT_FAST_OLLAMA_MODEL.to_string()),
            claude_code: TerminalAppRuntimeConfig::from_environment(
                environment,
                CLAUDE_CODE_MODEL_ENV,
                DEFAULT_CLAUDE_CODE_MODEL,
            ),
            codex: TerminalAppRuntimeConfig::from_environment(
                environment,
                CODEX_MODEL_ENV,
                DEFAULT_CODEX_MODEL,
            ),
        }
    }

    pub(crate) fn fast_ollama_model(&self) -> &str {
        &self.fast_ollama_model
    }

    pub(crate) fn claude_code(&self) -> &TerminalAppRuntimeConfig {
        &self.claude_code
    }

    pub(crate) fn codex(&self) -> &TerminalAppRuntimeConfig {
        &self.codex
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TerminalAppRuntimeConfig {
    model_name: String,
}

impl TerminalAppRuntimeConfig {
    fn from_environment<E>(
        environment: &E,
        model_env: &'static str,
        default_model: &'static str,
    ) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            model_name: environment
                .var(model_env)
                .unwrap_or_else(|| default_model.to_string()),
        }
    }

    pub(crate) fn model_name(&self) -> &str {
        &self.model_name
    }
}
