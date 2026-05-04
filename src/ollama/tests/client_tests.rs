use super::super::client::OllamaClient;

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
