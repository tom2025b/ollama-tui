use crate::Result;
use crate::llm::RouteDecision;

use super::{ModelRouter, PromptProfile};

/// Trace of a single routing decision: the chosen model + reason, the rule that
/// fired, and the prompt features that drove the decision. Used by `/route` to
/// show *why* the router picked a backend without actually calling it.
#[derive(Debug)]
pub struct RouteExplanation {
    pub decision: RouteDecision,
    pub matched_rule: &'static str,
    pub features: Vec<(&'static str, bool)>,
}

impl RouteExplanation {
    /// Render as a human-readable, multi-line block for the chat view.
    pub fn format(&self) -> String {
        let active: Vec<&str> = self
            .features
            .iter()
            .filter(|(_, on)| *on)
            .map(|(name, _)| *name)
            .collect();
        let active_line = if active.is_empty() {
            "(none)".to_string()
        } else {
            active.join(", ")
        };

        let features_block = self
            .features
            .iter()
            .map(|(name, on)| format!("    {} {}", if *on { "✓" } else { "·" }, name))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "Routing trace (no model called)\n\
             ────────────────────────────────\n\
             Provider : {}\n\
             Model    : {}\n\
             Rule     : {}\n\
             Active   : {}\n\
             Reason   : {}\n\
             \n\
             Prompt features:\n{}",
            self.decision.model.provider.label(),
            self.decision.model.name,
            self.matched_rule,
            active_line,
            self.decision.reason,
            features_block,
        )
    }
}

impl ModelRouter {
    /// Run the rule-based router and return both the decision and the
    /// classification that produced it.
    pub fn explain(&self, prompt: &str) -> Result<RouteExplanation> {
        let profile = PromptProfile::from_prompt(prompt);

        // Mirror the priority order in `selection.rs::route_with_rules`.
        let matched_rule = if profile.needs_privacy {
            "privacy → primary local Ollama"
        } else if profile.needs_current_context {
            "current context → xAI → OpenAI → Anthropic → Ollama"
        } else if profile.needs_deep_reasoning_or_code {
            "deep reasoning / code → Anthropic → OpenAI → xAI → Ollama"
        } else if profile.is_simple {
            "simple / short → fast local Ollama"
        } else if profile.is_creative_or_general_cloud {
            "creative / general cloud → OpenAI → Anthropic → xAI → Ollama"
        } else {
            "default → OpenAI → Anthropic → Ollama"
        };

        Ok(RouteExplanation {
            decision: self.route_with_rules(prompt)?,
            matched_rule,
            features: vec![
                ("needs_privacy", profile.needs_privacy),
                ("needs_current_context", profile.needs_current_context),
                (
                    "needs_deep_reasoning_or_code",
                    profile.needs_deep_reasoning_or_code,
                ),
                ("is_simple", profile.is_simple),
                (
                    "is_creative_or_general_cloud",
                    profile.is_creative_or_general_cloud,
                ),
            ],
        })
    }
}
