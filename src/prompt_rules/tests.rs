use std::path::PathBuf;

use super::content::active_rule_content;
use super::state::RulesState;
use super::storage::RulesFile;

#[test]
fn active_rule_content_ignores_template_comments() {
    let raw = "# heading\n\n- Prefer short answers.\n<!-- hidden -->\nUse local context.";

    assert_eq!(
        active_rule_content(raw),
        "- Prefer short answers.\nUse local context."
    );
}

#[test]
fn prompt_is_unchanged_when_rules_are_disabled() {
    let mut rules = RulesState {
        enabled: false,
        global_path: PathBuf::from("/tmp/rules.md"),
        global_rules: Some(RulesFile {
            path: PathBuf::from("/tmp/rules.md"),
            raw_content: "Prefer short answers.".to_string(),
        }),
        project_root: None,
        project_rules_path: PathBuf::from("/tmp/project-rules.md"),
        project_rules: None,
        load_warnings: Vec::new(),
    };

    assert_eq!(rules.prompt_with_rules("hello"), "hello");
    rules.set_enabled(true);
    assert!(
        rules
            .prompt_with_rules("hello")
            .contains("Prefer short answers.")
    );
}
