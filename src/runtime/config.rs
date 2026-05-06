use crate::providers::{anthropic, openai, xai};

use super::environment::RuntimeEnvironment;

/// Environment variable for the small local Ollama model.
pub(crate) const FAST_OLLAMA_MODEL_ENV: &str = "OLLAMA_FAST_MODEL";

/// Default fast local model.
///
/// This defaults to the same installed Llama 3 tag used by the primary local
/// fallback, so short prompts work on a fresh setup. Set `OLLAMA_FAST_MODEL` to
/// a smaller installed model if you want lower latency.
pub(crate) const DEFAULT_FAST_OLLAMA_MODEL: &str = "llama3:latest";

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
    anthropic: CloudProviderRuntimeConfig,
    openai: CloudProviderRuntimeConfig,
    xai: CloudProviderRuntimeConfig,
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
            anthropic: CloudProviderRuntimeConfig::from_environment(
                environment,
                anthropic::ANTHROPIC_API_KEY_ENV,
                anthropic::ANTHROPIC_MODEL_ENV,
                anthropic::DEFAULT_ANTHROPIC_MODEL,
                "Claude",
            ),
            openai: CloudProviderRuntimeConfig::from_environment(
                environment,
                openai::OPENAI_API_KEY_ENV,
                openai::OPENAI_MODEL_ENV,
                openai::DEFAULT_OPENAI_MODEL,
                "GPT-4o",
            ),
            xai: CloudProviderRuntimeConfig::from_environment(
                environment,
                xai::XAI_API_KEY_ENV,
                xai::XAI_MODEL_ENV,
                xai::DEFAULT_XAI_MODEL,
                "Grok",
            ),
        }
    }

    pub(crate) fn fast_ollama_model(&self) -> &str {
        &self.fast_ollama_model
    }

    pub(crate) fn anthropic(&self) -> &CloudProviderRuntimeConfig {
        &self.anthropic
    }

    pub(crate) fn openai(&self) -> &CloudProviderRuntimeConfig {
        &self.openai
    }

    pub(crate) fn xai(&self) -> &CloudProviderRuntimeConfig {
        &self.xai
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CloudProviderRuntimeConfig {
    model_name: String,
    configured: bool,
    missing_configuration_reason: String,
}

impl CloudProviderRuntimeConfig {
    fn from_environment<E>(
        environment: &E,
        api_key_env: &'static str,
        model_env: &'static str,
        default_model: &'static str,
        provider_label: &'static str,
    ) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            model_name: environment
                .var(model_env)
                .unwrap_or_else(|| default_model.to_string()),
            configured: environment.var(api_key_env).is_some(),
            missing_configuration_reason: format!("set {api_key_env} to enable {provider_label}"),
        }
    }

    pub(crate) fn model_name(&self) -> &str {
        &self.model_name
    }

    pub(crate) fn configured(&self) -> bool {
        self.configured
    }

    pub(crate) fn missing_configuration_reason(&self) -> &str {
        &self.missing_configuration_reason
    }
}
