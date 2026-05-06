use super::RulesState;
use crate::prompt_rules::storage::read_optional_rules;
use crate::runtime::RuntimePaths;

impl RulesState {
    /// Load global rules and project rules from the nearest project root.
    pub fn load(paths: &RuntimePaths) -> Self {
        let global_path = paths.global_rules_path().to_path_buf();
        let project_root = paths.project_root().map(ToOwned::to_owned);
        let project_rules_path = paths.project_rules_path().to_path_buf();

        let mut load_warnings = Vec::new();
        let global_rules = read_optional_rules(&global_path, &mut load_warnings);
        let project_rules = if project_root.is_some() {
            read_optional_rules(&project_rules_path, &mut load_warnings)
        } else {
            None
        };

        Self {
            enabled: true,
            global_path,
            global_rules,
            project_root,
            project_rules_path,
            project_rules,
            load_warnings,
        }
    }
}
