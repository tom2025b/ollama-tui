use std::env;

use super::RulesState;
use crate::rules::paths::{find_project_root, global_rules_path, home_dir, project_rules_path};
use crate::rules::storage::read_optional_rules;

impl RulesState {
    /// Load global rules and project rules from the nearest project root.
    pub fn load() -> Self {
        let home = home_dir();
        let global_path = global_rules_path(&home);
        let cwd = env::current_dir().unwrap_or_else(|_| home.clone());
        let project_root = find_project_root(&cwd);

        let mut load_warnings = Vec::new();
        let global_rules = read_optional_rules(&global_path, &mut load_warnings);
        let project_rules = project_root
            .as_ref()
            .map(|root| project_rules_path(root))
            .and_then(|path| read_optional_rules(&path, &mut load_warnings));

        Self {
            enabled: true,
            global_path,
            global_rules,
            project_root,
            project_rules,
            load_warnings,
        }
    }
}
