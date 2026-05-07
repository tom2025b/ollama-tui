use crate::providers::{anthropic, openai, xai};

use super::environment::RuntimeEnvironment;
use super::file_config::FileConfig;

/// Environment variable for the small local Ollama model.
pub(crate) const FAST_OLLAMA_MODEL_ENV: &str = "OLLAMA_FAST_MODEL";

/// Default fast local model.
///
/// This defaults to the same installed Llama 3 tag used by the primary local
/// fallback, so short prompts work on a fresh setup. Set `OLLAMA_FAST_MODEL` to
/// a smaller installed model if you want lower latency.
pub(crate) const DEFAULT_FAST_OLLAMA_MODEL: &str = "llama3:latest";

/// Default number of past user/assistant turns sent to the model.
pub(crate) const DEFAULT_CONTEXT_TURNS: usize = 6;

/// Default number of past turns kept in TUI memory.
pub(crate) const DEFAULT_STORED_TURNS: usize = 200;

/// Where a config value came from. Used by `/config` to show effective sources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConfigSource {
    /// Compile-time default; no env var or file value provided.
    Default,
    /// Loaded from `~/.config/ai-suite/config.toml`.
    File,
    /// Loaded from an environment variable (with the variable name).
    Env(&'static str),
}

impl ConfigSource {
    pub(crate) fn label(&self) -> String {
        match self {
            ConfigSource::Default => "default".to_string(),
            ConfigSource::File => "file".to_string(),
            ConfigSource::Env(name) => format!("env: {name}"),
        }
    }
}

/// One resolved string value plus where it came from.
#[derive(Clone, Debug)]
pub(crate) struct Setting<T> {
    value: T,
    source: ConfigSource,
}

impl<T> Setting<T> {
    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn source(&self) -> &ConfigSource {
        &self.source
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RuntimeConfig {
    models: ModelRuntimeConfig,
    context: ContextLimits,
}

impl RuntimeConfig {
    pub(super) fn from_environment_and_file<E>(environment: &E, file: &FileConfig) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            models: ModelRuntimeConfig::from_environment_and_file(environment, file),
            context: ContextLimits::from_file(&file.context),
        }
    }

    pub(crate) fn models(&self) -> &ModelRuntimeConfig {
        &self.models
    }

    pub(crate) fn context(&self) -> &ContextLimits {
        &self.context
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ContextLimits {
    context_turns: Setting<usize>,
    stored_turns: Setting<usize>,
}

impl ContextLimits {
    fn from_file(file: &super::file_config::ContextSection) -> Self {
        Self {
            context_turns: resolve_usize(file.context_turns, DEFAULT_CONTEXT_TURNS),
            stored_turns: resolve_usize(file.stored_turns, DEFAULT_STORED_TURNS),
        }
    }

    pub(crate) fn context_turns(&self) -> usize {
        *self.context_turns.value()
    }

    pub(crate) fn stored_turns(&self) -> usize {
        *self.stored_turns.value()
    }

    pub(crate) fn context_turns_setting(&self) -> &Setting<usize> {
        &self.context_turns
    }

    pub(crate) fn stored_turns_setting(&self) -> &Setting<usize> {
        &self.stored_turns
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ModelRuntimeConfig {
    fast_ollama_model: Setting<String>,
    anthropic: CloudProviderRuntimeConfig,
    openai: CloudProviderRuntimeConfig,
    xai: CloudProviderRuntimeConfig,
}

impl ModelRuntimeConfig {
    fn from_environment_and_file<E>(environment: &E, file: &FileConfig) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            fast_ollama_model: resolve_string(
                environment,
                FAST_OLLAMA_MODEL_ENV,
                file.models.ollama_fast_model.as_deref(),
                DEFAULT_FAST_OLLAMA_MODEL,
            ),
            anthropic: CloudProviderRuntimeConfig::from_environment_and_file(
                environment,
                anthropic::ANTHROPIC_API_KEY_ENV,
                anthropic::ANTHROPIC_MODEL_ENV,
                file.models.anthropic_model.as_deref(),
                anthropic::DEFAULT_ANTHROPIC_MODEL,
                "Claude",
            ),
            openai: CloudProviderRuntimeConfig::from_environment_and_file(
                environment,
                openai::OPENAI_API_KEY_ENV,
                openai::OPENAI_MODEL_ENV,
                file.models.openai_model.as_deref(),
                openai::DEFAULT_OPENAI_MODEL,
                "GPT-4o",
            ),
            xai: CloudProviderRuntimeConfig::from_environment_and_file(
                environment,
                xai::XAI_API_KEY_ENV,
                xai::XAI_MODEL_ENV,
                file.models.xai_model.as_deref(),
                xai::DEFAULT_XAI_MODEL,
                "Grok",
            ),
        }
    }

    pub(crate) fn fast_ollama_model(&self) -> &str {
        self.fast_ollama_model.value()
    }

    pub(crate) fn fast_ollama_model_setting(&self) -> &Setting<String> {
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
    model: Setting<String>,
    configured: bool,
    missing_configuration_reason: String,
}

impl CloudProviderRuntimeConfig {
    fn from_environment_and_file<E>(
        environment: &E,
        api_key_env: &'static str,
        model_env: &'static str,
        file_model: Option<&str>,
        default_model: &'static str,
        provider_label: &'static str,
    ) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            model: resolve_string(environment, model_env, file_model, default_model),
            configured: env_value(environment, api_key_env).is_some(),
            missing_configuration_reason: format!("set {api_key_env} to enable {provider_label}"),
        }
    }

    pub(crate) fn model_name(&self) -> &str {
        self.model.value()
    }

    pub(crate) fn model_setting(&self) -> &Setting<String> {
        &self.model
    }

    pub(crate) fn configured(&self) -> bool {
        self.configured
    }

    pub(crate) fn missing_configuration_reason(&self) -> &str {
        &self.missing_configuration_reason
    }
}

/// Read an env var, treating empty strings as unset (so an accidental
/// `export FOO=` doesn't clobber file/default values).
fn env_value<E>(environment: &E, key: &'static str) -> Option<String>
where
    E: RuntimeEnvironment + ?Sized,
{
    environment.var(key).filter(|value| !value.is_empty())
}

/// Resolve a string setting with priority: env > file > default.
fn resolve_string<E>(
    environment: &E,
    env_key: &'static str,
    file_value: Option<&str>,
    default: &'static str,
) -> Setting<String>
where
    E: RuntimeEnvironment + ?Sized,
{
    if let Some(value) = env_value(environment, env_key) {
        return Setting {
            value,
            source: ConfigSource::Env(env_key),
        };
    }
    if let Some(value) = file_value.filter(|value| !value.is_empty()) {
        return Setting {
            value: value.to_string(),
            source: ConfigSource::File,
        };
    }
    Setting {
        value: default.to_string(),
        source: ConfigSource::Default,
    }
}

/// Resolve a numeric setting with priority: file > default.
/// (No env-var path today; numeric overrides go in the config file.)
fn resolve_usize(file_value: Option<usize>, default: usize) -> Setting<usize> {
    match file_value {
        Some(value) => Setting {
            value,
            source: ConfigSource::File,
        },
        None => Setting {
            value: default,
            source: ConfigSource::Default,
        },
    }
}
