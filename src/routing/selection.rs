use super::{ModelRouter, PromptProfile};
use crate::llm::{Provider, RouteDecision};

impl ModelRouter {
    /// Internal implementation for the current rule-based router.
    pub(super) fn route_with_rules(&self, prompt: &str) -> RouteDecision {
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
                &[
                    Provider::OpenAi,
                    Provider::Anthropic,
                    Provider::Xai,
                    Provider::Ollama,
                ],
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
}
