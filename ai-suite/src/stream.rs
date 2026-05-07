use anyhow::Result;

use crate::llm::ConversationTurn;
use crate::providers::execution::{ModelRequest, stream_model_request};
use crate::routing::{ModelRouter, Router};

/// Public entry point for sending a prompt through the full routing + provider
/// pipeline without going through the TUI.
///
/// Returns (full_response_text, model_name) so callers can show which model
/// was chosen by the router.
pub async fn stream_prompt(
    prompt: String,
    context: Vec<ConversationTurn>,
    on_token: impl FnMut(String) + Send,
) -> Result<(String, String)> {
    let runtime = crate::runtime::Runtime::load();
    let router = ModelRouter::new(runtime.config().models());
    let decision = router.route(&prompt);
    let model_name = decision.model.name.clone();
    let request = ModelRequest::new(decision.model, context, prompt);
    let full_text = stream_model_request(&request, on_token).await?;
    Ok((full_text, model_name))
}
