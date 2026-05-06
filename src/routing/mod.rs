mod catalog;
mod profile;
mod selection;

use crate::llm::{LanguageModel, Provider, RouteDecision};
use profile::PromptProfile;

#[cfg(test)]
pub(crate) use crate::runtime::DEFAULT_FAST_OLLAMA_MODEL;
pub use catalog::PRIMARY_OLLAMA_MODEL;

/// Common interface for anything that can choose a backend for a prompt.
///
/// The rest of the app should call this trait instead of depending on the
/// current rule-based implementation directly.
///
/// Future dynamic router will implement this same trait.
pub trait Router {
    /// Route one prompt to one backend/model.
    fn route(&self, prompt: &str) -> RouteDecision;

    /// Return every model the router knows about.
    ///
    /// The TUI uses this for visibility into enabled and disabled backends.
    fn models(&self) -> &[LanguageModel];
}

/// Chooses the best available model for a prompt.
///
/// The router is intentionally readable. It classifies the prompt with simple
/// rules, then selects the best configured model for that class. If a cloud API
/// key is missing, the router falls back to a local Ollama model instead of
/// choosing a backend that cannot run.
pub struct ModelRouter {
    /// Ordered list of models.
    ///
    /// The first model is treated as the safest default.
    models: Vec<LanguageModel>,
}

impl ModelRouter {
    pub(super) fn first_enabled_provider(&self, provider: &Provider) -> Option<LanguageModel> {
        self.models
            .iter()
            .find(|model| model.enabled && &model.provider == provider)
            .cloned()
    }

    pub(super) fn primary_ollama_model(&self) -> LanguageModel {
        // SAFETY: `ModelRouter::new` (see `routing/catalog.rs`) unconditionally pushes
        // an Ollama model with `name == PRIMARY_OLLAMA_MODEL` as the first entry of
        // `self.models`, and no public API mutates that vector. The `expect` below
        // therefore relies on a construction-time invariant: if it ever fires, the
        // router was built without the primary Ollama entry, which is a programmer
        // error, not a runtime condition.
        self.models
            .iter()
            .find(|model| model.provider == Provider::Ollama && model.name == PRIMARY_OLLAMA_MODEL)
            .expect("router always contains primary Ollama model")
            .clone()
    }

    pub(super) fn fast_ollama_model_name(&self) -> String {
        self.models
            .iter()
            .filter(|model| model.provider == Provider::Ollama)
            .nth(1)
            .or_else(|| {
                self.models
                    .iter()
                    .find(|model| model.provider == Provider::Ollama)
            })
            .map(|model| model.name.clone())
            .unwrap_or_else(|| PRIMARY_OLLAMA_MODEL.to_string())
    }
}

impl Router for ModelRouter {
    /// Choose a model for the user's prompt.
    ///
    /// The rule order matters. More specific prompt classes are checked before
    /// broad general-purpose routing.
    fn route(&self, prompt: &str) -> RouteDecision {
        self.route_with_rules(prompt)
    }

    /// Return every model known to the router.
    fn models(&self) -> &[LanguageModel] {
        &self.models
    }
}

#[cfg(test)]
mod tests;
