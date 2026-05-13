//! History export helpers for `/history save` and `/summary export`.

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{Error, Result, runtime::RuntimePaths};

const HISTORY_WRAP_WIDTH: usize = 88;

/// Save a formatted history report to a user-supplied or default text file.
pub fn save_report(
    paths: &RuntimePaths,
    report: &str,
    requested_path: Option<&str>,
) -> Result<PathBuf> {
    let path = if let Some(p) = requested_path {
        paths.expand_user_path(p)
    } else {
        paths.history_report_path(unix_timestamp_seconds())
    };

    let wrapped = wrap_report(report, HISTORY_WRAP_WIDTH);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            Error::io("create history report parent directory", parent, source)
        })?;
    }

    fs::write(&path, wrapped).map_err(|source| Error::io("write history report", &path, source))?;

    Ok(path)
}

/// Wrap each line of `report` at `width` columns on word boundaries, leaving
/// blank lines and already-short lines untouched.
fn wrap_report(report: &str, width: usize) -> String {
    let mut out = String::with_capacity(report.len());

    if width == 0 {
        return report.to_string();
    }

    for line in report.lines() {
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }

        let mut current = String::new();

        for word in line.split_whitespace() {
            let extra_space = if current.is_empty() { 0 } else { 1 };

            if !current.is_empty() && current.len() + word.len() + extra_space > width {
                out.push_str(&current);
                out.push('\n');
                current.clear();
            }

            if !current.is_empty() {
                current.push(' ');
            }

            current.push_str(word);
        }

        if !current.is_empty() {
            out.push_str(&current);
            out.push('\n');
        }
    }

    out
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_path(label: &str) -> PathBuf {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ai-suite-storage-{label}-{}-{id}",
            std::process::id()
        ))
    }

    #[test]
    fn save_report_supports_home_prefix() {
        let paths = RuntimePaths::from_parts(
            PathBuf::from("/tmp"),
            PathBuf::from("/tmp"),
            Some(PathBuf::from("/tmp")),
        );

        let saved = match save_report(&paths, "hello", Some("~/ai-suite-test.txt")) {
            Ok(path) => path,
            Err(error) => panic!("history report should save: {error}"),
        };

        assert_eq!(saved, PathBuf::from("/tmp/ai-suite-test.txt"));
        let _ = fs::remove_file(saved);
    }

    #[test]
    fn save_report_returns_typed_error_for_invalid_parent_path() {
        let home = temp_path("invalid-parent-home");
        let blocked_parent = home.join("reports");
        if let Err(error) = fs::create_dir_all(&home) {
            panic!("failed to create temp home at {}: {error}", home.display());
        }
        if let Err(error) = fs::write(&blocked_parent, "not a directory") {
            panic!(
                "failed to create blocking file at {}: {error}",
                blocked_parent.display()
            );
        }

        let paths = RuntimePaths::from_parts(home.clone(), home.clone(), Some(home.clone()));
        let error = match save_report(&paths, "hello", Some("~/reports/session.txt")) {
            Ok(path) => panic!("save_report should have failed for {}.", path.display()),
            Err(error) => error,
        };

        match error {
            Error::Io {
                operation, path, ..
            } => {
                assert_eq!(operation, "create history report parent directory");
                assert_eq!(path, blocked_parent);
            }
            other => panic!("expected path-aware I/O error, got {other}"),
        }

        let _ = fs::remove_file(&blocked_parent);
        let _ = fs::remove_dir(&home);
    }
}
