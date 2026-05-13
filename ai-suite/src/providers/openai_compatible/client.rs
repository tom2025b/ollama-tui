use std::{env, time::Duration};

use reqwest::Client;

use crate::llm::{ConversationTurn, append_utf8_chunk, finish_utf8_stream};
use crate::providers::openai_compatible::http::require_success;
use crate::providers::openai_compatible::stream::{
    process_chat_completion_stream_buffer, process_final_chat_completion_stream_buffer,
};
use crate::providers::openai_compatible::types::ChatCompletionRequest;
use crate::{Error, Result};

/// Request timeout for cloud LLM calls.
const CLOUD_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Small client for APIs that use OpenAI-style chat completions.
#[derive(Clone, Debug)]
pub struct ChatCompletionsClient {
    /// Human-readable provider name used in error messages.
    provider_name: &'static str,
    /// Complete URL for the provider's chat completions endpoint.
    api_url: String,
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
        let api_key = env::var(api_key_env)
            .map_err(|_| Error::missing_api_key(provider_name, api_key_env))?;
        Self::new(provider_name, api_url.to_string(), api_key)
    }

    fn new(provider_name: &'static str, api_url: String, api_key: String) -> Result<Self> {
        let http = Client::builder()
            .timeout(CLOUD_REQUEST_TIMEOUT)
            .build()
            .map_err(|source| Error::http_client_build(provider_name, source))?;

        Ok(Self {
            provider_name,
            api_url,
            api_key,
            http,
        })
    }

    #[cfg(test)]
    pub(crate) fn for_test(provider_name: &'static str, api_url: String) -> Result<Self> {
        Self::new(provider_name, api_url, "test-api-key".to_string())
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
            .post(&self.api_url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|source| Error::http_request(self.provider_name, source))?;

        let mut response = require_success(response, self.provider_name).await?;
        let (mut buffer, mut answer, mut pending_utf8) = (String::new(), String::new(), Vec::new());

        while let Some(chunk) = response.chunk().await.map_err(|source| {
            Error::streaming(
                self.provider_name,
                format!(
                    "failed to read {} stream chunk: {source}",
                    self.provider_name
                ),
            )
        })? {
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
