use std::{env, time::Duration};

use anyhow::{Context, Result, bail};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::llm::ConversationTurn;

/// Default Ollama host.
///
/// Ollama normally listens here after you run `ollama serve`.
const DEFAULT_OLLAMA_HOST: &str = "http://localhost:11434";

/// Environment variable used by Ollama itself to point clients at a server.
///
/// Examples:
/// - `OLLAMA_HOST=http://localhost:11434`
/// - `OLLAMA_HOST=127.0.0.1:11434`
const OLLAMA_HOST_ENV: &str = "OLLAMA_HOST";

/// How long one Ollama request may run before the app gives up.
///
/// Local model generation can take a while on a small machine, so this timeout
/// is intentionally generous.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Small, focused client for the Ollama HTTP API.
///
/// This is the complete backend for the first version of the app. It owns:
/// - finding the Ollama host,
/// - checking whether the requested model exists,
/// - sending prompts to `/api/generate`,
/// - returning clear errors when Ollama is not ready.
#[derive(Clone, Debug)]
pub struct OllamaClient {
    /// Base server URL without a trailing slash.
    ///
    /// Example: `http://localhost:11434`
    base_url: String,

    /// Reusable HTTP client.
    ///
    /// Reusing a client is the normal Reqwest pattern because it can reuse
    /// connections internally.
    http: Client,
}

impl OllamaClient {
    /// Build a client using `OLLAMA_HOST`, or the local Ollama default.
    pub fn from_environment() -> Result<Self> {
        let host = env::var(OLLAMA_HOST_ENV).unwrap_or_else(|_| DEFAULT_OLLAMA_HOST.to_string());
        Self::new(host)
    }

    /// Build a client for a specific host.
    ///
    /// This is useful both for normal configuration and for tests that point
    /// the client at a tiny local mock server.
    pub fn new(host: impl Into<String>) -> Result<Self> {
        let base_url = normalize_host(host.into());
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .context("failed to create Ollama HTTP client")?;

        Ok(Self { base_url, http })
    }

    /// Stream a chat response from a specific Ollama model.
    pub async fn stream<F>(
        &self,
        model_name: &str,
        context: &[ConversationTurn],
        prompt: &str,
        on_token: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        self.ensure_model_is_available(model_name).await?;
        self.stream_without_model_check(model_name, context, prompt, on_token)
            .await
    }

    /// Ask Ollama which models are installed locally.
    async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = self.api_url("/api/tags");

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| connection_error(&url))?;

        let response = require_success(response).await?;

        let body = response
            .json::<TagsResponse>()
            .await
            .context("Ollama answered `/api/tags`, but the JSON shape was not recognized")?;

        Ok(body.models)
    }

    /// Verify that the requested model is installed in Ollama.
    async fn ensure_model_is_available(&self, requested_model: &str) -> Result<()> {
        let models = self.list_models().await?;
        ensure_model_name_is_available(&models, requested_model)
    }

    /// Send the streaming chat request after setup checks have passed.
    async fn stream_without_model_check<F>(
        &self,
        model_name: &str,
        context: &[ConversationTurn],
        prompt: &str,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        let url = self.api_url("/api/chat");
        let request = ChatRequest {
            model: model_name.to_string(),
            messages: chat_messages_from_context(context, prompt),
            stream: true,
        };

        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| connection_error(&url))?;

        let mut response = require_success(response).await?;
        let mut buffer = String::new();
        let mut answer = String::new();

        while let Some(chunk) = response
            .chunk()
            .await
            .context("failed to read Ollama stream chunk")?
        {
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            process_ollama_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
        }

        process_final_ollama_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
        Ok(answer)
    }

    /// Join the base host and an API path.
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

/// Response body from Ollama's `/api/tags` endpoint.
///
/// The endpoint returns extra fields for each model. This app only needs the
/// model name to verify that `llama3` is installed.
#[derive(Debug, Deserialize)]
struct TagsResponse {
    /// Installed local models.
    models: Vec<OllamaModel>,
}

/// One model entry returned by `/api/tags`.
#[derive(Debug, Deserialize)]
struct OllamaModel {
    /// Full Ollama model name, commonly something like `llama3:latest`.
    name: String,
}

/// Request body for Ollama's `/api/chat` endpoint.
#[derive(Debug, Serialize)]
struct ChatRequest {
    /// Ollama model name, such as `llama3`.
    model: String,

    /// Bounded conversation plus the current user prompt.
    messages: Vec<OllamaChatMessage>,

    /// `true` lets the TUI render text as it arrives.
    stream: bool,
}

/// One chat message sent to Ollama.
#[derive(Clone, Debug, Serialize)]
struct OllamaChatMessage {
    /// Chat role: `user` or `assistant`.
    role: &'static str,

    /// Plain text message content.
    content: String,
}

/// One JSON line from Ollama's streaming chat endpoint.
#[derive(Debug, Deserialize)]
struct ChatStreamChunk {
    /// Assistant message delta for this chunk.
    message: Option<OllamaChatResponseMessage>,

    /// True when Ollama has finished the response.
    #[serde(default)]
    #[allow(dead_code)]
    done: bool,
}

/// Assistant message object inside a streaming chat chunk.
#[derive(Debug, Deserialize)]
struct OllamaChatResponseMessage {
    /// Delta content for this chunk.
    content: String,
}

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

/// Convert bounded conversation context into Ollama chat messages.
fn chat_messages_from_context(
    context: &[ConversationTurn],
    prompt: &str,
) -> Vec<OllamaChatMessage> {
    let mut messages = Vec::with_capacity(context.len() * 2 + 1);

    for turn in context {
        messages.push(OllamaChatMessage {
            role: "user",
            content: turn.user.clone(),
        });
        messages.push(OllamaChatMessage {
            role: "assistant",
            content: turn.assistant.clone(),
        });
    }

    messages.push(OllamaChatMessage {
        role: "user",
        content: prompt.to_string(),
    });

    messages
}

