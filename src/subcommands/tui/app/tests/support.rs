use super::super::ChatMessage;

pub(super) fn completed_message(number: usize) -> ChatMessage {
    ChatMessage {
        prompt: format!("prompt {number}"),
        model_name: "Ollama llama3".to_string(),
        route_reason: "test route".to_string(),
        answer: format!("answer {number}"),
        in_progress: false,
        failed: false,
        include_in_context: true,
    }
}
