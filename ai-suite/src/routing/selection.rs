use super::{ModelRouter, PromptProfile};
use crate::{
    Result,
    llm::{Provider, RouteDecision},
};

struct RoutePlan {
    providers: &'static [Provider],
    reason: &'static str,
}

const PRIVACY_REASON: &str =
    "The prompt contains privacy instructions or sensitive data markers, so I kept it on Ollama.";
const CURRENT_CONTEXT_PLAN: RoutePlan = RoutePlan {
    providers: &[
        Provider::Xai,
        Provider::OpenAi,
        Provider::Anthropic,
        Provider::Ollama,
    ],
    reason: "This asks for current or public-context reasoning, so I preferred Grok and then fell back by availability.",
};
const DEEP_REASONING_PLAN: RoutePlan = RoutePlan {
    providers: &[
        Provider::Anthropic,
        Provider::OpenAi,
        Provider::Xai,
        Provider::Ollama,
    ],
    reason: "This looks like coding, debugging, planning, or deep reasoning, so I preferred Claude and then fell back by availability.",
};
const CREATIVE_OR_GENERAL_PLAN: RoutePlan = RoutePlan {
    providers: &[
        Provider::OpenAi,
        Provider::Anthropic,
        Provider::Xai,
        Provider::Ollama,
    ],
    reason: "This is a general or creative prompt, so I preferred GPT-4o and then fell back by availability.",
};
const DEFAULT_GENERAL_PLAN: RoutePlan = RoutePlan {
    providers: &[Provider::OpenAi, Provider::Anthropic, Provider::Ollama],
    reason: "No special rule matched, so I chose the best configured general-purpose model.",
};
const SIMPLE_REASON: &str = "This is short/simple, so I chose the fast local Ollama model.";
const NO_BACKEND_REASON: &str =
    "No preferred cloud backend is configured, so I used the primary local Ollama model.";
const EXACT_MODEL_UNAVAILABLE_REASON: &str =
    "The preferred exact model is not enabled, so I used the primary local Ollama model.";

impl ModelRouter {
    /// Internal implementation for the current rule-based router.
    pub(super) fn route_with_rules(&self, prompt: &str) -> Result<RouteDecision> {
        let profile = PromptProfile::from_prompt(prompt);

        if profile.needs_privacy {
            return Ok(RouteDecision {
                model: self.primary_ollama_model()?,
                reason: PRIVACY_REASON.to_string(),
            });
        }

        if profile.needs_current_context {
            return self.choose_from_plan(&CURRENT_CONTEXT_PLAN);
        }

        if profile.needs_deep_reasoning_or_code {
            return self.choose_from_plan(&DEEP_REASONING_PLAN);
        }

        if profile.is_simple {
            return self.choose_specific_model(
                &Provider::Ollama,
                &self.fast_ollama_model_name(),
                SIMPLE_REASON,
            );
        }

        if profile.is_creative_or_general_cloud {
            return self.choose_from_plan(&CREATIVE_OR_GENERAL_PLAN);
        }

        self.choose_from_plan(&DEFAULT_GENERAL_PLAN)
    }

    fn choose_from_plan(&self, plan: &RoutePlan) -> Result<RouteDecision> {
        for provider in plan.providers {
            if let Some(model) = self.first_enabled_provider(provider) {
                return Ok(RouteDecision {
                    model,
                    reason: plan.reason.to_string(),
                });
            }
        }

        Ok(RouteDecision {
            model: self.primary_ollama_model()?,
            reason: NO_BACKEND_REASON.to_string(),
        })
    }

    fn choose_specific_model(
        &self,
        provider: &Provider,
        model_name: &str,
        reason: &str,
    ) -> Result<RouteDecision> {
        if let Some(model) = self
            .models
            .iter()
            .find(|model| model.enabled && &model.provider == provider && model.name == model_name)
            .cloned()
        {
            return Ok(RouteDecision {
                model,
                reason: reason.to_string(),
            });
        }

        Ok(RouteDecision {
            model: self.primary_ollama_model()?,
            reason: EXACT_MODEL_UNAVAILABLE_REASON.to_string(),
        })
    }
}
