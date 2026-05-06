use anyhow::Result;
use clap::Parser;

use crate::runtime::Runtime;
use crate::subcommands::spec::SubcommandId;

#[derive(Debug, Parser)]
#[command(name = "ai-suite", version, about = "A modular AI command suite")]
struct Cli {
    #[command(subcommand)]
    command: Option<SubcommandId>,
}

pub async fn dispatch(runtime: Runtime) -> Result<()> {
    match Cli::parse().command.unwrap_or(SubcommandId::Tui) {
        SubcommandId::Tui => crate::subcommands::tui::run(&runtime).await,
        SubcommandId::Swarm(args) => crate::subcommands::swarm::run(&runtime, args).await,
    }
}
