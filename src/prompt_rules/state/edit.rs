use std::{fs, io, path::PathBuf};

use super::RulesState;
use crate::prompt_rules::storage::default_rules_template;
use crate::prompt_rules::target::RulesTarget;

impl RulesState {
    pub fn prepare_edit(&self, target: RulesTarget) -> io::Result<PathBuf> {
        let path = match target {
            RulesTarget::Global => self.global_path.clone(),
            RulesTarget::Project => self.project_edit_path(),
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            fs::write(&path, default_rules_template(target))?;
        }

        Ok(path)
    }

    pub(in crate::prompt_rules) fn project_edit_path(&self) -> PathBuf {
        self.project_rules_path.clone()
    }
}
