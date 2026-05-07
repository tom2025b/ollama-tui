use super::state::RulesState;

impl RulesState {
    /// Wrap a user prompt with the active rules, if any are enabled.
    pub fn prompt_with_rules(&self, prompt: &str) -> String {
        let sections = self.active_rule_sections();

        if !self.enabled || sections.is_empty() {
            return prompt.to_string();
        }

        let mut wrapped = String::from(
            "Use these persistent ai-suite rules for this answer. If the current user request explicitly conflicts with a style preference, follow the current request.\n\n",
        );

        for section in sections {
            wrapped.push_str(&format!(
                "[{} from {}]\n{}\n\n",
                section.label,
                section.path.display(),
                section.content
            ));
        }

        wrapped.push_str("Current user request:\n");
        wrapped.push_str(prompt);
        wrapped
    }

    /// Short status suitable for the TUI status panel.
    pub fn status_line(&self) -> String {
        if !self.enabled {
            return "off".to_string();
        }

        let active = self.active_rule_short_labels();

        if active.is_empty() {
            "on (none loaded)".to_string()
        } else {
            format!("on ({})", active.join(" + "))
        }
    }

    /// Summary appended to route explanations when rules are applied.
    pub fn application_summary(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let active = self.active_rule_short_labels();

        if active.is_empty() {
            None
        } else {
            Some(format!("Applied {} rules.", active.join(" and ")))
        }
    }

    fn active_rule_short_labels(&self) -> Vec<&'static str> {
        self.active_rule_sections()
            .into_iter()
            .map(|section| section.short_label)
            .collect()
    }
}
