use std::future::Future;
use std::pin::Pin;

use clap::Subcommand;

use crate::Result;
use crate::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum SubcommandId {
    /// Launch the interactive terminal UI.
    Tui,

    /// Run the swarm orchestration tool.
    Swarm,

    /// Run the food planning tool.
    Food,
}

pub type SubcommandFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
pub type SubcommandRunner = for<'a> fn(&'a Runtime) -> SubcommandFuture<'a>;
