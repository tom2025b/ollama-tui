use crate::Result;
use crate::llm::RouteDecision;

use super::{ModelRouter, PromptProfile};

/// Trace of a single routing decision: the chosen model + reason, the rule that
/// fired, and the prompt features that drove the decision.
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
             Provider : Ollama\n\
             Model    : {}\n\
             Rule     : {}\n\
             Active   : {}\n\
             Reason   : {}\n\
             \n\
             Prompt features:\n{}",
            self.decision.model.name,
            self.matched_rule,
            active_line,
            self.decision.reason,
            features_block,
        )
    }
}

impl ModelRouter {
    /// Run the router and return both the decision and the classification that
    /// produced it.
    pub fn explain(&self, prompt: &str) -> Result<RouteExplanation> {
        let profile = PromptProfile::from_prompt(prompt);

        let matched_rule = if profile.is_simple {
            "simple / short → fast local Ollama"
        } else {
            "default → primary local Ollama"
        };

        Ok(RouteExplanation {
            decision: self.route_with_rules(prompt)?,
            matched_rule,
            features: vec![("is_simple", profile.is_simple)],
        })
    }
}
