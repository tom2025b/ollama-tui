use super::super::models::{
    OllamaModel, ensure_model_name_is_available, model_name_matches_request,
};
use crate::Error;

#[test]
fn model_name_match_accepts_latest_tag() {
    assert!(model_name_matches_request("llama3:latest", "llama3"));
}

#[test]
fn available_model_check_accepts_latest_tag() {
    let installed_models = vec![OllamaModel {
        name: "llama3:latest".to_string(),
    }];

    ensure_model_name_is_available(&installed_models, "llama3")
        .expect("llama3:latest should satisfy llama3");
}

#[test]
fn available_model_check_explains_missing_model() {
    let installed_models = vec![OllamaModel {
        name: "mistral:latest".to_string(),
    }];

    let error = ensure_model_name_is_available(&installed_models, "llama3")
        .expect_err("missing llama3 should be explained");

    match error {
        Error::ProviderResponse { provider, message } => {
            assert_eq!(provider, "Ollama");
            assert!(message.contains("ollama pull llama3"), "got: {message}");
        }
        other => panic!("expected ProviderResponse error, got {other:?}"),
    }
}
