use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::{
    content::{RuleSection, active_section},
    storage::{RulesFile, default_rules_template, read_optional_rules},
    target::RulesTarget,
};
use crate::runtime::RuntimePaths;

/// Loaded rules and the paths where they live.
#[derive(Clone, Debug)]
pub struct RulesState {
    pub(super) enabled: bool,
    pub(super) global_path: PathBuf,
    pub(super) global_rules: Option<RulesFile>,
    pub(super) project_root: Option<PathBuf>,
    pub(super) project_rules_path: PathBuf,
    pub(super) project_rules: Option<RulesFile>,
    pub(super) load_warnings: Vec<String>,
}

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

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

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
