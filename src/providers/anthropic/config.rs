use std::time::Duration;

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
