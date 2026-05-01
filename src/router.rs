mod catalog;
mod profile;

use crate::llm::{LanguageModel, Provider, RouteDecision};
use profile::PromptProfile;

pub use catalog::{DEFAULT_FAST_OLLAMA_MODEL, PRIMARY_OLLAMA_MODEL};

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
    /// Internal implementation for the current rule-based router.
    fn route_with_rules(&self, prompt: &str) -> RouteDecision {
        let profile = PromptProfile::from_prompt(prompt);

        if profile.needs_privacy {
            return self.choose_with_fallback(
                &[Provider::Ollama],
                "The prompt contains privacy instructions or sensitive data markers, so I kept it on Ollama.",
            );
        }

        if profile.needs_current_context {
            return self.choose_with_fallback(
                &[Provider::Xai, Provider::OpenAi, Provider::Anthropic, Provider::Ollama],
                "This asks for current or public-context reasoning, so I preferred Grok and then fell back by availability.",
            );
        }

        if profile.needs_deep_reasoning_or_code {
            return self.choose_with_fallback(
                &[Provider::Anthropic, Provider::OpenAi, Provider::Xai, Provider::Ollama],
                "This looks like coding, debugging, planning, or deep reasoning, so I preferred Claude and then fell back by availability.",
            );
        }

        if profile.is_simple {
            return self.choose_specific_model(
                &Provider::Ollama,
                &self.fast_ollama_model_name(),
                "This is short/simple, so I chose the fast local Ollama model.",
            );
        }

        if profile.is_creative_or_general_cloud {
            return self.choose_with_fallback(
                &[Provider::OpenAi, Provider::Anthropic, Provider::Xai, Provider::Ollama],
                "This is a general or creative prompt, so I preferred GPT-4o and then fell back by availability.",
            );
        }

        self.choose_with_fallback(
            &[Provider::OpenAi, Provider::Anthropic, Provider::Ollama],
            "No special rule matched, so I chose the best configured general-purpose model.",
        )
    }

    fn choose_with_fallback(&self, providers: &[Provider], reason: &str) -> RouteDecision {
        for provider in providers {
            if let Some(model) = self.first_enabled_provider(provider) {
                return RouteDecision {
                    model,
                    reason: reason.to_string(),
                };
            }
        }

        RouteDecision {
            model: self.primary_ollama_model(),
            reason: "No preferred cloud backend is configured, so I used the primary local Ollama model.".to_string(),
        }
    }

    fn choose_specific_model(
        &self,
        provider: &Provider,
        model_name: &str,
        reason: &str,
    ) -> RouteDecision {
        if let Some(model) = self
            .models
            .iter()
            .find(|model| model.enabled && &model.provider == provider && model.name == model_name)
            .cloned()
        {
            return RouteDecision {
                model,
                reason: reason.to_string(),
            };
        }

        RouteDecision {
            model: self.primary_ollama_model(),
            reason: "The preferred exact model is not enabled, so I used the primary local Ollama model.".to_string(),
        }
    }

    fn first_enabled_provider(&self, provider: &Provider) -> Option<LanguageModel> {
        self.models
            .iter()
            .find(|model| model.enabled && &model.provider == provider)
            .cloned()
    }

    fn primary_ollama_model(&self) -> LanguageModel {
        self.models
            .iter()
            .find(|model| model.provider == Provider::Ollama && model.name == PRIMARY_OLLAMA_MODEL)
            .expect("router always contains primary Ollama model")
            .clone()
    }

    fn fast_ollama_model_name(&self) -> String {
        self.models
            .iter()
            .find(|model| model.provider == Provider::Ollama && model.name != PRIMARY_OLLAMA_MODEL)
            .expect("router always contains fast Ollama model")
            .name
            .clone()
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
