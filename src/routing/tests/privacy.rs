use super::super::*;
use super::support::{enabled_model, router_with_models};
use crate::runtime::{DEFAULT_CLAUDE_CODE_MODEL, DEFAULT_CODEX_MODEL};

#[test]
fn privacy_prompt_stays_on_primary_ollama() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::Codex, DEFAULT_CODEX_MODEL),
    ]);

    let decision = router.route("private local only: summarize this personal note");

    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}

#[test]
fn sensitive_medical_prompt_stays_on_primary_ollama_even_when_terminal_apps_are_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::ClaudeCode, DEFAULT_CLAUDE_CODE_MODEL),
        enabled_model(Provider::Codex, DEFAULT_CODEX_MODEL),
    ]);

    let decision = router.route("summarize these medical records and draft an email");

    assert_eq!(decision.model.provider, Provider::Ollama);
    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}

#[test]
fn sensitive_credential_prompt_stays_on_primary_ollama_even_for_code() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::ClaudeCode, DEFAULT_CLAUDE_CODE_MODEL),
        enabled_model(Provider::Codex, DEFAULT_CODEX_MODEL),
    ]);

    let decision = router.route("debug this Python error; it includes my API key");

    assert_eq!(decision.model.provider, Provider::Ollama);
    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}
