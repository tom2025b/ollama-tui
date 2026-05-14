use super::super::*;
use super::support::{ollama_model, router_with_models};

#[test]
fn simple_prompt_chooses_fast_ollama_model() {
    let router = router_with_models(vec![
        ollama_model(PRIMARY_OLLAMA_MODEL),
        ollama_model(DEFAULT_FAST_OLLAMA_MODEL),
    ]);

    let decision = router.route("quick summary").unwrap();

    assert_eq!(decision.model.name, DEFAULT_FAST_OLLAMA_MODEL);
}

#[test]
fn default_fast_model_uses_installed_llama3_latest_tag() {
    assert_eq!(DEFAULT_FAST_OLLAMA_MODEL, "llama3:latest");
}

#[test]
fn long_prompt_chooses_primary_ollama_model() {
    let router = router_with_models(vec![
        ollama_model(PRIMARY_OLLAMA_MODEL),
        ollama_model(DEFAULT_FAST_OLLAMA_MODEL),
    ]);

    let prompt = "word ".repeat(25);
    let decision = router.route(&prompt).unwrap();

    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}

#[test]
fn single_model_router_returns_that_model_for_any_prompt() {
    let router = router_with_models(vec![ollama_model(PRIMARY_OLLAMA_MODEL)]);

    let decision = router.route("quick summary").unwrap();

    assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
}
