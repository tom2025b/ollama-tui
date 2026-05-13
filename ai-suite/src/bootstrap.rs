use crate::Result;

/// Start ai-suite.
pub async fn run() -> Result<()> {
    crate::errors::init_debug_mode_from_env();
    let runtime = crate::runtime::Runtime::load();

    // Surface config-file problems once, on startup. Bad config never aborts
    // launch; the runtime falls back to defaults and we just warn the user.
    for warning in runtime.config_warnings() {
        eprintln!("{}", startup_warning_line(warning));
    }

    if let Err(error) = crate::cli::dispatch(crate::cli::Cli::parse_args(), runtime).await {
        // Print a friendly summary on stderr and exit non-zero. We bypass the
        // default `Termination` impl so users don't see a raw `Error: …` debug
        // dump; the full chain is still available via `AI_SUITE_DEBUG=1`.
        eprintln!("{}", fatal_error_line(&error));
        std::process::exit(1);
    }

    Ok(())
}

fn startup_warning_line(warning: &str) -> String {
    format!("ai-suite: {warning}")
}

fn fatal_error_line(error: &crate::Error) -> String {
    format!("Error: {}", crate::errors::friendly_error(error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_warning_line_prefixes_warning() {
        assert_eq!(
            startup_warning_line("config fell back to defaults"),
            "ai-suite: config fell back to defaults"
        );
    }

    #[test]
    fn fatal_error_line_uses_friendly_error_rendering() {
        let rendered = fatal_error_line(&crate::Error::missing_api_key("OpenAI", "OPENAI_API_KEY"));
        assert!(rendered.starts_with("Error: "), "got: {rendered}");
        assert!(rendered.contains("OPENAI_API_KEY"), "got: {rendered}");
    }
}
