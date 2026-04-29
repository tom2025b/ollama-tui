use std::{env, time::Duration};

use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::llm::ConversationTurn;

/// Anthropic API key environment variable.
pub const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Optional model override for this app.
pub const ANTHROPIC_MODEL_ENV: &str = "ANTHROPIC_MODEL";

/// Stable Claude model used by default.
///
/// Anthropic recommends pinned model IDs for consistent behavior. This is the
/// Sonnet 4 model ID from Anthropic's model list.
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-20250514";

/// Anthropic Messages API endpoint.
const ANTHROPIC_MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";

/// Required API version header for Anthropic's Messages API.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Maximum answer length for this first version.
const MAX_TOKENS: u32 = 2048;

/// Cloud request timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Return the configured Claude model name.
pub fn configured_model_name() -> String {
    env::var(ANTHROPIC_MODEL_ENV).unwrap_or_else(|_| DEFAULT_ANTHROPIC_MODEL.to_string())
}

/// True when the Claude backend has enough local configuration to be selected.
pub fn is_configured() -> bool {
    env::var(ANTHROPIC_API_KEY_ENV).is_ok()
}

/// Explain how to enable this backend.
pub fn missing_configuration_reason() -> String {
    format!("set {ANTHROPIC_API_KEY_ENV} to enable Claude")
}

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
    let api_key = env::var(ANTHROPIC_API_KEY_ENV).with_context(|| {
        format!("Anthropic backend requires the `{ANTHROPIC_API_KEY_ENV}` environment variable")
    })?;

    let http = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("failed to create Anthropic HTTP client")?;

    let request = AnthropicRequest {
        model: model_name.to_string(),
        max_tokens: MAX_TOKENS,
        messages: anthropic_messages_from_context(context, prompt),
        stream: true,
    };

    let response = http
        .post(ANTHROPIC_MESSAGES_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&request)
        .send()
        .await
        .context("failed to contact Anthropic")?;

    let mut response = require_success(response).await?;
    let mut buffer = String::new();
    let mut answer = String::new();

    while let Some(chunk) = response
        .chunk()
        .await
        .context("failed to read Anthropic stream chunk")?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        process_anthropic_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
    }

    process_final_anthropic_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
    Ok(answer)
}

/// Request body for Anthropic's Messages API.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    /// Claude model ID.
    model: String,

    /// Maximum number of tokens Claude may generate.
    max_tokens: u32,

    /// Bounded conversation plus the current user prompt.
    messages: Vec<AnthropicMessage>,

    /// Whether Anthropic should return server-sent events.
    stream: bool,
}

/// One message sent to Claude.
#[derive(Clone, Debug, Serialize)]
struct AnthropicMessage {
    /// Chat role: `user` or `assistant`.
    role: &'static str,

    /// Plain text message content.
    content: String,
}

/// One data payload from Anthropic's streaming Messages API.
#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    /// Event payload type.
    #[serde(rename = "type")]
    event_type: String,

    /// Text delta for `content_block_delta` events.
    delta: Option<AnthropicStreamDelta>,
}

/// Delta object inside a streaming event.
#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    /// Delta payload type.
    #[serde(rename = "type")]
    delta_type: String,

    /// Text chunk for `text_delta` events.
    text: Option<String>,
}

/// Convert bounded conversation context into Anthropic messages.
fn anthropic_messages_from_context(
    context: &[ConversationTurn],
    prompt: &str,
) -> Vec<AnthropicMessage> {
    let mut messages = Vec::with_capacity(context.len() * 2 + 1);

    for turn in context {
        messages.push(AnthropicMessage {
            role: "user",
            content: turn.user.clone(),
        });
        messages.push(AnthropicMessage {
            role: "assistant",
            content: turn.assistant.clone(),
        });
    }

    messages.push(AnthropicMessage {
        role: "user",
        content: prompt.to_string(),
    });

    messages
}

/// Process complete SSE lines currently in the stream buffer.
fn process_anthropic_stream_buffer<F>(
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    while let Some(newline_index) = buffer.find('\n') {
        let line = buffer.drain(..=newline_index).collect::<String>();
        process_anthropic_stream_line(line.trim(), answer, on_token)?;
    }

    Ok(())
}

/// Process any final unterminated SSE line left after the response ends.
fn process_final_anthropic_stream_buffer<F>(
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    let line = buffer.trim().to_string();
    buffer.clear();

    if !line.is_empty() {
        process_anthropic_stream_line(&line, answer, on_token)?;
    }

    Ok(())
}

/// Parse one Anthropic SSE line and emit any text delta.
fn process_anthropic_stream_line<F>(line: &str, answer: &mut String, on_token: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(());
    };

    let data = data.trim();
    if data.is_empty() {
        return Ok(());
    }

    let event = serde_json::from_str::<AnthropicStreamEvent>(data)
        .with_context(|| format!("Anthropic returned an invalid stream event: {data}"))?;

    if event.event_type != "content_block_delta" {
        return Ok(());
    }

    let Some(delta) = event.delta else {
        return Ok(());
    };

    if delta.delta_type != "text_delta" {
        return Ok(());
    }

    if let Some(text) = delta.text
        && !text.is_empty()
    {
        answer.push_str(&text);
        on_token(text);
    }

    Ok(())
}

/// Convert non-success Anthropic responses into useful error messages.
async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "response body could not be read".to_string());

    bail!("Anthropic returned HTTP {status}. Response body: {body}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_messages_include_bounded_context_then_current_prompt() {
        let context = vec![ConversationTurn {
            user: "old prompt".to_string(),
            assistant: "old answer".to_string(),
        }];

        let messages = anthropic_messages_from_context(&context, "new prompt");

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "old prompt");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "old answer");
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "new prompt");
    }

    #[test]
    fn stream_line_emits_anthropic_text_delta() {
        let mut answer = String::new();
        let mut tokens = Vec::new();

        process_anthropic_stream_line(
            r#"data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"hello"}}"#,
            &mut answer,
            &mut |token| tokens.push(token),
        )
        .expect("stream line should parse");

        assert_eq!(answer, "hello");
        assert_eq!(tokens, vec!["hello"]);
    }

    #[tokio::test]
    #[ignore = "requires ANTHROPIC_API_KEY and makes a live Anthropic API call"]
    async fn live_anthropic_stream_smoke_test() {
        let answer = stream(
            DEFAULT_ANTHROPIC_MODEL,
            &[],
            "Reply with one short sentence confirming Claude is working.",
            |_| {},
        )
        .await
        .expect("Anthropic streaming should work");

        assert!(!answer.trim().is_empty());
    }
}
