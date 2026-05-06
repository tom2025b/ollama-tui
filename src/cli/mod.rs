use anyhow::Result;
use clap::Parser;

use crate::runtime::Runtime;
use crate::subcommands::spec::SubcommandId;

#[derive(Debug, Parser)]
#[command(name = "ai-suite", version, about = "A modular AI command suite")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubcommandId>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

pub async fn dispatch(cli: Cli, runtime: Runtime) -> Result<()> {
    match cli.command.unwrap_or(SubcommandId::Tui) {
        SubcommandId::Tui => crate::subcommands::tui::run(&runtime).await,
        SubcommandId::Swarm(args) => crate::subcommands::swarm::run(&runtime, args).await,
        SubcommandId::Food => crate::subcommands::food::run(&runtime).await,
    }
}
