use crate::providers::execution::{ModelRequest, stream_model_request};
use crate::routing::{ModelRouter, Router};
use crate::{
    Error, Result,
    llm::{ConversationTurn, LanguageModel, Provider},
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
        id: model_id_for(model),
        provider: model.provider.label().to_string(),
        name: model.name.clone(),
        label: model.display_label(),
        strengths: model.strengths.clone(),
        enabled: model.enabled,
        disabled_reason: model.disabled_reason.clone(),
    }
}

fn model_id_for(model: &LanguageModel) -> String {
    format!("{}:{}", provider_id(&model.provider), model.name)
}

fn provider_id(provider: &Provider) -> &'static str {
    match provider {
        Provider::Ollama => "ollama",
        Provider::Anthropic => "anthropic",
        Provider::OpenAi => "openai",
        Provider::Xai => "xai",
    }
}

fn select_model_by_id(models: &[LanguageModel], model_id: &str) -> Result<LanguageModel> {
    let model = models
        .iter()
        .find(|model| model_id_for(model) == model_id)
        .cloned()
        .ok_or_else(|| Error::validation(format!("unknown model selection: {model_id}")))?;

    if !model.enabled {
        let reason = model
            .disabled_reason
            .clone()
            .unwrap_or_else(|| "model is disabled".to_string());
        return Err(Error::validation(format!(
            "{} is unavailable: {reason}",
            model.display_label()
        )));
    }

    Ok(model)
}

fn format_route_prompt(explanation: Result<crate::routing::RouteExplanation>) -> String {
    match explanation {
        Ok(explanation) => explanation.format(),
        Err(error) => format!("Routing failed: {}", crate::friendly_error(&error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::RouteExplanation;

    #[test]
    fn select_model_by_id_rejects_unknown_selection() {
        let models = vec![LanguageModel::ollama("llama3", &["test"])];

        let error = select_model_by_id(&models, "ollama:missing")
            .expect_err("unknown model id should fail");

        match error {
            Error::Validation { message } => {
                assert!(
                    message.contains("unknown model selection"),
                    "got: {message}"
                );
                assert!(message.contains("ollama:missing"), "got: {message}");
            }
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn select_model_by_id_rejects_disabled_model() {
        let models = vec![LanguageModel::openai(
            "gpt-4o",
            &["test"],
            false,
            Some("missing key".to_string()),
        )];

        let error =
            select_model_by_id(&models, "openai:gpt-4o").expect_err("disabled model should fail");

        match error {
            Error::Validation { message } => {
                assert!(message.contains("OpenAI gpt-4o"), "got: {message}");
                assert!(message.contains("missing key"), "got: {message}");
            }
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn format_route_prompt_uses_friendly_error_for_failures() {
        let rendered = format_route_prompt(Err(Error::routing("router invariant broke")));
        assert!(rendered.contains("Routing failed:"), "got: {rendered}");
        assert!(
            rendered.contains("router invariant broke"),
            "got: {rendered}"
        );
    }

    #[test]
    fn format_route_prompt_renders_route_explanation() {
        let explanation = RouteExplanation {
            decision: crate::llm::RouteDecision {
                model: LanguageModel::ollama("llama3", &["test"]),
                reason: "picked local".to_string(),
            },
            matched_rule: "default",
            features: vec![("needs_privacy", false), ("is_simple", true)],
        };

        let rendered = format_route_prompt(Ok(explanation));
        assert!(rendered.contains("Routing trace"), "got: {rendered}");
        assert!(rendered.contains("llama3"), "got: {rendered}");
    }
}
