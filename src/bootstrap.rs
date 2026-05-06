use anyhow::Result;

/// Start ai-suite.
pub async fn run() -> Result<()> {
    crate::cli::dispatch(crate::cli::Cli::parse_args()).await
}
