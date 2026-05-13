use crate::{Error, Result};

/// Convert non-success Anthropic responses into useful error messages.
pub(super) async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "response body could not be read".to_string());

    Err(Error::http_status("Anthropic", status, body))
}
