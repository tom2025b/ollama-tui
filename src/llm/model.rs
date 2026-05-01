use super::provider::Provider;

/// A language model the router is allowed to choose.
#[derive(Clone, Debug)]
pub struct LanguageModel {
    /// The provider-specific model name.
    pub name: String,
    /// The backend that serves this model.
    pub provider: Provider,
    /// Human-readable notes used by the TUI.
    pub strengths: Vec<String>,
    /// Whether the router is allowed to choose this model right now.
    pub enabled: bool,
    /// Short setup note shown when a model is not currently usable.
    pub disabled_reason: Option<String>,
}

impl LanguageModel {
    /// Build a model entry backed by Ollama.
    pub fn ollama(name: &str, strengths: &[&str]) -> Self {
        Self::new(Provider::Ollama, name, strengths, true, None)
    }

    /// Build a Claude model entry.
    pub fn anthropic(
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self::new(
            Provider::Anthropic,
            name,
            strengths,
            enabled,
            disabled_reason,
        )
    }

    /// Build an OpenAI model entry.
    pub fn openai(
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self::new(Provider::OpenAi, name, strengths, enabled, disabled_reason)
    }

    /// Build an xAI model entry.
    pub fn xai(
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self::new(Provider::Xai, name, strengths, enabled, disabled_reason)
    }

    fn new(
        provider: Provider,
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            provider,
            strengths: strengths
                .iter()
                .map(|strength| strength.to_string())
                .collect(),
            enabled,
            disabled_reason,
        }
    }

    /// Label used in history and status messages.
    pub fn display_label(&self) -> String {
        format!("{} {}", self.provider.label(), self.name)
    }
}
