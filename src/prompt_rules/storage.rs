use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::content::active_rule_content;
use super::target::RulesTarget;

#[derive(Clone, Debug)]
pub(super) struct RulesFile {
    pub(super) path: PathBuf,
    pub(super) raw_content: String,
}

pub(super) fn read_optional_rules(path: &Path, warnings: &mut Vec<String>) -> Option<RulesFile> {
    match fs::read_to_string(path) {
        Ok(raw_content) => Some(RulesFile {
            path: path.to_path_buf(),
            raw_content,
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            warnings.push(format!("failed to read {}: {error}", path.display()));
            None
        }
    }
}

pub(super) fn rules_file_state(rules_file: Option<&RulesFile>) -> &'static str {
    match rules_file {
        None => "not created",
        Some(file) if active_rule_content(&file.raw_content).is_empty() => "empty",
        Some(_) => "active",
    }
}

pub(super) fn default_rules_template(target: RulesTarget) -> String {
    format!(
        "# ai-suite {}\n# Lines beginning with # are ignored by ai-suite.\n# Add persistent instructions below. Leave blank to disable this file.\n",
        target.label()
    )
}
