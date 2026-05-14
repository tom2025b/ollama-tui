use super::{ModelRouter, PromptProfile};
use crate::{Result, llm::RouteDecision};

const SIMPLE_REASON: &str =
    "This is short or simple, so I chose the fast local Ollama model.";
const DEFAULT_REASON: &str =
    "Routing to the primary local Ollama model.";

impl ModelRouter {
    /// Choose a model for the user's prompt.
    pub(super) fn route_with_rules(&self, prompt: &str) -> Result<RouteDecision> {
        let profile = PromptProfile::from_prompt(prompt);

        if profile.is_simple {
            let name = self.fast_ollama_model_name();
            return self.choose_by_name(&name, SIMPLE_REASON);
        }

        self.choose_by_name(super::catalog::PRIMARY_OLLAMA_MODEL, DEFAULT_REASON)
    }

    fn choose_by_name(&self, name: &str, reason: &str) -> Result<RouteDecision> {
        let model = self
            .models
            .iter()
            .find(|m| m.name == name)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| self.primary_ollama_model())?;

        Ok(RouteDecision {
            model,
            reason: reason.to_string(),
        })
    }
}
