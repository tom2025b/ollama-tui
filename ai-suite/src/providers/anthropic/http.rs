use crate::Result;

/// Convert non-success Anthropic responses into useful error messages.
pub(super) async fn require_success(response: reqwest::Response) -> Result<reqwest::Response> {
    crate::providers::http::require_success(response, "Anthropic").await
}
