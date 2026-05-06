use super::super::ChatMessage;

pub(super) fn completed_message(number: usize) -> ChatMessage {
    let mut message = ChatMessage::streaming_model_turn(
        format!("prompt {number}"),
        "Ollama llama3".to_string(),
        "test route".to_string(),
    );
    message.append_token(&format!("answer {number}"));
    message.finish_streaming();
    message
}
