use crate::providers::ollama;
use crate::{
    Result,
    llm::{ConversationTurn, LanguageModel},
};

/// Provider-neutral request handed to the Ollama backend.
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
        "Ollama"
    }
}

/// Stream a request through the Ollama backend.
pub(crate) async fn stream_model_request<F>(request: &ModelRequest, on_token: F) -> Result<String>
where
    F: FnMut(String),
{
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
