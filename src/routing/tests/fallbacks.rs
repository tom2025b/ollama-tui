use super::super::*;
use super::support::{disabled_model, enabled_model, router_with_models};
use crate::runtime::{DEFAULT_CLAUDE_CODE_MODEL, DEFAULT_CODEX_MODEL};

#[test]
fn simple_prompt_chooses_fast_ollama_model() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
    ]);

    let decision = router.route("quick summary");

    assert_eq!(decision.model.name, DEFAULT_FAST_OLLAMA_MODEL);
}

#[test]
fn default_fast_model_uses_installed_llama3_latest_tag() {
    assert_eq!(DEFAULT_FAST_OLLAMA_MODEL, "llama3:latest");
}

#[test]
fn fast_ollama_model_can_match_primary_model_name() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
    ]);

    let decision = router.route("quick summary");

    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}

#[test]
fn code_prompt_prefers_claude_when_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::ClaudeCode, DEFAULT_CLAUDE_CODE_MODEL),
        enabled_model(Provider::Codex, DEFAULT_CODEX_MODEL),
    ]);

    let decision = router.route("debug this Rust compile error and explain the fix");

    assert_eq!(decision.model.provider, Provider::ClaudeCode);
}

#[test]
fn code_prompt_falls_back_when_claude_is_disabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        disabled_model(Provider::ClaudeCode, DEFAULT_CLAUDE_CODE_MODEL),
        enabled_model(Provider::Codex, DEFAULT_CODEX_MODEL),
    ]);

    let decision = router.route("debug this Rust compile error and explain the fix");

    assert_eq!(decision.model.provider, Provider::Codex);
}

#[test]
fn short_complex_prompt_prefers_claude_when_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::ClaudeCode, DEFAULT_CLAUDE_CODE_MODEL),
        enabled_model(Provider::Codex, "codex-mini-latest"),
    ]);

    let decision = router
        .route("complex: compare architectures and recommend a careful implementation approach");

    assert_eq!(decision.model.provider, Provider::ClaudeCode);
}

#[test]
fn short_complex_prompt_falls_back_to_codex_model_when_claude_is_disabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        disabled_model(Provider::ClaudeCode, DEFAULT_CLAUDE_CODE_MODEL),
        enabled_model(Provider::Codex, "codex-mini-latest"),
    ]);

    let decision = router
        .route("complex: compare architectures and recommend a careful implementation approach");

    assert_eq!(decision.model.provider, Provider::Codex);
    assert_eq!(decision.model.name, "codex-mini-latest");
}

#[test]
fn current_context_prompt_prefers_codex_when_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::Codex, DEFAULT_CODEX_MODEL),
    ]);

    let decision = router.route("what is the latest public debate around AI policy");

    assert_eq!(decision.model.provider, Provider::Codex);
}
