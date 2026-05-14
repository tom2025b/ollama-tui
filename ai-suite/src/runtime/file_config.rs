//! Optional `~/.config/ai-suite/config.toml` loader.
//!
//! Every field is optional. A missing file is normal (returns defaults). A
//! malformed file is non-fatal: the loader records human-readable warnings and
//! falls back to defaults. Environment variables always take priority over
//! file values.

use std::{io::ErrorKind, path::Path};

use anyhow::{Context, anyhow};
use serde::Deserialize;

use crate::Result;

/// Top-level file config. All fields optional.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct FileConfig {
    #[serde(default)]
    pub(crate) models: ModelsSection,
    #[serde(default)]
    pub(crate) context: ContextSection,
}

/// `[models]` section — model name overrides.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct ModelsSection {
    pub(crate) ollama_fast_model: Option<String>,
}

/// `[context]` section — conversation memory bounds.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct ContextSection {
    /// How many past turns to send to the model on each request.
    pub(crate) context_turns: Option<usize>,
    /// How many past turns to keep in memory before dropping the oldest.
    pub(crate) stored_turns: Option<usize>,
}

/// Result of attempting to load the file config.
#[derive(Debug)]
pub(crate) struct LoadedFileConfig {
    pub(crate) config: FileConfig,
    pub(crate) source_path: Option<std::path::PathBuf>,
    pub(crate) warnings: Vec<String>,
}

impl LoadedFileConfig {
    /// Read the config file at `path` if it exists.
    pub(crate) fn read(path: &Path) -> Self {
        match read_config_file(path) {
            Ok(Some(config)) => Self {
                config,
                source_path: Some(path.to_path_buf()),
                warnings: Vec::new(),
            },
            Ok(None) => Self {
                config: FileConfig::default(),
                source_path: None,
                warnings: Vec::new(),
            },
            Err(error) => Self {
                config: FileConfig::default(),
                source_path: None,
                warnings: vec![warning_for_load_error(path, &error)],
            },
        }
    }
}

fn read_config_file(path: &Path) -> Result<Option<FileConfig>> {
    let raw = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(anyhow!(error))
                .with_context(|| format!("failed to read config file at {}", path.display()));
        }
    };

    parse_config_file(&raw).map(Some)
}

fn parse_config_file(raw: &str) -> Result<FileConfig> {
    toml::from_str(raw)
        .map_err(|e| anyhow!("failed to parse TOML for runtime config file: {e}"))
}

fn warning_for_load_error(path: &Path, error: &anyhow::Error) -> String {
    format!(
        "Could not load config file at {}: {error}. Using defaults.",
        path.display()
    )
}

/// Default contents written by `/config edit` when no file exists yet.
pub(crate) fn default_config_template() -> &'static str {
    r#"# ai-suite configuration
#
# Every field is optional. Environment variables always take priority over
# values here. Delete or comment out anything you don't want to override.

[models]
# Small, fast local Ollama model used for short prompts.
# ollama_fast_model = "llama3:latest"

[context]
# How many past user/assistant turns to send to the model on each request.
# context_turns = 6

# How many past turns to keep in memory before dropping the oldest.
# stored_turns = 200
"#
}
