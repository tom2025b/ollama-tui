//! Runtime path discovery for config, rules, history, and editor selection,
//! including non-fatal fallbacks when process state is incomplete.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::environment::RuntimeEnvironment;

const APP_DIR: &str = "ai-suite";
const GLOBAL_CONFIG_DIR: &str = ".config";
const HISTORY_BASE_DIR: &str = ".local/share";
const HISTORY_DIR: &str = "history";
const PROJECT_RULES_DIR: &str = ".ai-suite";
const RULES_FILE: &str = "rules.md";
const CONFIG_FILE: &str = "config.toml";

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "Gemfile",
    "Makefile",
];

/// Resolved runtime paths plus any non-fatal fallback warnings collected while
/// choosing defaults.
#[derive(Debug)]
pub(super) struct LoadedRuntimePaths {
    pub(super) paths: RuntimePaths,
    pub(super) warnings: Vec<String>,
}

/// Runtime-derived paths and command defaults shared across the application.
#[derive(Clone, Debug)]
pub(crate) struct RuntimePaths {
    home_dir: PathBuf,
    current_dir: PathBuf,
    project_root: Option<PathBuf>,
    global_rules_path: PathBuf,
    project_rules_path: PathBuf,
    history_dir: PathBuf,
    config_file_path: PathBuf,
    editor: OsString,
}

impl RuntimePaths {
    /// Resolve runtime paths from the current process environment.
    pub(super) fn from_environment<E>(environment: &E) -> LoadedRuntimePaths
    where
        E: RuntimeEnvironment + ?Sized,
    {
        let mut warnings = Vec::new();
        let current_dir_result = environment.current_dir();
        let current_dir_fallback = current_dir_result.as_ref().ok().cloned();
        let home_dir = match environment.var_os("HOME").filter(|value| !value.is_empty()) {
            Some(path) => PathBuf::from(path),
            None => {
                let fallback = current_dir_fallback
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("."));
                warnings.push(format!(
                    "HOME is not set. Using {} as the runtime home directory for config, rules, and history files.",
                    fallback.display()
                ));
                fallback.clone()
            }
        };
        let current_dir = match current_dir_result {
            Ok(path) => path,
            Err(error) => {
                warnings.push(format!(
                    "Could not resolve the current working directory: {error}. Falling back to {}.",
                    home_dir.display()
                ));
                home_dir.clone()
            }
        };
        let project_root = find_project_root(&current_dir);
        let editor = environment
            .var_os("VISUAL")
            .filter(|value| !value.is_empty())
            .or_else(|| {
                environment
                    .var_os("EDITOR")
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| OsString::from("vi"));

        LoadedRuntimePaths {
            paths: Self::from_resolved_parts(home_dir, current_dir, project_root, editor),
            warnings,
        }
    }

    /// Build test paths from already-resolved components.
    #[cfg(test)]
    pub(crate) fn from_parts(
        home_dir: PathBuf,
        current_dir: PathBuf,
        project_root: Option<PathBuf>,
    ) -> Self {
        Self::from_resolved_parts(home_dir, current_dir, project_root, OsString::from("vi"))
    }

    /// Project root inferred from common repository marker files.
    pub(crate) fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Global user rules file path.
    pub(crate) fn global_rules_path(&self) -> &Path {
        &self.global_rules_path
    }

    /// Project-local rules file path.
    pub(crate) fn project_rules_path(&self) -> &Path {
        &self.project_rules_path
    }

    /// History export path for a specific timestamp.
    pub(crate) fn history_report_path(&self, timestamp_seconds: u64) -> PathBuf {
        self.history_dir
            .join(format!("ai-suite-history-{timestamp_seconds}.txt"))
    }

    /// Preferred editor command from `VISUAL`, `EDITOR`, or `vi`.
    pub(crate) fn editor(&self) -> &OsStr {
        &self.editor
    }

    /// Config file path under the runtime home directory.
    pub(crate) fn config_file_path(&self) -> &Path {
        &self.config_file_path
    }

