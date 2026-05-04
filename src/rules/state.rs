mod edit;
mod load;
mod sections;

use std::path::{Path, PathBuf};

use super::storage::RulesFile;

/// Loaded rules and the paths where they live.
#[derive(Clone, Debug)]
pub struct RulesState {
    pub(super) enabled: bool,
    pub(super) global_path: PathBuf,
    pub(super) global_rules: Option<RulesFile>,
    pub(super) project_root: Option<PathBuf>,
    pub(super) project_rules: Option<RulesFile>,
    pub(super) load_warnings: Vec<String>,
}

impl RulesState {
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
}
