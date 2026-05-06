use clap::{Args, Subcommand};

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum SubcommandId {
    /// Launch the interactive terminal UI.
    Tui,

    /// Run a task on the local AI stack, or show readiness if no task is given.
    Swarm(SwarmArgs),

    /// Run the food planning tool.
    Food,
}

/// Arguments for the `swarm` subcommand.
#[derive(Debug, Clone, PartialEq, Eq, Args, Default)]
pub struct SwarmArgs {
    /// Task to route and run on the local Ollama stack. Omit for a readiness report.
    #[arg(value_name = "TASK")]
    pub task: Option<String>,
}
