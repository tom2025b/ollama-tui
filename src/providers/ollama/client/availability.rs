use anyhow::{Context, Result};

use super::OllamaClient;
use crate::providers::ollama::http::{connection_error, require_success};
use crate::providers::ollama::models::{OllamaModel, TagsResponse, ensure_model_name_is_available};

impl OllamaClient {
    /// Ask Ollama which models are installed locally.
    async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = self.api_url("/api/tags");
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| connection_error(&url))?;
        let body = require_success(response)
            .await?
            .json::<TagsResponse>()
            .await
            .context("Ollama answered `/api/tags`, but the JSON shape was not recognized")?;

        Ok(body.models)
    }

    pub(super) async fn ensure_model_is_available(&self, requested_model: &str) -> Result<()> {
        let models = self.list_models().await?;
        ensure_model_name_is_available(&models, requested_model)
    }
}
