use std::{
    env,
    path::{Path, PathBuf},
};

const PROJECT_RULES_DIR: &str = ".ollama-me";
const PROJECT_RULES_FILE: &str = "rules.md";

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "Gemfile",
    "Makefile",
];

pub(super) fn find_project_root(start: &Path) -> Option<PathBuf> {
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

pub(super) fn global_rules_path(home: &Path) -> PathBuf {
    home.join(".config").join("ollama-me").join("rules.md")
}

pub(super) fn project_rules_path(project_root: &Path) -> PathBuf {
    project_root
        .join(PROJECT_RULES_DIR)
        .join(PROJECT_RULES_FILE)
}

pub(super) fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
