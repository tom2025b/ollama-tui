use std::{
    fs::{self, File},
    io,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::runtime::RuntimePaths;

const HISTORY_WRAP_WIDTH: usize = 88;

/// Save a formatted history report to a user-supplied or default text file.
pub fn save_report(
    paths: &RuntimePaths,
    report: &str,
    requested_path: Option<&str>,
) -> io::Result<PathBuf> {
    let path = if let Some(p) = requested_path {
        paths.expand_user_path(p)
    } else {
        paths.history_report_path(unix_timestamp_seconds())
    };

    let wrapped = wrap_report(report, HISTORY_WRAP_WIDTH);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&path)?;
    file.write_all(wrapped.as_bytes())?;

    println!("Exported history to {}", path.display());
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

    #[test]
    fn save_report_supports_home_prefix() {
        let paths = RuntimePaths::from_parts(
            PathBuf::from("/tmp"),
            PathBuf::from("/tmp"),
            Some(PathBuf::from("/tmp")),
        );

        let saved = save_report(&paths, "hello", Some("~/ollama-me-test.txt"))
            .expect("history report should save");

        assert_eq!(saved, PathBuf::from("/tmp/ollama-me-test.txt"));
        let _ = fs::remove_file(saved);
    }
}
