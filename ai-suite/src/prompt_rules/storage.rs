use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};

use super::content::active_rule_content;
use super::target::RulesTarget;
use crate::Result;

/// Raw contents of one rules file on disk.
#[derive(Clone, Debug)]
pub(super) struct RulesFile {
    pub(super) path: PathBuf,
    pub(super) raw_content: String,
}

/// Read one optional rules file. A missing file is normal; other read failures
/// are returned so the caller can decide whether to surface them as warnings or
/// hard errors.
pub(super) fn read_optional_rules(path: &Path) -> Result<Option<RulesFile>> {
    match fs::read_to_string(path) {
        Ok(raw_content) => Ok(Some(RulesFile {
            path: path.to_path_buf(),
            raw_content,
        })),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(anyhow!(error)).with_context(|| {
            format!("I/O error while reading rules file at {}", path.display())
        }),
    }
}

/// User-facing warning emitted when a rules file could not be loaded.
pub(super) fn warning_for_load_error(path: &Path, error: &anyhow::Error) -> String {
    format!(
        "Could not read rules file at {}: {error}. Ignoring this rules file.",
        path.display()
    )
}

/// Human-readable state for `/rules show`.
pub(super) fn rules_file_state(rules_file: Option<&RulesFile>) -> &'static str {
    match rules_file {
        None => "not created",
        Some(file) if active_rule_content(&file.raw_content).is_empty() => "empty",
        Some(_) => "active",
    }
}

/// Starter template written when the user edits a rules file that does not yet
/// exist.
pub(super) fn default_rules_template(target: RulesTarget) -> String {
    format!(
        "# ai-suite {}\n# Lines beginning with # are ignored by ai-suite.\n# Add persistent instructions below. Leave blank to disable this file.\n",
        target.label()
    )
}
