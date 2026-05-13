use crate::providers::openai_compatible::ChatCompletionsClient;
use crate::{Result, llm::ConversationTurn};

/// xAI API key environment variable.
pub const XAI_API_KEY_ENV: &str = "XAI_API_KEY";

/// Optional model override for this app.
pub const XAI_MODEL_ENV: &str = "XAI_MODEL";

/// Default Grok model.
///
/// xAI documents this model for chat completions and recommends a long timeout
/// for reasoning models.
pub const DEFAULT_XAI_MODEL: &str = "grok-4.20-reasoning";

/// xAI chat completions endpoint.
const XAI_CHAT_COMPLETIONS_URL: &str = "https://api.x.ai/v1/chat/completions";

/// Stream one prompt to xAI.
pub async fn stream<F>(
    model_name: &str,
    context: &[ConversationTurn],
    prompt: &str,
    on_token: F,
) -> Result<String>
where
    F: FnMut(String),
{
    let client = ChatCompletionsClient::from_env("xAI", XAI_CHAT_COMPLETIONS_URL, XAI_API_KEY_ENV)?;

    client.stream(model_name, context, prompt, on_token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires XAI_API_KEY and makes a live xAI API call"]
    async fn live_xai_stream_smoke_test() {
        let answer = stream(
            DEFAULT_XAI_MODEL,
            &[],
            "Reply with one short sentence confirming Grok is working.",
            |_| {},
        )
        .await
        .expect("xAI streaming should work");

        assert!(!answer.trim().is_empty());
    }
}
