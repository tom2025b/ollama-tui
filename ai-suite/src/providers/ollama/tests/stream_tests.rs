use super::super::stream::process_ollama_stream_line;

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
