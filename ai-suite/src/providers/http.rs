//! Shared HTTP response helpers used by every provider client.

use crate::{Error, Result};

/// Treat non-success HTTP responses as `Error::HttpStatus` with the response
/// body attached when readable. Each provider's `http.rs` delegates here with
/// its own static provider name.
pub(crate) async fn require_success(
    response: reqwest::Response,
    provider_name: &'static str,
) -> Result<reqwest::Response> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "response body could not be read".to_string());

    Err(Error::http_status(provider_name, status, body))
}