/// Normalize user-provided Ollama host values into a URL base.
///
/// Users often write `127.0.0.1:11434` because Ollama examples use host-style
/// values. Reqwest needs a URL with a scheme, so this adds `http://` when the
/// scheme is missing.
fn normalize_host(raw_host: String) -> String {
    let trimmed = raw_host.trim().trim_end_matches('/').to_string();

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed
    } else {
        format!("http://{trimmed}")
    }
}

/// Decide whether an installed Ollama model satisfies a requested model name.
///
/// Ollama commonly stores `llama3` as `llama3:latest`. The app requests
/// `llama3`, so both exact and `:latest` names are accepted.
fn model_name_matches_request(installed_name: &str, requested_name: &str) -> bool {
    installed_name == requested_name || installed_name == format!("{requested_name}:latest")
}

/// Check a `/api/tags` model list for one requested model.
///
/// This is split out from the HTTP method so the important setup behavior can
/// be tested without requiring a real server or a test socket.
fn ensure_model_name_is_available(models: &[OllamaModel], requested_model: &str) -> Result<()> {
    if models
        .iter()
        .any(|model| model_name_matches_request(&model.name, requested_model))
    {
        return Ok(());
    }

    let installed_names = models
        .iter()
        .map(|model| model.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    if installed_names.is_empty() {
        bail!(
            "Ollama is running, but no local models are installed. Run `ollama pull {requested_model}`."
        );
    }

    bail!(
        "Ollama model `{requested_model}` is not installed. Installed models: {installed_names}. Run `ollama pull {requested_model}`."
    );
}

/// Process complete newline-delimited JSON records currently in the stream buffer.
fn process_ollama_stream_buffer<F>(
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    while let Some(newline_index) = buffer.find('\n') {
        let line = buffer.drain(..=newline_index).collect::<String>();
        process_ollama_stream_line(line.trim(), answer, on_token)?;
    }

    Ok(())
}

/// Process any final unterminated JSON record left after the response ends.
fn process_final_ollama_stream_buffer<F>(
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
        process_ollama_stream_line(&line, answer, on_token)?;
    }

    Ok(())
}

/// Parse one Ollama streaming JSON line and emit its content delta.
fn process_ollama_stream_line<F>(line: &str, answer: &mut String, on_token: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    if line.is_empty() {
        return Ok(());
    }

    let chunk = serde_json::from_str::<ChatStreamChunk>(line)
        .with_context(|| format!("Ollama returned an invalid stream line: {line}"))?;

    if let Some(message) = chunk.message
        && !message.content.is_empty()
    {
        answer.push_str(&message.content);
        on_token(message.content);
    }

    Ok(())
}

/// Turn connection failures into a message that tells the user what to do.
fn connection_error(url: &str) -> String {
    format!("failed to contact Ollama at {url}; make sure `ollama serve` is running")
}

/// Treat non-success HTTP responses as errors with useful response text.
///
/// Reqwest's `error_for_status` is fine, but reading the body gives better
/// messages for setup problems returned by Ollama.
async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "response body could not be read".to_string());

    if status == StatusCode::NOT_FOUND {
        bail!("Ollama returned 404 Not Found. Response body: {body}");
    }

    bail!("Ollama returned HTTP {status}. Response body: {body}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_host_adds_http_scheme_when_missing() {
        assert_eq!(
            normalize_host("127.0.0.1:11434/".to_string()),
            "http://127.0.0.1:11434"
        );
    }

    #[test]
    fn normalize_host_keeps_existing_scheme() {
        assert_eq!(
            normalize_host("https://example.test:11434/".to_string()),
            "https://example.test:11434"
        );
    }

    #[test]
    fn model_name_match_accepts_latest_tag() {
        assert!(model_name_matches_request("llama3:latest", "llama3"));
    }

    #[test]
    fn available_model_check_accepts_latest_tag() {
        let installed_models = vec![OllamaModel {
            name: "llama3:latest".to_string(),
        }];

        ensure_model_name_is_available(&installed_models, "llama3")
            .expect("llama3:latest should satisfy llama3");
    }

    #[test]
    fn available_model_check_explains_missing_model() {
        let installed_models = vec![OllamaModel {
            name: "mistral:latest".to_string(),
        }];

        let error = ensure_model_name_is_available(&installed_models, "llama3")
            .expect_err("missing llama3 should be explained");
        assert!(error.to_string().contains("ollama pull llama3"));
    }

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
    fn stream_line_emits_ollama_chat_content() {
        let mut answer = String::new();
        let mut tokens = Vec::new();

        process_ollama_stream_line(
            r#"{"message":{"role":"assistant","content":"hello"},"done":false}"#,
            &mut answer,
            &mut |token| tokens.push(token),
        )
        .expect("stream line should parse");

        assert_eq!(answer, "hello");
        assert_eq!(tokens, vec!["hello"]);
    }

    #[tokio::test]
    #[ignore = "requires a running local Ollama server with llama3 installed"]
    async fn live_ollama_stream_smoke_test() {
        let client = OllamaClient::from_environment().expect("client should build");
        let mut tokens = Vec::new();
        let answer = client
            .stream(
                "llama3",
                &[],
                "Reply with one short sentence confirming you are working.",
                |token| tokens.push(token),
            )
            .await
            .expect("local Ollama llama3 streaming should work");

        assert!(!answer.trim().is_empty());
        assert!(!tokens.is_empty());
    }
}
