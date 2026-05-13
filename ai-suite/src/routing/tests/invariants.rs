use super::super::*;
use crate::Error;

#[test]
fn route_errors_when_primary_ollama_model_is_missing() {
    let router = super::support::router_with_models(Vec::new());

    let error = router
        .route("private local only: summarize this note")
        .expect_err("missing primary Ollama entry should fail");

    match error {
        Error::Routing { message } => {
            assert!(message.contains("primary Ollama model"), "got: {message}");
            assert!(message.contains(PRIMARY_OLLAMA_MODEL), "got: {message}");
        }
        other => panic!("expected Routing error, got {other:?}"),
    }
}

#[test]
fn explain_propagates_missing_primary_ollama_model_error() {
    let router = super::support::router_with_models(Vec::new());

    let error = router
        .explain("private local only: summarize this note")
        .expect_err("missing primary Ollama entry should fail");

    match error {
        Error::Routing { message } => {
            assert!(message.contains(PRIMARY_OLLAMA_MODEL), "got: {message}");
        }
        other => panic!("expected Routing error, got {other:?}"),
    }
}
