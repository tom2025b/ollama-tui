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
    pub(super) fn from_environment<E>(environment: &E) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        let home_dir = environment
            .var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let current_dir = environment
            .current_dir()
            .unwrap_or_else(|| home_dir.clone());
        let project_root = find_project_root(&current_dir);
        let editor = environment
            .var_os("VISUAL")
            .or_else(|| environment.var_os("EDITOR"))
            .unwrap_or_else(|| OsString::from("vi"));

        Self::from_resolved_parts(home_dir, current_dir, project_root, editor)
    }

    #[cfg(test)]
    pub(crate) fn from_parts(
        home_dir: PathBuf,
        current_dir: PathBuf,
        project_root: Option<PathBuf>,
    ) -> Self {
        Self::from_resolved_parts(home_dir, current_dir, project_root, OsString::from("vi"))
    }


    pub(crate) fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    pub(crate) fn global_rules_path(&self) -> &Path {
        &self.global_rules_path
    }

    pub(crate) fn project_rules_path(&self) -> &Path {
        &self.project_rules_path
    }

    pub(crate) fn history_report_path(&self, timestamp_seconds: u64) -> PathBuf {
        self.history_dir
            .join(format!("ai-suite-history-{timestamp_seconds}.txt"))
    }

    pub(crate) fn editor(&self) -> &OsStr {
        &self.editor
    }

    pub(crate) fn config_file_path(&self) -> &Path {
        &self.config_file_path
    }

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
}
