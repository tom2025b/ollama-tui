//! Centralized error definitions and user-facing error rendering.
//!
//! The long-term contract for this crate is that public fallible APIs return
//! [`Result<T>`](Result) with the root [`Error`] enum defined here.
//! [`friendly_error`] intentionally accepts any standard error so callers can
//! get the same user-facing rendering for typed errors and standard error
//! chains.

use std::{
    env::VarError,
    error::Error as StdError,
    path::PathBuf,
    str::Utf8Error,
    sync::atomic::{AtomicBool, Ordering},
};

use reqwest::StatusCode;
use thiserror::Error as ThisError;

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// Canonical result type for fallible public APIs in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Centralized error type for all production code paths in `ai-suite`.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Filesystem I/O failed for a path-aware operation.
    #[error("I/O error while {operation} at {}: {source}", path.display())]
    Io {
        /// Short description of the operation being attempted.
        operation: &'static str,
        /// Path involved in the failing I/O operation.
        path: PathBuf,
        /// Underlying operating-system error.
        #[source]
        source: std::io::Error,
    },

    /// Filesystem I/O failed for an operation without a single stable path.
    #[error("I/O error while {operation}: {source}")]
    IoOperation {
        /// Short description of the operation being attempted.
        operation: &'static str,
        /// Underlying operating-system error.
        #[source]
        source: std::io::Error,
    },

    /// A required environment variable was missing or invalid.
    #[error("environment variable `{name}` is not set or invalid: {source}")]
    EnvVar {
        /// Environment variable name.
        name: &'static str,
        /// Source error returned by the standard library.
        #[source]
        source: VarError,
    },

    /// A cloud provider requires an API key that is not configured.
    #[error("{provider} backend requires the `{env_var}` environment variable")]
    MissingApiKey {
        /// Human-readable provider name.
        provider: &'static str,
        /// Environment variable that must be exported.
        env_var: &'static str,
    },

    /// Building the HTTP client for a provider failed.
    #[error("failed to build {provider} HTTP client: {source}")]
    HttpClientBuild {
        /// Human-readable provider name.
        provider: &'static str,
        /// Source error returned by `reqwest`.
        #[source]
        source: reqwest::Error,
    },

    /// Sending an HTTP request to a provider failed before a valid response.
    #[error("failed to contact {provider}: {source}")]
    HttpRequest {
        /// Human-readable provider name.
        provider: &'static str,
        /// Source error returned by `reqwest`.
        #[source]
        source: reqwest::Error,
    },

    /// A provider returned a non-success HTTP status.
    #[error("{provider} returned HTTP {status}. Response body: {body}")]
    HttpStatus {
        /// Human-readable provider name.
        provider: &'static str,
        /// HTTP status code returned by the backend.
        status: StatusCode,
        /// Provider response body, when available.
        body: String,
    },

    /// JSON parsing or serialization failed with call-site context.
    #[error("failed to process JSON for {context}: {source}")]
    Json {
        /// Short description of the JSON boundary.
        context: &'static str,
        /// Source error returned by `serde_json`.
        #[source]
        source: serde_json::Error,
    },

    /// TOML parsing failed with call-site context.
    #[error("failed to parse TOML for {context}: {source}")]
    TomlDeserialize {
        /// Short description of the TOML boundary.
        context: &'static str,
        /// Source error returned by `toml`.
        #[source]
        source: toml::de::Error,
    },

    /// TOML serialization failed with call-site context.
    #[error("failed to serialize TOML for {context}: {source}")]
    TomlSerialize {
        /// Short description of the TOML boundary.
        context: &'static str,
        /// Source error returned by `toml`.
        #[source]
        source: toml::ser::Error,
    },

    /// UTF-8 decoding failed, usually while consuming provider streams.
    #[error("invalid UTF-8 while {context}: {source}")]
    Utf8 {
        /// Short description of the decoding boundary.
        context: &'static str,
        /// Source error returned by `std::str`.
        #[source]
        source: Utf8Error,
    },

    /// A provider returned malformed or semantically invalid data.
    #[error("{provider} returned an invalid response: {message}")]
    ProviderResponse {
        /// Human-readable provider name.
        provider: &'static str,
        /// Additional response-specific context.
        message: String,
    },

    /// A provider stream failed after the request was accepted.
    #[error("{provider} streaming error: {message}")]
    Streaming {
        /// Human-readable provider name.
        provider: &'static str,
        /// Additional streaming-specific context.
        message: String,
    },

    /// Runtime configuration is invalid or incomplete.
    #[error("configuration error: {message}")]
    Configuration {
        /// Human-readable explanation of the configuration problem.
        message: String,
    },

    /// Model selection or routing failed.
    #[error("routing error: {message}")]
    Routing {
        /// Human-readable explanation of the routing problem.
        message: String,
    },

    /// Tool execution failed.
    #[error("tool error: {message}")]
    Tool {
        /// Human-readable explanation of the tool failure.
        message: String,
    },

    /// Extension integration failed.
    #[error("extension error: {message}")]
    Extension {
        /// Human-readable explanation of the extension failure.
        message: String,
    },

    /// User input or internal arguments failed validation.
    #[error("validation error: {message}")]
    Validation {
        /// Human-readable explanation of the validation failure.
        message: String,
    },

    /// Terminal or UI state management failed.
    #[error("terminal error: {message}")]
    Terminal {
        /// Human-readable explanation of the terminal failure.
        message: String,
    },

    /// An internal invariant was violated.
    #[error("internal invariant violated: {message}")]
    Invariant {
        /// Human-readable explanation of the invariant failure.
        message: String,
    },
}

