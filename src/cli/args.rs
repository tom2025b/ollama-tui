use clap::Parser;

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
