use serde::Deserialize;

use crate::{Error, Result};

/// Response body from Ollama's `/api/tags` endpoint.
#[derive(Debug, Deserialize)]
pub(super) struct TagsResponse {
    /// Installed local models.
    pub(super) models: Vec<OllamaModel>,
}

/// One model entry returned by `/api/tags`.
#[derive(Debug, Deserialize)]
pub struct OllamaModel {
    /// Full Ollama model name, commonly something like `llama3:latest`.
    pub name: String,
    /// On-disk size in bytes.
    #[serde(default)]
    pub size: u64,
    /// ISO-8601 timestamp of the last modification, as returned by Ollama.
    #[serde(default, rename = "modified_at")]
    pub modified_at: String,
}

/// Decide whether an installed Ollama model satisfies a requested model name.
pub(super) fn model_name_matches_request(installed_name: &str, requested_name: &str) -> bool {
    installed_name == requested_name || installed_name == format!("{requested_name}:latest")
}

/// Check a `/api/tags` model list for one requested model.
pub(super) fn ensure_model_name_is_available(
    models: &[OllamaModel],
    requested_model: &str,
) -> Result<()> {
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
        return Err(Error::provider_response(
            "Ollama",
            format!(
                "Ollama is running, but no local models are installed. Run `ollama pull {requested_model}`."
            ),
        ));
    }

    Err(Error::provider_response(
        "Ollama",
        format!(
            "Ollama model `{requested_model}` is not installed. Installed models: {installed_names}. Run `ollama pull {requested_model}`."
        ),
    ))
}