impl Error {
    /// Build a path-aware I/O error.
    pub fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    /// Build an I/O error for operations without a single stable path.
    pub fn io_operation(operation: &'static str, source: std::io::Error) -> Self {
        Self::IoOperation { operation, source }
    }

    /// Build an environment-variable error with the variable name attached.
    pub fn env_var(name: &'static str, source: VarError) -> Self {
        Self::EnvVar { name, source }
    }

    /// Build a missing-provider-key error.
    pub fn missing_api_key(provider: &'static str, env_var: &'static str) -> Self {
        Self::MissingApiKey { provider, env_var }
    }

    /// Build an HTTP-client construction error for a provider.
    pub fn http_client_build(provider: &'static str, source: reqwest::Error) -> Self {
        Self::HttpClientBuild { provider, source }
    }

    /// Build an HTTP transport error for a provider.
    pub fn http_request(provider: &'static str, source: reqwest::Error) -> Self {
        Self::HttpRequest { provider, source }
    }

    /// Build a non-success HTTP status error for a provider.
    pub fn http_status(
        provider: &'static str,
        status: StatusCode,
        body: impl Into<String>,
    ) -> Self {
        Self::HttpStatus {
            provider,
            status,
            body: body.into(),
        }
    }

    /// Build a JSON processing error with call-site context.
    pub fn json(context: &'static str, source: serde_json::Error) -> Self {
        Self::Json { context, source }
    }

    /// Build a TOML parse error with call-site context.
    pub fn toml_deserialize(context: &'static str, source: toml::de::Error) -> Self {
        Self::TomlDeserialize { context, source }
    }

    /// Build a TOML serialization error with call-site context.
    pub fn toml_serialize(context: &'static str, source: toml::ser::Error) -> Self {
        Self::TomlSerialize { context, source }
    }

    /// Build a UTF-8 decoding error with call-site context.
    pub fn utf8(context: &'static str, source: Utf8Error) -> Self {
        Self::Utf8 { context, source }
    }

    /// Build an invalid-provider-response error.
    pub fn provider_response(provider: &'static str, message: impl Into<String>) -> Self {
        Self::ProviderResponse {
            provider,
            message: message.into(),
        }
    }

    /// Build a provider-streaming error.
    pub fn streaming(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Streaming {
            provider,
            message: message.into(),
        }
    }

    /// Build a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Build a routing error.
    pub fn routing(message: impl Into<String>) -> Self {
        Self::Routing {
            message: message.into(),
        }
    }

    /// Build a tool-execution error.
    pub fn tool(message: impl Into<String>) -> Self {
        Self::Tool {
            message: message.into(),
        }
    }

    /// Build an extension error.
    pub fn extension(message: impl Into<String>) -> Self {
        Self::Extension {
            message: message.into(),
        }
    }

    /// Build a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Build a terminal/UI error.
    pub fn terminal(message: impl Into<String>) -> Self {
        Self::Terminal {
            message: message.into(),
        }
    }

    /// Build an internal-invariant error.
    pub fn invariant(message: impl Into<String>) -> Self {
        Self::Invariant {
            message: message.into(),
        }
    }
}

/// Initialize debug mode from the `AI_SUITE_DEBUG` env var. Call once at
/// startup. Anything truthy (`1`, `true`, `yes`, on any case) enables debug
/// output for the rest of the process.
pub fn init_debug_mode_from_env() {
    if let Ok(value) = std::env::var("AI_SUITE_DEBUG") {
        let on = matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        );
        DEBUG_MODE.store(on, Ordering::Relaxed);
    }
}

/// Toggle the in-process debug flag. Used by `/debug` in the TUI. Returns the
/// new value.
pub fn toggle_debug_mode() -> bool {
    let new = !DEBUG_MODE.load(Ordering::Relaxed);
    DEBUG_MODE.store(new, Ordering::Relaxed);
    new
}

/// Read the current debug flag.
pub fn debug_mode_enabled() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

