use crate::Result;

/// Convert non-success OpenAI-compatible responses into useful error messages.
pub(super) async fn require_success(
    response: reqwest::Response,
    provider_name: &'static str,
) -> Result<reqwest::Response> {
    crate::providers::http::require_success(response, provider_name).await
}
