use crate::Result;

/// Start ai-suite.
pub async fn run() -> Result<()> {
    crate::errors::init_debug_mode_from_env();
    let runtime = crate::runtime::Runtime::load();

    // Surface config-file problems once, on startup. Bad config never aborts
    // launch; the runtime falls back to defaults and we just warn the user.
    for warning in runtime.config_warnings() {
        eprintln!("ai-suite: {warning}");
    }

    if let Err(error) = crate::cli::dispatch(crate::cli::Cli::parse_args(), runtime).await {
        // Print a friendly summary on stderr and exit non-zero. We bypass the
        // default `Termination` impl so users don't see a raw `Error: …` debug
        // dump; the full chain is still available via `AI_SUITE_DEBUG=1`.
        eprintln!("Error: {}", crate::errors::friendly_error(&error));
        std::process::exit(1);
    }

    Ok(())
}
