mod streaming;

use std::{env, time::Duration};

use anyhow::{Context, Result};
use reqwest::Client;

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
}
