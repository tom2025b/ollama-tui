use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::content::active_rule_content;
use super::state::RulesState;
use super::storage::RulesFile;
use super::target::RulesTarget;
use crate::Error;
use crate::runtime::RuntimePaths;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn temp_path(label: &str) -> PathBuf {
    let id = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("ai-suite-prompt-rules-{label}-{id}"))
}

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

#[test]
fn load_keeps_rule_read_failures_non_fatal() {
    let home_dir = temp_path("home");
    let current_dir = temp_path("cwd");

    std::fs::create_dir_all(home_dir.join(".config/ai-suite")).unwrap();
    std::fs::create_dir_all(&current_dir).unwrap();
    std::fs::create_dir(home_dir.join(".config/ai-suite/rules.md")).unwrap();

    let paths = RuntimePaths::from_parts(home_dir.clone(), current_dir.clone(), None);
    let rules = RulesState::load(&paths);

    assert!(rules.global_rules.is_none());
    assert_eq!(rules.load_warnings.len(), 1);
    assert!(rules.load_warnings[0].contains("Could not read rules file"));
    assert!(rules.load_warnings[0].contains("Ignoring this rules file"));

    let _ = std::fs::remove_dir_all(home_dir);
    let _ = std::fs::remove_dir_all(current_dir);
}

#[test]
fn prepare_edit_creates_parent_dirs_and_template() {
    let base = temp_path("edit");
    let rules_path = base.join("nested/.config/ai-suite/rules.md");

    let rules = RulesState {
        enabled: true,
        global_path: rules_path.clone(),
        global_rules: None,
        project_root: None,
        project_rules_path: base.join("project/.ai-suite/rules.md"),
        project_rules: None,
        load_warnings: Vec::new(),
    };

    let prepared = rules.prepare_edit(RulesTarget::Global).unwrap();
    let contents = std::fs::read_to_string(&prepared).unwrap();

    assert_eq!(prepared, rules_path);
    assert!(contents.contains("# ai-suite global rules"));

    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn prepare_edit_returns_typed_error_for_invalid_parent_path() {
    let base = temp_path("invalid-parent");
    std::fs::create_dir_all(&base).unwrap();

    let parent_file = base.join("not-a-directory");
    std::fs::write(&parent_file, "block directory creation").unwrap();

    let rules = RulesState {
        enabled: true,
        global_path: parent_file.join("rules.md"),
        global_rules: None,
        project_root: None,
        project_rules_path: base.join("project/.ai-suite/rules.md"),
        project_rules: None,
        load_warnings: Vec::new(),
    };

    let error = rules
        .prepare_edit(RulesTarget::Global)
        .expect_err("invalid parent path should fail");

    match error {
        Error::Io { operation, .. } => assert_eq!(operation, "create rules parent directory"),
        other => panic!("expected Io error, got {other:?}"),
    }

    let _ = std::fs::remove_dir_all(base);
}
