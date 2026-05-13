//! Runtime configuration assembly from environment variables, the optional user
//! config file, and derived paths.

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
use paths::LoadedRuntimePaths;

/// Runtime state shared across command dispatch, routing, and UI layers.
#[derive(Clone, Debug)]
pub(crate) struct Runtime {
    config: RuntimeConfig,
    paths: RuntimePaths,
    config_warnings: Vec<String>,
    config_source_path: Option<std::path::PathBuf>,
}

impl Runtime {
    /// Load runtime state from the current process environment.
    pub(crate) fn load() -> Self {
        Self::from_environment(&ProcessEnvironment)
    }

    /// Resolved runtime config.
    pub(crate) fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Resolved runtime paths.
    pub(crate) fn paths(&self) -> &RuntimePaths {
        &self.paths
    }

    /// Non-fatal warnings produced while resolving runtime paths or loading
    /// `config.toml`. Empty when every input resolved cleanly.
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
        let resolved_paths = RuntimePaths::from_environment(environment);
        let loaded = LoadedFileConfig::read(resolved_paths.paths.config_file_path());
        let LoadedRuntimePaths { paths, warnings } = resolved_paths;
        let mut config_warnings = warnings;
        config_warnings.extend(loaded.warnings);

        Self {
            config: RuntimeConfig::from_environment_and_file(environment, &loaded.config),
            paths,
            config_warnings,
            config_source_path: loaded.source_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::config::DEFAULT_CONTEXT_TURNS;
    use super::*;
    use std::{ffi::OsString, path::PathBuf};

    #[derive(Debug)]
    struct TestEnvironment {
        home: Option<OsString>,
        current_dir: std::result::Result<PathBuf, crate::Error>,
    }

    impl RuntimeEnvironment for TestEnvironment {
        fn var(&self, key: &str) -> Option<String> {
            self.var_os(key).and_then(|value| value.into_string().ok())
        }

        fn var_os(&self, key: &str) -> Option<OsString> {
            match key {
                "HOME" => self.home.clone(),
                _ => None,
            }
        }

        fn current_dir(&self) -> crate::Result<PathBuf> {
            match &self.current_dir {
                Ok(path) => Ok(path.clone()),
                Err(error) => Err(crate::Error::io_operation(
                    "resolve current working directory",
                    std::io::Error::new(std::io::ErrorKind::PermissionDenied, error.to_string()),
                )),
            }
        }
    }

    #[test]
    fn runtime_load_surfaces_path_warnings_without_aborting() {
        let runtime = Runtime::from_environment(&TestEnvironment {
            home: None,
            current_dir: Ok(PathBuf::from("/work/project")),
        });

        assert_eq!(
            runtime.config().context().context_turns(),
            DEFAULT_CONTEXT_TURNS
        );
        assert_eq!(
            runtime.paths().config_file_path(),
            std::path::Path::new("/work/project/.config/ai-suite/config.toml")
        );
        assert_eq!(runtime.config_warnings().len(), 1);
        assert!(runtime.config_warnings()[0].contains("HOME is not set"));
    }
}
