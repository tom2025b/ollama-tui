mod config;
mod http;
mod stream_parser;
mod types;

#[allow(unused_imports)]
pub use config::{
    ANTHROPIC_API_KEY_ENV, ANTHROPIC_MODEL_ENV, DEFAULT_ANTHROPIC_MODEL, configured_model_name,
    is_configured, missing_configuration_reason,
};

use anyhow::{Context, Result};
use reqwest::Client;

use self::config::{ANTHROPIC_MESSAGES_URL, ANTHROPIC_VERSION, MAX_TOKENS, REQUEST_TIMEOUT};
use self::http::require_success;
use self::stream_parser::{process_anthropic_stream_buffer, process_final_anthropic_stream_buffer};
use self::types::AnthropicRequest;
use crate::llm::{ConversationTurn, append_utf8_chunk, finish_utf8_stream};

/// Stream one prompt plus bounded context from Anthropic's Messages API.
pub async fn stream<F>(
    model_name: &str,
    context: &[ConversationTurn],
    prompt: &str,
    mut on_token: F,
) -> Result<String>
where
    F: FnMut(String),
{
    let api_key = std::env::var(ANTHROPIC_API_KEY_ENV).with_context(|| {
        format!("Anthropic backend requires the `{ANTHROPIC_API_KEY_ENV}` environment variable")
    })?;
    let http = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("failed to create Anthropic HTTP client")?;
    let request = AnthropicRequest::new(model_name, context, prompt, MAX_TOKENS);
    let response = http
        .post(ANTHROPIC_MESSAGES_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&request)
        .send()
        .await
        .context("failed to contact Anthropic")?;

    let mut response = require_success(response).await?;
    let (mut buffer, mut answer, mut pending_utf8) = (String::new(), String::new(), Vec::new());

    while let Some(chunk) = response
        .chunk()
        .await
        .context("failed to read Anthropic stream chunk")?
    {
        append_utf8_chunk("Anthropic", &mut pending_utf8, &mut buffer, &chunk)?;
        process_anthropic_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
    }

    finish_utf8_stream("Anthropic", &mut pending_utf8, &mut buffer)?;
    process_final_anthropic_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
    Ok(answer)
}

#[cfg(test)]
mod tests;
