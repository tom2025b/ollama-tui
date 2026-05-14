//! Centralized error utilities and user-facing error rendering.
//!
//! `Error` and `Result` are re-exported from `anyhow`. Use `anyhow::anyhow!` to
//! construct errors and `anyhow::Context` (`.context()`) to add call-site context.
//! The `friendly_error` helper renders any error chain for end users.

use std::sync::atomic::{AtomicBool, Ordering};

pub use anyhow::{Context, Error, Result, anyhow};

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// Initialize debug mode from the `AI_SUITE_DEBUG` env var. Call once at
/// startup. Anything truthy (`1`, `true`, `yes`, `on` in any case) enables debug
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
pub fn friendly_error(error: &anyhow::Error) -> String {
    if debug_mode_enabled() {
        return format!("{error:#}");
    }

    let chain_text = chain_text(error);
    classify(&chain_text).unwrap_or_else(|| short_summary(error))
}

fn chain_text(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(|cause| cause.to_string())
        .collect::<Vec<_>>()
        .join(" :: ")
        .to_lowercase()
}

fn short_summary(error: &anyhow::Error) -> String {
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

fn detect_provider_with_http(chain: &str, codes: &[&str]) -> Option<&'static str> {
    if !chain.contains("ollama") {
        return None;
    }

    let mentions_code = codes
        .iter()
        .any(|code| chain.contains(&format!("http {code}")));

    mentions_code.then_some("Ollama")
}
