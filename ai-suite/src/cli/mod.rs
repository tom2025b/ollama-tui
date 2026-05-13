use clap::Parser;

use crate::Result;
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
    let command = cli
        .command
        .unwrap_or_else(crate::subcommands::registry::default_command);

    crate::subcommands::registry::run(command, &runtime).await
}
