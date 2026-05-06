use std::{env, time::Duration};

use anyhow::{Context, Result};
use reqwest::Client;

use crate::llm::{ConversationTurn, append_utf8_chunk, finish_utf8_stream};
use crate::providers::openai_compatible::http::require_success;
use crate::providers::openai_compatible::stream::{
    process_chat_completion_stream_buffer, process_final_chat_completion_stream_buffer,
};
use crate::providers::openai_compatible::types::ChatCompletionRequest;

/// Request timeout for cloud LLM calls.
const CLOUD_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Small client for APIs that use OpenAI-style chat completions.
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
        let request = ChatCompletionRequest::new(model_name, context, prompt);
        let response = self
            .http
            .post(self.api_url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("failed to contact {}", self.provider_name))?;

        let mut response = require_success(response, self.provider_name).await?;
        let (mut buffer, mut answer, mut pending_utf8) = (String::new(), String::new(), Vec::new());

        while let Some(chunk) = response
            .chunk()
            .await
            .with_context(|| format!("failed to read {} stream chunk", self.provider_name))?
        {
            append_utf8_chunk(self.provider_name, &mut pending_utf8, &mut buffer, &chunk)?;
            process_chat_completion_stream_buffer(
                self.provider_name,
                &mut buffer,
                &mut answer,
                &mut on_token,
            )?;
        }

        finish_utf8_stream(self.provider_name, &mut pending_utf8, &mut buffer)?;
        process_final_chat_completion_stream_buffer(
            self.provider_name,
            &mut buffer,
            &mut answer,
            &mut on_token,
        )?;
        Ok(answer)
    }
}
