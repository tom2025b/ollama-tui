use anyhow::Result;

use crate::llm::{ConversationTurn, LanguageModel, Provider};
use crate::providers::ollama;

/// Provider-neutral request handed to the concrete model backends.
#[derive(Clone, Debug)]
pub(crate) struct ModelRequest {
    /// Model selected by routing.
    pub(crate) model: LanguageModel,

    /// Bounded conversation context to include with the prompt.
    pub(crate) context: Vec<ConversationTurn>,

    /// Prompt to send to the selected model.
    pub(crate) prompt: String,
}

impl ModelRequest {
    pub(crate) fn new(
        model: LanguageModel,
        context: Vec<ConversationTurn>,
        prompt: String,
    ) -> Self {
        Self {
            model,
            context,
            prompt,
        }
    }

    pub(crate) fn provider_label(&self) -> &'static str {
        self.model.provider.label()
    }
}

/// Stream a model request through Ollama.
///
/// `ClaudeCode` and `Codex` are handled outside this path: the TUI suspends and
/// hands off to the external CLI before reaching the streaming layer, so by the
/// time we get here the provider is always `Ollama`.
pub(crate) async fn stream_model_request<F>(request: &ModelRequest, on_token: F) -> Result<String>
where
    F: FnMut(String),
{
    debug_assert!(
        matches!(request.model.provider, Provider::Ollama),
        "stream_model_request only supports Ollama; terminal apps are intercepted upstream",
    );

    let mut on_token = on_token;
    ollama::stream(
        &request.model.name,
        &request.context,
        &request.prompt,
        &mut on_token,
    )
    .await
}

#[cfg(test)]
mod tests {
    use crate::llm::LanguageModel;

    use super::ModelRequest;

    #[test]
    fn request_exposes_provider_label() {
        let request = ModelRequest::new(
            LanguageModel::ollama("llama3", &["fast"]),
            Vec::new(),
            "hello".to_string(),
        );

        assert_eq!(request.provider_label(), "Ollama");
    }
}
