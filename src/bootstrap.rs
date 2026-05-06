use anyhow::Result;

/// Start ai-suite.
pub async fn run() -> Result<()> {
    let runtime = crate::runtime::Runtime::load();

    crate::cli::dispatch(crate::cli::Cli::parse_args(), runtime).await
}
