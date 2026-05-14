use anyhow::anyhow;

use crate::providers::execution::{ModelRequest, stream_model_request};
use crate::routing::{ModelRouter, Router};
use crate::{
    Result,
    llm::{ConversationTurn, LanguageModel},
};

/// Model details safe for GUI and other public callers to display.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub label: String,
    pub strengths: Vec<String>,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

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
    let router = router_from_runtime();
    let decision = router.route(&prompt)?;
    let model_name = decision.model.name.clone();
    let request = ModelRequest::new(decision.model, context, prompt);
    let full_text = stream_model_request(&request, on_token).await?;
    Ok((full_text, model_name))
}

/// Stream a prompt through one explicitly selected enabled model.
pub async fn stream_prompt_with_model(
    model_id: String,
    prompt: String,
    context: Vec<ConversationTurn>,
    on_token: impl FnMut(String) + Send,
) -> Result<(String, String)> {
    let router = router_from_runtime();
    let model = select_model_by_id(router.models(), &model_id)?;

    let model_name = model.name.clone();
    let request = ModelRequest::new(model, context, prompt);
    let full_text = stream_model_request(&request, on_token).await?;
    Ok((full_text, model_name))
}

/// Return every model known to the runtime router.
pub fn available_models() -> Vec<ModelInfo> {
    router_from_runtime()
        .models()
        .iter()
        .map(model_info_for)
        .collect()
}

/// Explain the model the router would pick without calling a provider.
pub fn route_prompt(prompt: &str) -> String {
    format_route_prompt(router_from_runtime().explain(prompt))
}

fn router_from_runtime() -> ModelRouter {
    let runtime = crate::runtime::Runtime::load();
    ModelRouter::new(runtime.config().models())
}

fn model_info_for(model: &LanguageModel) -> ModelInfo {
    ModelInfo {
        id: format!("ollama:{}", model.name),
        provider: "Ollama".to_string(),
        name: model.name.clone(),
        label: model.display_label(),
        strengths: model.strengths.clone(),
        enabled: true,
        disabled_reason: None,
    }
}

fn select_model_by_id(models: &[LanguageModel], model_id: &str) -> Result<LanguageModel> {
    models
        .iter()
        .find(|model| format!("ollama:{}", model.name) == model_id)
        .cloned()
        .ok_or_else(|| anyhow!("unknown model selection: {model_id}"))
}

fn format_route_prompt(explanation: Result<crate::routing::RouteExplanation>) -> String {
    match explanation {
        Ok(explanation) => explanation.format(),
        Err(error) => format!("Routing failed: {}", crate::friendly_error(&error)),
    }
}
