//! Optional `~/.config/ai-suite/config.toml` loader.
//!
//! Every field is optional. A missing file is normal (returns defaults). A
//! malformed file is non-fatal: the loader records human-readable warnings and
//! falls back to defaults. Environment variables always take priority over
//! file values.

use std::path::Path;

use serde::Deserialize;

/// Top-level file config. All fields optional.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct FileConfig {
    #[serde(default)]
    pub(crate) models: ModelsSection,
    #[serde(default)]
    pub(crate) context: ContextSection,
}

/// `[models]` section — default model name overrides per provider.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct ModelsSection {
    pub(crate) ollama_fast_model: Option<String>,
    pub(crate) anthropic_model: Option<String>,
    pub(crate) openai_model: Option<String>,
    pub(crate) xai_model: Option<String>,
}

/// `[context]` section — conversation memory bounds.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct ContextSection {
    /// How many past turns to send to the model on each request.
    pub(crate) context_turns: Option<usize>,
    /// How many past turns to keep in memory before dropping the oldest.
    pub(crate) stored_turns: Option<usize>,
}

/// Result of attempting to load the file config: always returns *some* config
/// (defaults on error), plus warnings for the user.
pub(crate) struct LoadedFileConfig {
    pub(crate) config: FileConfig,
    pub(crate) source_path: Option<std::path::PathBuf>,
    pub(crate) warnings: Vec<String>,
}

impl LoadedFileConfig {
    /// Read the config file at `path` if it exists. A missing file is silent
    /// (returns defaults, no warnings). I/O errors and parse errors become
    /// warnings; the returned config is the default in those cases.
    pub(crate) fn read(path: &Path) -> Self {
        if !path.exists() {
            return Self {
                config: FileConfig::default(),
                source_path: None,
                warnings: Vec::new(),
            };
        }

        let raw = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                return Self {
                    config: FileConfig::default(),
                    source_path: None,
                    warnings: vec![format!(
                        "Could not read config file at {}: {error}. Using defaults.",
                        path.display()
                    )],
                };
            }
        };

        match toml::from_str::<FileConfig>(&raw) {
            Ok(config) => Self {
                config,
                source_path: Some(path.to_path_buf()),
                warnings: Vec::new(),
            },
            Err(error) => Self {
                config: FileConfig::default(),
                source_path: None,
                warnings: vec![format!(
                    "Config file at {} is malformed: {error}. Using defaults.",
                    path.display()
                )],
            },
        }
    }
}

/// Default contents written by `/config edit` when no file exists yet.
pub(crate) fn default_config_template() -> &'static str {
    r#"# ai-suite configuration
#
# Every field is optional. Environment variables (e.g. ANTHROPIC_API_KEY,
# ANTHROPIC_MODEL, OPENAI_MODEL) always take priority over values here.
# Delete or comment out anything you don't want to override.

[models]
# Small, fast local Ollama model used for short prompts.
# ollama_fast_model = "llama3:latest"

# Default cloud model names (used when the matching API key is set).
# anthropic_model = "claude-sonnet-4-5"
# openai_model    = "gpt-4o-mini"
# xai_model       = "grok-2-latest"

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
        std::fs::write(&path, contents).expect("write temp");
        path
    }

    #[test]
    fn missing_file_yields_defaults_no_warnings() {
        let result = LoadedFileConfig::read(Path::new("/definitely/not/here/config.toml"));
        assert!(result.warnings.is_empty());
        assert!(result.source_path.is_none());
        assert!(result.config.models.openai_model.is_none());
    }

    #[test]
    fn malformed_file_yields_defaults_with_warning() {
        let path = write_temp("this is = = not = toml");
        let result = LoadedFileConfig::read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("malformed"));
        assert!(result.config.models.openai_model.is_none());
    }

    #[test]
    fn parses_partial_config() {
        let path = write_temp(
            r#"
[models]
openai_model = "gpt-4o"

[context]
context_turns = 12
"#,
        );
        let result = LoadedFileConfig::read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.warnings.is_empty());
        assert_eq!(result.config.models.openai_model.as_deref(), Some("gpt-4o"));
        assert_eq!(result.config.context.context_turns, Some(12));
        assert_eq!(result.config.context.stored_turns, None);
    }
}
