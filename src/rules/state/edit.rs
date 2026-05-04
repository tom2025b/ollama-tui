use std::{env, fs, io, path::PathBuf};

use super::RulesState;
use crate::rules::paths::home_dir;
use crate::rules::storage::default_rules_template;
use crate::rules::target::RulesTarget;

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

    pub(in crate::rules) fn project_edit_path(&self) -> PathBuf {
        self.project_root
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| home_dir()))
            .join(".ollama-me")
            .join("rules.md")
    }
}
