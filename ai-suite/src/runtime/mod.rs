mod config;
mod environment;
mod file_config;
mod paths;

#[cfg(test)]
pub(crate) use config::DEFAULT_FAST_OLLAMA_MODEL;
pub(crate) use config::{CloudProviderRuntimeConfig, ModelRuntimeConfig, RuntimeConfig};
pub(crate) use file_config::default_config_template;
pub(crate) use paths::RuntimePaths;

use environment::{ProcessEnvironment, RuntimeEnvironment};
use file_config::LoadedFileConfig;

#[derive(Clone, Debug)]
pub(crate) struct Runtime {
    config: RuntimeConfig,
    paths: RuntimePaths,
    config_warnings: Vec<String>,
    config_source_path: Option<std::path::PathBuf>,
}

impl Runtime {
    pub(crate) fn load() -> Self {
        Self::from_environment(&ProcessEnvironment)
    }

    pub(crate) fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub(crate) fn paths(&self) -> &RuntimePaths {
        &self.paths
    }

    /// Warnings produced while loading `config.toml` (e.g. malformed file).
    /// Empty when the file is missing, empty, or parsed cleanly.
    pub(crate) fn config_warnings(&self) -> &[String] {
        &self.config_warnings
    }

    /// `Some(path)` when a config file was successfully loaded; `None` when no
    /// file exists or parsing failed.
    pub(crate) fn config_source_path(&self) -> Option<&std::path::Path> {
        self.config_source_path.as_deref()
    }

    fn from_environment<E>(environment: &E) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        let paths = RuntimePaths::from_environment(environment);
        let loaded = LoadedFileConfig::read(paths.config_file_path());
        Self {
            config: RuntimeConfig::from_environment_and_file(environment, &loaded.config),
            paths,
            config_warnings: loaded.warnings,
            config_source_path: loaded.source_path,
        }
    }
}
