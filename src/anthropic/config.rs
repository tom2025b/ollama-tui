use std::{env, time::Duration};

/// Anthropic API key environment variable.
pub const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Optional model override for this app.
pub const ANTHROPIC_MODEL_ENV: &str = "ANTHROPIC_MODEL";

/// Stable Claude model used by default.
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-20250514";

/// Anthropic Messages API endpoint.
pub(super) const ANTHROPIC_MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";

/// Required API version header for Anthropic's Messages API.
pub(super) const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Maximum answer length for this first version.
pub(super) const MAX_TOKENS: u32 = 2048;

/// Cloud request timeout.
pub(super) const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Return the configured Claude model name.
pub fn configured_model_name() -> String {
    env::var(ANTHROPIC_MODEL_ENV).unwrap_or_else(|_| DEFAULT_ANTHROPIC_MODEL.to_string())
}

/// True when the Claude backend has enough local configuration to be selected.
pub fn is_configured() -> bool {
    env::var(ANTHROPIC_API_KEY_ENV).is_ok()
}

/// Explain how to enable this backend.
pub fn missing_configuration_reason() -> String {
    format!("set {ANTHROPIC_API_KEY_ENV} to enable Claude")
}
