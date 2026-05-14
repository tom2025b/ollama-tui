//! Optional `~/.config/ai-suite/config.toml` loader.
//!
//! Every field is optional. A missing file is normal (returns defaults). A
//! malformed file is non-fatal: the loader records human-readable warnings and
//! falls back to defaults. Environment variables always take priority over
//! file values.

use std::{io::ErrorKind, path::Path};

use serde::Deserialize;

use crate::{Error, Result};

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
        Err(error) => return Err(Error::io("read config file", path, error)),
    };

    parse_config_file(&raw).map(Some)
}

fn parse_config_file(raw: &str) -> Result<FileConfig> {
    toml::from_str(raw).map_err(|source| Error::toml_deserialize("runtime config file", source))
}

fn warning_for_load_error(path: &Path, error: &Error) -> String {
    match error {
        Error::TomlDeserialize { .. } => format!(
            "Config file at {} is malformed: {error}. Using defaults.",
            path.display()
        ),
        Error::Io { source, .. } => format!(
            "Could not read config file at {}: {source}. Using defaults.",
            path.display()
        ),
        _ => format!(
            "Could not load config file at {}: {error}. Using defaults.",
            path.display()
        ),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn write_temp(contents: &str) -> std::path::PathBuf {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ai-suite-config-test-{}-{id}.toml",
            std::process::id()
        ));
        if let Err(error) = std::fs::write(&path, contents) {
            panic!(
                "failed to write temp config fixture at {}: {error}",
                path.display()
            );
        }
        path
    }

    #[test]
    fn missing_file_yields_defaults_no_warnings() {
        let result = LoadedFileConfig::read(Path::new("/definitely/not/here/config.toml"));
        assert!(result.warnings.is_empty());
        assert!(result.source_path.is_none());
        assert!(result.config.models.ollama_fast_model.is_none());
    }

    #[test]
    fn malformed_file_yields_defaults_with_warning() {
        let path = write_temp("this is = = not = toml");
        let result = LoadedFileConfig::read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("malformed"));
        assert!(result.config.models.ollama_fast_model.is_none());
    }

    #[test]
    fn parses_partial_config() {
        let path = write_temp(
            r#"
[models]
ollama_fast_model = "llama3:7b"

[context]
context_turns = 12
"#,
        );
        let result = LoadedFileConfig::read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.warnings.is_empty());
        assert_eq!(
            result.config.models.ollama_fast_model.as_deref(),
            Some("llama3:7b")
        );
        assert_eq!(result.config.context.context_turns, Some(12));
        assert_eq!(result.config.context.stored_turns, None);
    }

    #[test]
    fn unreadable_path_yields_defaults_with_warning() {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ai-suite-config-test-dir-{}-{id}",
            std::process::id()
        ));
        if let Err(error) = std::fs::create_dir(&path) {
            panic!(
                "failed to create temp config fixture dir at {}: {error}",
                path.display()
            );
        }

        let result = LoadedFileConfig::read(&path);

        let _ = std::fs::remove_dir(&path);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("Could not read config file"));
        assert!(result.config.models.ollama_fast_model.is_none());
    }
}
