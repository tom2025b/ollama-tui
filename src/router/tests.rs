use super::*;
use crate::{anthropic, openai, xai};

fn enabled_model(provider: Provider, name: &str) -> LanguageModel {
    match provider {
        Provider::Ollama => LanguageModel::ollama(name, &["test"]),
        Provider::Anthropic => LanguageModel::anthropic(name, &["test"], true, None),
        Provider::OpenAi => LanguageModel::openai(name, &["test"], true, None),
        Provider::Xai => LanguageModel::xai(name, &["test"], true, None),
    }
}

fn disabled_model(provider: Provider, name: &str) -> LanguageModel {
    match provider {
        Provider::Ollama => LanguageModel::ollama(name, &["test"]),
        Provider::Anthropic => {
            LanguageModel::anthropic(name, &["test"], false, Some("missing key".to_string()))
        }
        Provider::OpenAi => {
            LanguageModel::openai(name, &["test"], false, Some("missing key".to_string()))
        }
        Provider::Xai => {
            LanguageModel::xai(name, &["test"], false, Some("missing key".to_string()))
        }
    }
}

fn router_with_models(models: Vec<LanguageModel>) -> ModelRouter {
    ModelRouter { models }
}

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
fn code_prompt_prefers_claude_when_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
    ]);

    let decision = router.route("debug this Rust compile error and explain the fix");

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

    let decision = router.route("debug this Rust compile error and explain the fix");

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

    let decision = router.route("what is the latest public debate around AI policy");

    assert_eq!(decision.model.provider, Provider::Xai);
}

#[test]
fn privacy_prompt_stays_on_primary_ollama() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
    ]);

    let decision = router.route("private local only: summarize this personal note");

    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}

#[test]
fn sensitive_medical_prompt_stays_on_primary_ollama_even_when_cloud_is_enabled() {
    let router = router_with_models(vec![
        enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
        enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
        enabled_model(Provider::Xai, xai::DEFAULT_XAI_MODEL),
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
        enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
        enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
    ]);

    let decision = router.route("debug this Python error; it includes my API key");

    assert_eq!(decision.model.provider, Provider::Ollama);
    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}
