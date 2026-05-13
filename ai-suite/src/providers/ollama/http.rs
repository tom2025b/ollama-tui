use crate::Result;

/// Treat non-success Ollama responses as errors with useful response text.
pub(super) async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
    crate::providers::http::require_success(response, "Ollama").await
}
