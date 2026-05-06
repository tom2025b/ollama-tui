use super::paths::project_rules_path;
use super::state::RulesState;
use super::storage::rules_file_state;

impl RulesState {
    /// Full local report for `/rules show`.
    pub fn report(&self) -> String {
        let mut report = String::new();
        self.write_header(&mut report);
        self.write_warnings(&mut report);
        self.write_loaded_text(&mut report);
        report.push_str(
            "\nCommands: /rules global, /rules project, /rules show, /rules off, /rules on, /rules toggle\n",
        );
        report
    }

    fn write_header(&self, report: &mut String) {
        report.push_str(&format!(
            "Rules are {}.\n\n",
            if self.enabled { "on" } else { "off" }
        ));
        report.push_str(&format!(
            "Global rules: {} ({})\n",
            self.global_path.display(),
            rules_file_state(self.global_rules.as_ref())
        ));

        if let Some(root) = &self.project_root {
            report.push_str(&format!("Project root: {}\n", root.display()));
            report.push_str(&format!(
                "Project rules: {} ({})\n",
                project_rules_path(root).display(),
                rules_file_state(self.project_rules.as_ref())
            ));
        } else {
            report.push_str("Project root: none detected from the current directory\n");
            report.push_str(&format!(
                "Project rules edit path: {}\n",
                self.project_edit_path().display()
            ));
        }
    }

    fn write_warnings(&self, report: &mut String) {
        if self.load_warnings.is_empty() {
            return;
        }

        report.push_str("\nLoad warnings:\n");
        for warning in &self.load_warnings {
            report.push_str("- ");
            report.push_str(warning);
            report.push('\n');
        }
    }

    fn write_loaded_text(&self, report: &mut String) {
        let sections = self.loaded_rule_sections();
        if sections.is_empty() {
            report.push_str("\nNo rule text is loaded. Lines beginning with # are ignored.\n");
            return;
        }

        report.push_str("\nLoaded rule text:\n");
        for section in sections {
            report.push_str(&format!("\n[{}]\n{}\n", section.label, section.content));
        }
    }
}
