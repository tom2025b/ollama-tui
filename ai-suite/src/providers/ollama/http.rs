use anyhow::anyhow;

use crate::Result;

/// Treat non-success Ollama responses as errors with the response body attached.
pub(super) async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "response body could not be read".to_string());

    Err(anyhow!("Ollama returned HTTP {status}. Response body: {body}"))
}
