//! Translate `anyhow::Error` chains into short, actionable messages for users.
//!
//! Internal code keeps using `anyhow::Result` and rich `.context()` chains. At
//! the display boundary (TUI failure events, top-level CLI exit), call
//! [`friendly_error`] to convert the chain into one human-readable line.
//!
//! Set `AI_SUITE_DEBUG=1` (or toggle `/debug` in the TUI) to print the full
//! technical chain instead.

use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

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
pub fn friendly_error(error: &anyhow::Error) -> String {
    if debug_mode_enabled() {
        return format!("{error:#}");
    }

    let chain_text = chain_text(error);
    classify(&chain_text).unwrap_or_else(|| short_summary(error))
}

/// Concatenate every `.context()` layer into one lowercase string for matching.
fn chain_text(error: &anyhow::Error) -> String {
    let mut buffer = String::new();
    for cause in error.chain() {
        if !buffer.is_empty() {
            buffer.push_str(" :: ");
        }
        buffer.push_str(&cause.to_string());
    }
    buffer.to_lowercase()
}

/// Pull just the top-level message when no specific pattern matches.
fn short_summary(error: &anyhow::Error) -> String {
    let head = error.to_string();
    let head = head.lines().next().unwrap_or("").trim();
    if head.is_empty() {
        "Something went wrong. Re-run with AI_SUITE_DEBUG=1 for details.".to_string()
    } else {
        format!(
            "{head}. Re-run with AI_SUITE_DEBUG=1 (or type /debug) for the full error."
        )
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
    if chain.contains("ollama returned 404") {
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
        .any(|code| chain.contains(&format!("http {code}")) || chain.contains(&format!(" {code} ")));

    mentions_code.then_some(provider)
}

#[cfg(test)]
mod tests {
    //! Tests target the pure helpers (`classify`, `short_summary`, `chain_text`)
    //! to avoid racing on the process-wide `DEBUG_MODE` flag.

    use super::*;
    use anyhow::anyhow;

    fn err(msg: &str) -> anyhow::Error {
        anyhow!("{msg}")
    }

    fn classify_of(error: &anyhow::Error) -> Option<String> {
        classify(&chain_text(error))
    }

    #[test]
    fn translates_ollama_connection_failure() {
        let e = err("connection refused")
            .context("failed to contact Ollama at http://127.0.0.1:11434/api/chat");
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("ollama serve"), "got: {out}");
    }

    #[test]
    fn translates_missing_api_key() {
        let e = err("environment variable not found").context(
            "Anthropic backend requires the `ANTHROPIC_API_KEY` environment variable",
        );
        let out = classify_of(&e).expect("should classify");
        assert!(out.contains("ANTHROPIC_API_KEY"), "got: {out}");
        assert!(out.contains("Claude"), "got: {out}");
    }

    #[test]
    fn translates_http_401_for_openai() {
        let e = err("OpenAI returned HTTP 401 Unauthorized. Response body: {\"error\":\"bad key\"}");
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
        assert!(out.contains("/clear") || out.contains("too long"), "got: {out}");
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
}
