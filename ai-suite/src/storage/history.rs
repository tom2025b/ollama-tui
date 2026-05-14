//! History export helpers for `/history save` and `/summary export`.

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Context;

use crate::{Result, runtime::RuntimePaths};

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
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create history report directory at {}", parent.display())
        })?;
    }

    fs::write(&path, wrapped)
        .with_context(|| format!("failed to write history report at {}", path.display()))?;

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
