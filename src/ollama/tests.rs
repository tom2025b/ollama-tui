use super::client::OllamaClient;
use super::host::normalize_host;
use super::models::{OllamaModel, ensure_model_name_is_available, model_name_matches_request};
use super::stream::process_ollama_stream_line;
use super::types::chat_messages_from_context;
use crate::llm::ConversationTurn;

#[test]
fn normalize_host_adds_http_scheme_when_missing() {
    assert_eq!(
        normalize_host("127.0.0.1:11434/".to_string()),
        "http://127.0.0.1:11434"
    );
}

#[test]
fn normalize_host_keeps_existing_scheme() {
    assert_eq!(
        normalize_host("https://example.test:11434/".to_string()),
        "https://example.test:11434"
    );
}

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
    assert!(error.to_string().contains("ollama pull llama3"));
}

#[test]
fn chat_messages_include_bounded_context_then_current_prompt() {
    let context = vec![ConversationTurn {
        user: "old prompt".to_string(),
        assistant: "old answer".to_string(),
    }];

    let messages = chat_messages_from_context(&context, "new prompt");

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "old prompt");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content, "old answer");
    assert_eq!(messages[2].role, "user");
    assert_eq!(messages[2].content, "new prompt");
}

#[test]
fn stream_line_emits_ollama_chat_content() {
    let mut answer = String::new();
    let mut tokens = Vec::new();

    process_ollama_stream_line(
        r#"{"message":{"role":"assistant","content":"hello"},"done":false}"#,
        &mut answer,
        &mut |token| tokens.push(token),
    )
    .expect("stream line should parse");

    assert_eq!(answer, "hello");
    assert_eq!(tokens, vec!["hello"]);
}

#[tokio::test]
#[ignore = "requires a running local Ollama server with llama3 installed"]
async fn live_ollama_stream_smoke_test() {
    let client = OllamaClient::from_environment().expect("client should build");
    let mut tokens = Vec::new();
    let answer = client
        .stream(
            "llama3",
            &[],
            "Reply with one short sentence confirming you are working.",
            |token| tokens.push(token),
        )
        .await
        .expect("local Ollama llama3 streaming should work");

    assert!(!answer.trim().is_empty());
    assert!(!tokens.is_empty());
}
