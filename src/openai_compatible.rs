use std::{env, time::Duration};

use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::llm::ConversationTurn;

/// Request timeout for cloud LLM calls.
///
/// Reasoning models can take longer than a normal web request, so this is
/// intentionally generous while still preventing a request from hanging
/// forever.
const CLOUD_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Small client for APIs that use OpenAI-style chat completions.
///
/// OpenAI and xAI both support the same basic request/response shape:
/// - `POST /v1/chat/completions`
/// - bearer-token authentication,
/// - `messages`,
/// - response text at `choices[0].message.content`.
///
/// Keeping that shared shape here avoids duplicate code while the public
/// `openai.rs` and `xai.rs` modules remain provider-specific.
#[derive(Clone, Debug)]
pub struct ChatCompletionsClient {
    /// Human-readable provider name used in error messages.
    provider_name: &'static str,

    /// Complete URL for the provider's chat completions endpoint.
    api_url: &'static str,

    /// API key loaded from the provider-specific environment variable.
    api_key: String,

    /// Reusable HTTP client.
    http: Client,
}

impl ChatCompletionsClient {
    /// Build a client from an API key environment variable.
    pub fn from_env(
        provider_name: &'static str,
        api_url: &'static str,
        api_key_env: &'static str,
    ) -> Result<Self> {
        let api_key = env::var(api_key_env).with_context(|| {
            format!("{provider_name} backend requires the `{api_key_env}` environment variable")
        })?;

        let http = Client::builder()
            .timeout(CLOUD_REQUEST_TIMEOUT)
            .build()
            .with_context(|| format!("failed to create {provider_name} HTTP client"))?;

        Ok(Self {
            provider_name,
            api_url,
            api_key,
            http,
        })
    }

    /// Stream a prompt with bounded conversation context.
    pub async fn stream<F>(
        &self,
        model_name: &str,
        context: &[ConversationTurn],
        prompt: &str,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        let request = ChatCompletionRequest {
            model: model_name.to_string(),
            messages: chat_messages_from_context(context, prompt),
            stream: true,
        };

        let response = self
            .http
            .post(self.api_url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("failed to contact {}", self.provider_name))?;

        let mut response = require_success(response, self.provider_name).await?;
        let mut buffer = String::new();
        let mut answer = String::new();

        while let Some(chunk) = response
            .chunk()
            .await
            .with_context(|| format!("failed to read {} stream chunk", self.provider_name))?
        {
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            process_chat_completion_stream_buffer(
                self.provider_name,
                &mut buffer,
                &mut answer,
                &mut on_token,
            )?;
        }

        process_final_chat_completion_stream_buffer(
            self.provider_name,
            &mut buffer,
            &mut answer,
            &mut on_token,
        )?;

        Ok(answer)
    }
}

/// Request body for OpenAI-style chat completions.
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    /// Model ID, such as `gpt-4o` or `grok-4.20-reasoning`.
    model: String,

    /// Bounded conversation plus the current user prompt.
    messages: Vec<ChatCompletionMessage>,

    /// `true` lets the TUI render text as it arrives.
    stream: bool,
}

/// One chat message sent to an OpenAI-compatible backend.
#[derive(Clone, Debug, Serialize)]
struct ChatCompletionMessage {
    /// Chat role: `user` or `assistant`.
    role: &'static str,

    /// Plain text message content.
    content: String,
}

/// Streaming response frame returned as a server-sent event data payload.
#[derive(Debug, Deserialize)]
struct ChatCompletionStreamResponse {
    /// Candidate completion deltas.
    choices: Vec<ChatCompletionStreamChoice>,
}

/// One streaming choice.
#[derive(Debug, Deserialize)]
struct ChatCompletionStreamChoice {
    /// Assistant delta for this frame.
    delta: ChatCompletionStreamDelta,
}

/// Assistant message delta.
#[derive(Debug, Deserialize)]
struct ChatCompletionStreamDelta {
    /// Text content for this frame, when present.
    content: Option<String>,
}

/// Convert bounded conversation context into chat-completions messages.
fn chat_messages_from_context(
    context: &[ConversationTurn],
    prompt: &str,
) -> Vec<ChatCompletionMessage> {
    let mut messages = Vec::with_capacity(context.len() * 2 + 1);

    for turn in context {
        messages.push(ChatCompletionMessage {
            role: "user",
            content: turn.user.clone(),
        });
        messages.push(ChatCompletionMessage {
            role: "assistant",
            content: turn.assistant.clone(),
        });
    }

    messages.push(ChatCompletionMessage {
        role: "user",
        content: prompt.to_string(),
    });

    messages
}

/// Process complete SSE lines currently in the stream buffer.
fn process_chat_completion_stream_buffer<F>(
    provider_name: &'static str,
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    while let Some(newline_index) = buffer.find('\n') {
        let line = buffer.drain(..=newline_index).collect::<String>();
        process_chat_completion_stream_line(provider_name, line.trim(), answer, on_token)?;
    }

    Ok(())
}

/// Process a final unterminated SSE line left after the stream ends.
fn process_final_chat_completion_stream_buffer<F>(
    provider_name: &'static str,
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
        process_chat_completion_stream_line(provider_name, &line, answer, on_token)?;
    }

    Ok(())
}

/// Parse one OpenAI-compatible SSE line and emit any content delta.
fn process_chat_completion_stream_line<F>(
    provider_name: &'static str,
    line: &str,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(());
    };

    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return Ok(());
    }

    let frame = serde_json::from_str::<ChatCompletionStreamResponse>(data)
        .with_context(|| format!("{provider_name} returned an invalid stream frame: {data}"))?;

    for choice in frame.choices {
        if let Some(content) = choice.delta.content
            && !content.is_empty()
        {
            answer.push_str(&content);
            on_token(content);
        }
    }

    Ok(())
}

/// Convert non-success HTTP responses into useful error messages.
async fn require_success(
    response: reqwest::Response,
    provider_name: &'static str,
) -> Result<reqwest::Response> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "response body could not be read".to_string());

    bail!("{provider_name} returned HTTP {status}. Response body: {body}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_messages_include_bounded_context_then_current_prompt() {
        let context = vec![ConversationTurn {
            user: "old prompt".to_string(),
            assistant: "old answer".to_string(),
        }];

        let messages = chat_messages_from_context(&context, "new prompt");

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "old prompt");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "old answer");
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "new prompt");
    }

    #[test]
    fn stream_line_emits_chat_completion_delta() {
        let mut answer = String::new();
        let mut tokens = Vec::new();

        process_chat_completion_stream_line(
            "test provider",
            r#"data: {"choices":[{"delta":{"content":"hello"}}]}"#,
            &mut answer,
            &mut |token| tokens.push(token),
        )
        .expect("stream line should parse");

        assert_eq!(answer, "hello");
        assert_eq!(tokens, vec!["hello"]);
    }
}
