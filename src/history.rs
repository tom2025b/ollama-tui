use std::{
    env, fs, io,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const HISTORY_DIR: &str = ".local/share/ollama-me/history";
const SEND_REPORT_RELATIVE_PATH: &str = "bin/send-report";

/// Save a formatted history report to a user-supplied or default text file.
pub fn save_report(report: &str, requested_path: Option<&str>) -> io::Result<PathBuf> {
    let path = requested_path
        .map(expand_user_path)
        .unwrap_or_else(default_history_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, report)?;
    Ok(path)
}

/// Email a formatted history report through Tom's `send-report` helper.
pub fn email_report(report: &str, subject: &str) -> io::Result<()> {
    let mut child = Command::new(send_report_command())
        .arg(subject)
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(report.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "send-report exited with status {status}"
        )))
    }
}

fn default_history_path() -> PathBuf {
    history_dir().join(format!(
        "ollama-me-history-{}.txt",
        unix_timestamp_seconds()
    ))
}

fn history_dir() -> PathBuf {
    home_dir().join(HISTORY_DIR)
}

fn send_report_command() -> PathBuf {
    let home_send_report = home_dir().join(SEND_REPORT_RELATIVE_PATH);

    if home_send_report.is_file() {
        home_send_report
    } else {
        PathBuf::from("send-report")
    }
}

fn expand_user_path(path: &str) -> PathBuf {
    if path == "~" {
        return home_dir();
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir().join(rest);
    }

    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap_or_else(|_| home_dir()).join(path)
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_user_path_supports_home_prefix() {
        let expanded = expand_user_path("~/ollama-me-test.txt");

        assert!(expanded.ends_with("ollama-me-test.txt"));
        assert!(expanded.is_absolute() || expanded.starts_with("."));
    }
}
