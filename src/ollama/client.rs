mod availability;
mod streaming;

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use super::host::{OLLAMA_HOST_ENV, default_host, normalize_host};
use crate::llm::ConversationTurn;

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

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
