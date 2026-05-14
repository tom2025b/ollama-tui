mod catalog;
mod explain;
mod profile;
mod selection;

use crate::{
    Result,
    llm::{LanguageModel, RouteDecision},
};
use profile::PromptProfile;

pub use explain::RouteExplanation;

#[cfg(test)]
pub(crate) use crate::runtime::DEFAULT_FAST_OLLAMA_MODEL;
pub use catalog::PRIMARY_OLLAMA_MODEL;

/// Common interface for anything that can choose a backend for a prompt.
pub trait Router {
    /// Route one prompt to one backend/model.
    fn route(&self, prompt: &str) -> Result<RouteDecision>;

    /// Return every model the router knows about.
    fn models(&self) -> &[LanguageModel];
}

/// Chooses the best available Ollama model for a prompt.
///
/// Uses the primary model by default; falls back to the fast model for short
/// or simple prompts.
pub struct ModelRouter {
    /// Ordered list of models. First entry is the primary fallback.
    models: Vec<LanguageModel>,
}

impl ModelRouter {
    pub(super) fn fast_ollama_model_name(&self) -> String {
        self.models
            .get(1)
            .or_else(|| self.models.first())
            .map(|m| m.name.clone())
            .unwrap_or_else(|| PRIMARY_OLLAMA_MODEL.to_string())
    }

    pub(super) fn primary_ollama_model(&self) -> Result<LanguageModel> {
        self.models
            .first()
            .cloned()
            .ok_or_else(|| {
                crate::Error::invariant(format!(
                    "router is missing required primary Ollama model `{PRIMARY_OLLAMA_MODEL}`"
                ))
            })
    }
}

impl Router for ModelRouter {
    fn route(&self, prompt: &str) -> Result<RouteDecision> {
        self.route_with_rules(prompt)
    }

    fn models(&self) -> &[LanguageModel] {
        &self.models
    }
}

#[cfg(test)]
mod tests;
