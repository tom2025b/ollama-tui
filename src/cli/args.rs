use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ai-suite", version, about = "A modular AI command suite")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum CliCommand {
    /// Launch the interactive terminal UI.
    Tui,

    /// Run the swarm orchestration tool.
    Swarm,

    /// Run the food planning tool.
    Food,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
