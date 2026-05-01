use anyhow::{Result, bail};
use reqwest::StatusCode;

/// Turn connection failures into a message that tells the user what to do.
pub(super) fn connection_error(url: &str) -> String {
    format!("failed to contact Ollama at {url}; make sure `ollama serve` is running")
}

/// Treat non-success HTTP responses as errors with useful response text.
pub(super) async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
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
