use super::super::*;
use super::support::{disabled_model, enabled_model, router_with_models};
use crate::providers::{anthropic, openai, xai};

#[test]
fn simple_prompt_chooses_fast_ollama_model() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
    ]);

    let decision = router.route("quick summary").unwrap();

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

    let decision = router.route("quick summary").unwrap();

    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}

#[test]
fn code_prompt_prefers_claude_when_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
    ]);

    let decision = router
        .route("debug this Rust compile error and explain the fix")
        .unwrap();

    assert_eq!(decision.model.provider, Provider::Anthropic);
}

#[test]
fn code_prompt_falls_back_when_claude_is_disabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        disabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
    ]);

    let decision = router
        .route("debug this Rust compile error and explain the fix")
        .unwrap();

    assert_eq!(decision.model.provider, Provider::OpenAi);
}

#[test]
fn current_context_prompt_prefers_grok_when_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::Xai, xai::DEFAULT_XAI_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
    ]);

    let decision = router
        .route("what is the latest public debate around AI policy")
        .unwrap();

    assert_eq!(decision.model.provider, Provider::Xai);
}
