use anyhow::Result;

use crate::llm::ConversationTurn;
use crate::providers::openai_compatible::ChatCompletionsClient;

/// OpenAI API key environment variable.
pub const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";

/// Optional model override for this app.
pub const OPENAI_MODEL_ENV: &str = "OPENAI_MODEL";

/// GPT-4o model requested for the OpenAI backend.
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4o";

/// OpenAI chat completions endpoint.
const OPENAI_CHAT_COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Stream one prompt to OpenAI.
pub async fn stream<F>(
    model_name: &str,
    context: &[ConversationTurn],
    prompt: &str,
    on_token: F,
) -> Result<String>
where
    F: FnMut(String),
{
    let client =
        ChatCompletionsClient::from_env("OpenAI", OPENAI_CHAT_COMPLETIONS_URL, OPENAI_API_KEY_ENV)?;

    client.stream(model_name, context, prompt, on_token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY and makes a live OpenAI API call"]
    async fn live_openai_stream_smoke_test() {
        let answer = stream(
            DEFAULT_OPENAI_MODEL,
            &[],
            "Reply with one short sentence confirming GPT-4o is working.",
            |_| {},
        )
        .await
        .expect("OpenAI streaming should work");

        assert!(!answer.trim().is_empty());
    }
}
