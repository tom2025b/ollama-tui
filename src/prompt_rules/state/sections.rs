use super::RulesState;
use crate::prompt_rules::content::{RuleSection, active_section};

impl RulesState {
    pub(in crate::prompt_rules) fn active_rule_sections(&self) -> Vec<RuleSection> {
        if !self.enabled {
            return Vec::new();
        }

        self.loaded_rule_sections()
    }

    pub(in crate::prompt_rules) fn loaded_rule_sections(&self) -> Vec<RuleSection> {
        let mut sections = Vec::new();

        if let Some(global_rules) = active_section("Global rules", "global", &self.global_rules) {
            sections.push(global_rules);
        }
        if let Some(project_rules) = active_section("Project rules", "project", &self.project_rules)
        {
            sections.push(project_rules);
        }

        sections
    }
}