/// Render an error chain for a user. In debug mode, returns the full chain via
/// `{:#}`. Otherwise, returns a single short sentence with a recovery hint.
pub fn friendly_error(error: &(dyn StdError + 'static)) -> String {
    if debug_mode_enabled() {
        return format!("{error:#}");
    }

    let chain_text = chain_text_std(error);
    classify(&chain_text).unwrap_or_else(|| short_summary(error))
}

/// Concatenate every `.context()` layer into one lowercase string for matching.
fn chain_text_std(error: &(dyn StdError + 'static)) -> String {
    let mut buffer = String::new();
    let mut current = Some(error);

    while let Some(cause) = current {
        if !buffer.is_empty() {
            buffer.push_str(" :: ");
        }
        buffer.push_str(&cause.to_string());
        current = cause.source();
    }

    buffer.to_lowercase()
}

/// Pull just the top-level message when no specific pattern matches.
fn short_summary<E>(error: &E) -> String
where
    E: std::fmt::Display + ?Sized,
{
    let head = error.to_string();
    let head = head.lines().next().unwrap_or("").trim();
    if head.is_empty() {
        "Something went wrong. Re-run with AI_SUITE_DEBUG=1 for details.".to_string()
    } else {
        format!("{head}. Re-run with AI_SUITE_DEBUG=1 (or type /debug) for the full error.")
    }
}

/// Try to recognise a known failure mode in the chain and return a tailored
/// message. Returns `None` when nothing specific matches.
fn classify(chain: &str) -> Option<String> {
    // --- Provider connection failures ------------------------------------
    if chain.contains("failed to contact ollama")
        || (chain.contains("ollama") && chain.contains("connection refused"))
    {
        return Some(
            "Can't reach Ollama. Start it with `ollama serve`, or set OLLAMA_HOST if it's on another machine."
                .into(),
        );
    }
    if chain.contains("failed to contact anthropic") {
        return Some(
            "Can't reach Anthropic right now. Check your internet connection and try again.".into(),
        );
    }
    if chain.contains("failed to contact openai") {
        return Some(
            "Can't reach OpenAI right now. Check your internet connection and try again.".into(),
        );
    }
    if chain.contains("failed to contact xai") {
        return Some(
            "Can't reach xAI right now. Check your internet connection and try again.".into(),
        );
    }

    // --- Missing API keys -------------------------------------------------
    if let Some(env_var) = detect_missing_api_key(chain) {
        let provider = match env_var {
            "ANTHROPIC_API_KEY" => "Claude",
            "OPENAI_API_KEY" => "GPT",
            "XAI_API_KEY" => "Grok",
            _ => "this provider",
        };
        return Some(format!(
            "{env_var} is not set, so {provider} can't be used. Export it (e.g. `export {env_var}=...`) and try again. The router will fall back to local Ollama if available."
        ));
    }

    // --- HTTP status codes from any provider -----------------------------
    if let Some(provider) = detect_provider_with_http(chain, &["401", "403"]) {
        return Some(format!(
            "{provider} rejected the API key (HTTP 401/403). Double-check the key is valid and has credit."
        ));
    }
    if let Some(provider) = detect_provider_with_http(chain, &["429"]) {
        return Some(format!(
            "{provider} rate-limited the request (HTTP 429). Wait a moment and try again."
        ));
    }
    if let Some(provider) = detect_provider_with_http(chain, &["500", "502", "503", "504"]) {
        return Some(format!(
            "{provider} is having trouble right now (server error). Try again in a moment."
        ));
    }
    if chain.contains("ollama returned http 404") {
        return Some(
            "Ollama is running but rejected the request (404). The model name may be wrong or the API path changed."
                .into(),
        );
    }
    if let Some(provider) = detect_provider_with_http(chain, &["404"]) {
        return Some(format!(
            "{provider} returned 404. The model name may not exist on this account."
        ));
    }

    // --- Ollama model availability ---------------------------------------
    if chain.contains("no local models are installed") {
        return Some(
            "Ollama is running, but you have no local models. Install one with `ollama pull llama3` (or another model)."
                .into(),
        );
    }
    if chain.contains("is not installed. installed models") {
        return Some(
            "That Ollama model isn't installed locally. Run `ollama pull <model>` to fetch it, or pick a different one with /model."
                .into(),
        );
    }

    // --- Timeouts ---------------------------------------------------------
    if chain.contains("timed out") || chain.contains("operation timed out") {
        return Some(
            "The request timed out. The model may be overloaded or the prompt may be too large. Try again or shorten the prompt."
                .into(),
        );
    }

    // --- Context / token-limit hints -------------------------------------
    if chain.contains("context length")
        || chain.contains("maximum context")
        || chain.contains("context window")
        || chain.contains("too many tokens")
    {
        return Some(
            "The conversation is too long for this model. Use /clear to drop history, or pick a model with a larger context window."
                .into(),
        );
    }

    None
}

fn detect_missing_api_key(chain: &str) -> Option<&'static str> {
    for env_var in ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "XAI_API_KEY"] {
        let needle = env_var.to_lowercase();
        if chain.contains(&needle)
            && (chain.contains("environment variable") || chain.contains("not set"))
        {
            return Some(env_var);
        }
    }
    None
}

