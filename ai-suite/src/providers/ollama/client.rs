use std::time::Duration;

use anyhow::anyhow;
use reqwest::Client;

use super::host::{OLLAMA_HOST_ENV, default_host, normalize_host};
use crate::llm::{ConversationTurn, append_utf8_chunk, finish_utf8_stream};
use crate::providers::ollama::http::require_success;
use crate::providers::ollama::models::{OllamaModel, TagsResponse, ensure_model_name_is_available};
use crate::providers::ollama::stream::{
    process_final_ollama_stream_buffer, process_ollama_stream_buffer,
};
use crate::providers::ollama::types::ChatRequest;
use crate::Result;

/// How long one Ollama request may run before the app gives up.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone, Debug)]
pub struct OllamaClient {
    base_url: String,
    http: Client,
}

impl OllamaClient {
    /// Build a client using `OLLAMA_HOST`, or the local Ollama default.
    pub fn from_environment() -> Result<Self> {
        let host = std::env::var(OLLAMA_HOST_ENV).unwrap_or_else(|_| default_host().to_string());
        Self::new(host)
    }

    /// Build a client for a specific host.
    pub fn new(host: impl Into<String>) -> Result<Self> {
        let base_url = normalize_host(host.into());
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| anyhow!("failed to build Ollama HTTP client: {e}"))?;

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

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Ask Ollama which models are installed locally.
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = self.api_url("/api/tags");
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("failed to contact Ollama: {e}"))?;
        let response = require_success(response).await?;
        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("failed to read Ollama /api/tags response body: {e}"))?;
        let body = serde_json::from_str::<TagsResponse>(&body)
            .map_err(|e| anyhow!("failed to process JSON for Ollama /api/tags response: {e}"))?;

        Ok(body.models)
    }

    async fn ensure_model_is_available(&self, requested_model: &str) -> Result<()> {
        let models = self.list_models().await?;
        ensure_model_name_is_available(&models, requested_model)
    }

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
        let request = ChatRequest::new(model_name, context, prompt);
        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("failed to contact Ollama: {e}"))?;

        let mut response = require_success(response).await?;
        let (mut buffer, mut answer, mut pending_utf8) = (String::new(), String::new(), Vec::new());

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| anyhow!("Ollama streaming error: failed to read stream chunk: {e}"))?
        {
            append_utf8_chunk("Ollama", &mut pending_utf8, &mut buffer, &chunk)?;
            process_ollama_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
        }

        finish_utf8_stream("Ollama", &mut pending_utf8, &mut buffer)?;
        process_final_ollama_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
        Ok(answer)
    }
}
