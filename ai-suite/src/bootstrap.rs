use crate::Result;

/// Start ai-suite.
pub async fn run() -> Result<()> {
    crate::errors::init_debug_mode_from_env();
    let runtime = crate::runtime::Runtime::load();

    for warning in runtime.config_warnings() {
        eprintln!("{}", startup_warning_line(warning));
    }

    if let Err(error) = crate::cli::dispatch(crate::cli::Cli::parse_args(), runtime).await {
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