    /// Expand `~` and relative paths against the runtime home/current dir.
    pub(crate) fn expand_user_path(&self, path: &str) -> PathBuf {
        if path == "~" {
            return self.home_dir.clone();
        }

        if let Some(rest) = path.strip_prefix("~/") {
            return self.home_dir.join(rest);
        }

        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.current_dir.join(path)
        }
    }

    fn from_resolved_parts(
        home_dir: PathBuf,
        current_dir: PathBuf,
        project_root: Option<PathBuf>,
        editor: OsString,
    ) -> Self {
        let global_config_dir = home_dir.join(GLOBAL_CONFIG_DIR).join(APP_DIR);
        let global_rules_path = global_config_dir.join(RULES_FILE);
        let config_file_path = global_config_dir.join(CONFIG_FILE);
        let project_rules_base = project_root.as_ref().unwrap_or(&current_dir);
        let project_rules_path = project_rules_base.join(PROJECT_RULES_DIR).join(RULES_FILE);
        let history_dir = home_dir
            .join(HISTORY_BASE_DIR)
            .join(APP_DIR)
            .join(HISTORY_DIR);

        Self {
            home_dir,
            current_dir,
            project_root,
            global_rules_path,
            project_rules_path,
            history_dir,
            config_file_path,
            editor,
        }
    }
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        if PROJECT_MARKERS
            .iter()
            .any(|marker| current.join(marker).exists())
        {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Result};
    use std::ffi::OsString;

    #[derive(Debug)]
    enum CurrentDirOutcome {
        Ok(PathBuf),
        Err,
    }

    #[derive(Debug)]
    struct TestEnvironment {
        home: Option<OsString>,
        visual: Option<OsString>,
        editor: Option<OsString>,
        current_dir: CurrentDirOutcome,
    }

    impl RuntimeEnvironment for TestEnvironment {
        fn var(&self, key: &str) -> Option<String> {
            self.var_os(key).and_then(|value| value.into_string().ok())
        }

        fn var_os(&self, key: &str) -> Option<OsString> {
            match key {
                "HOME" => self.home.clone(),
                "VISUAL" => self.visual.clone(),
                "EDITOR" => self.editor.clone(),
                _ => None,
            }
        }

        fn current_dir(&self) -> Result<PathBuf> {
            match &self.current_dir {
                CurrentDirOutcome::Ok(path) => Ok(path.clone()),
                CurrentDirOutcome::Err => Err(Error::io_operation(
                    "resolve current working directory",
                    std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
                )),
            }
        }
    }

    #[test]
    fn expands_home_relative_paths() {
        let paths = RuntimePaths::from_parts(
            PathBuf::from("/home/tester"),
            PathBuf::from("/work/project"),
            Some(PathBuf::from("/work/project")),
        );

        assert_eq!(
            paths.expand_user_path("~/report.txt"),
            PathBuf::from("/home/tester/report.txt")
        );
    }

    #[test]
    fn expands_relative_paths_from_runtime_current_dir() {
        let paths = RuntimePaths::from_parts(
            PathBuf::from("/home/tester"),
            PathBuf::from("/work/project"),
            None,
        );

        assert_eq!(
            paths.expand_user_path("reports/session.txt"),
            PathBuf::from("/work/project/reports/session.txt")
        );
    }

    #[test]
    fn missing_home_warns_and_falls_back_to_current_dir() {
        let loaded = RuntimePaths::from_environment(&TestEnvironment {
            home: None,
            visual: None,
            editor: None,
            current_dir: CurrentDirOutcome::Ok(PathBuf::from("/work/project")),
        });

        assert_eq!(
            loaded.paths.config_file_path(),
            Path::new("/work/project/.config/ai-suite/config.toml")
        );
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("HOME is not set"));
    }

    #[test]
    fn current_dir_failure_warns_and_falls_back_to_home() {
        let loaded = RuntimePaths::from_environment(&TestEnvironment {
            home: Some(OsString::from("/home/tester")),
            visual: None,
            editor: None,
            current_dir: CurrentDirOutcome::Err,
        });

        assert_eq!(
            loaded.paths.expand_user_path("logs"),
            PathBuf::from("/home/tester/logs")
        );
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("current working directory"));
        assert!(loaded.warnings[0].contains("/home/tester"));
    }
}
