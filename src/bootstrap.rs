use anyhow::Result;

/// Start ai-suite.
pub async fn run() -> Result<()> {
    crate::subcommands::tui::run().await
}
