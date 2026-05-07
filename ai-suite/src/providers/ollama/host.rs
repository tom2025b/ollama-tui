/// Default Ollama host.
const DEFAULT_OLLAMA_HOST: &str = "http://localhost:11434";

/// Environment variable used by Ollama itself to point clients at a server.
pub(super) const OLLAMA_HOST_ENV: &str = "OLLAMA_HOST";

pub(super) fn default_host() -> &'static str {
    DEFAULT_OLLAMA_HOST
}

/// Normalize user-provided Ollama host values into a URL base.
pub(super) fn normalize_host(raw_host: String) -> String {
    let trimmed = raw_host.trim().trim_end_matches('/').to_string();

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed
    } else {
        format!("http://{trimmed}")
    }
}
