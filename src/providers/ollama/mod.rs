mod client;
mod host;
mod http;
mod models;
mod stream;
mod types;

pub use client::OllamaClient;

use anyhow::Result;

use crate::llm::ConversationTurn;

/// Stream a prompt to Ollama using environment/default configuration.
pub async fn stream<F>(
    model_name: &str,
    context: &[ConversationTurn],
    prompt: &str,
    on_token: F,
) -> Result<String>
where
    F: FnMut(String),
{
    let client = OllamaClient::from_environment()?;
    client.stream(model_name, context, prompt, on_token).await
}

#[cfg(test)]
mod tests;
