use super::ChatCompletionsClient;
use super::stream::process_chat_completion_stream_line;
use super::types::chat_messages_from_context;
use crate::Error;
use crate::llm::ConversationTurn;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

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
fn stream_line_emits_chat_completion_delta() {
    let mut answer = String::new();
    let mut tokens = Vec::new();

    process_chat_completion_stream_line(
        "test provider",
        r#"data: {"choices":[{"delta":{"content":"hello"}}]}"#,
        &mut answer,
        &mut |token| tokens.push(token),
    )
    .expect("stream line should parse");

    assert_eq!(answer, "hello");
    assert_eq!(tokens, vec!["hello"]);
}

#[tokio::test]
async fn test_stream_error_propagates() {
    let (api_url, server) = start_truncated_stream_server();
    let client = ChatCompletionsClient::for_test("OpenAI", api_url).expect("client should build");
    let mut tokens = Vec::new();

    let err = client
        .stream("gpt-4o", &[], "Hello", |token| tokens.push(token))
        .await
        .expect_err("expected streaming error to propagate");

    server.join().expect("server thread should finish cleanly");

    assert_eq!(tokens, vec!["hello"]);

    match err {
        Error::Streaming { provider, message } => {
            assert_eq!(provider, "OpenAI");
            assert!(
                message.contains("failed to read stream chunk"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected Streaming error, got {other:?}"),
    }
}

fn start_truncated_stream_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let address = listener.local_addr().expect("listener should have address");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("server should accept one client");
        read_request_headers(&mut stream);

        let body = "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n";
        let truncated_length = body.len() + 32;

        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {truncated_length}\r\nconnection: close\r\n\r\n{body}"
        )
        .expect("server should write response");
        stream.flush().expect("server should flush response");
    });

    (format!("http://{address}"), server)
}

fn read_request_headers(stream: &mut TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("server should set a read timeout");

    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];

    while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
        let read = stream.read(&mut chunk).expect("server should read request");
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
    }
}