fn detect_provider_with_http(chain: &str, codes: &[&str]) -> Option<&'static str> {
    let provider = if chain.contains("anthropic") {
        "Anthropic"
    } else if chain.contains("openai") {
        "OpenAI"
    } else if chain.contains("xai") {
        "xAI"
    } else if chain.contains("ollama") {
        "Ollama"
    } else {
        return None;
    };

    let mentions_code = codes
        .iter()
        .any(|code| chain.contains(&format!("http {code}")));

    mentions_code.then_some(provider)
}

#[cfg(test)]
mod tests {
    //! Tests target the pure helpers (`classify`, `short_summary`,
    //! `chain_text_std`) to avoid racing on the process-wide `DEBUG_MODE` flag.

    use super::*;

    #[derive(Debug)]
    struct TestError {
        message: &'static str,
        source: Option<Box<dyn StdError + Send + Sync>>,
    }

    impl TestError {
        fn new(message: &'static str) -> Self {
            Self {
                message,
                source: None,
            }
        }

        fn with_source(
            message: &'static str,
            source: impl StdError + Send + Sync + 'static,
        ) -> Self {
            Self {
                message,
                source: Some(Box::new(source)),
            }
        }
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.message)
        }
    }

    impl StdError for TestError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            self.source
                .as_deref()
                .map(|source| source as &(dyn StdError + 'static))
        }
    }

    fn err(msg: &'static str) -> TestError {
        TestError::new(msg)
    }

    fn classify_of(error: &TestError) -> Option<String> {
        classify(&chain_text_std(error))
    }

    #[test]
    fn translates_ollama_connection_failure() {
        let e = TestError::with_source(
            "failed to contact Ollama at http://127.0.0.1:11434/api/chat",
            err("connection refused"),
        );
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("ollama serve"), "got: {out}");
    }

    #[test]
    fn translates_missing_api_key() {
        let e = TestError::with_source(
            "Anthropic backend requires the `ANTHROPIC_API_KEY` environment variable",
            err("environment variable not found"),
        );
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("ANTHROPIC_API_KEY"), "got: {out}");
        assert!(out.contains("Claude"), "got: {out}");
    }

    #[test]
    fn translates_http_401_for_openai() {
        let e =
            err("OpenAI returned HTTP 401 Unauthorized. Response body: {\"error\":\"bad key\"}");
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("OpenAI"), "got: {out}");
        assert!(out.contains("rejected"), "got: {out}");
    }

    #[test]
    fn translates_ollama_missing_model() {
        let e = err(
            "Ollama model `phi3` is not installed. Installed models: llama3:latest. Run `ollama pull phi3`.",
        );
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("ollama pull"), "got: {out}");
    }

    #[test]
    fn translates_rate_limit() {
        let e = err("xAI returned HTTP 429 Too Many Requests. Response body: ...");
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("rate-limited"), "got: {out}");
        assert!(out.contains("xAI"), "got: {out}");
    }

    #[test]
    fn translates_context_window_overflow() {
        let e = err("openai returned: maximum context length is 8192 tokens");
        let out = classify_of(&e).expect("should classify");
        assert!(
            out.contains("/clear") || out.contains("too long"),
            "got: {out}"
        );
    }

    #[test]
    fn unknown_error_returns_none_from_classify() {
        let e = err("the disk fell off");
        assert!(classify_of(&e).is_none());
    }

    #[test]
    fn short_summary_includes_message_and_debug_hint() {
        let e = err("the disk fell off");
        let out = short_summary(&e);
        assert!(out.contains("the disk fell off"), "got: {out}");
        assert!(out.contains("AI_SUITE_DEBUG"), "got: {out}");
    }

    #[test]
    fn typed_missing_api_key_is_rendered() {
        let e = Error::missing_api_key("OpenAI", "OPENAI_API_KEY");
        let out = friendly_error(&e);
        assert!(out.contains("OPENAI_API_KEY"), "got: {out}");
        assert!(out.contains("GPT"), "got: {out}");
    }

    #[test]
    fn typed_http_status_is_classified() {
        let e = Error::http_status("OpenAI", StatusCode::UNAUTHORIZED, "bad key");
        let out = friendly_error(&e);
        assert!(out.contains("OpenAI"), "got: {out}");
        assert!(out.contains("rejected"), "got: {out}");
    }
}
