use anyhow::anyhow;

use crate::runtime::Runtime;
use crate::Result;

use super::spec::{SubcommandFuture, SubcommandId, SubcommandRunner};

#[derive(Clone, Copy)]
struct BuiltInSubcommand {
    command: SubcommandId,
    #[allow(dead_code)]
    name: &'static str,
    runner: SubcommandRunner,
}

impl BuiltInSubcommand {
    const fn new(command: SubcommandId, name: &'static str, runner: SubcommandRunner) -> Self {
        Self {
            command,
            name,
            runner,
        }
    }

    fn matches(&self, command: SubcommandId) -> bool {
        self.command == command
    }

    async fn run(&self, runtime: &Runtime) -> Result<()> {
        (self.runner)(runtime).await
    }
}

const BUILT_IN_SUBCOMMANDS: &[BuiltInSubcommand] = &[
    BuiltInSubcommand::new(SubcommandId::Tui, "tui", run_tui),
    BuiltInSubcommand::new(SubcommandId::Swarm, "swarm", run_swarm),
    BuiltInSubcommand::new(SubcommandId::Food, "food", run_food),
];

pub fn default_command() -> SubcommandId {
    SubcommandId::Tui
}

pub async fn run(command: SubcommandId, runtime: &Runtime) -> Result<()> {
    let Some(subcommand) = find(command) else {
        return Err(anyhow!("unregistered subcommand: {command:?}"));
    };

    subcommand.run(runtime).await
}

fn find(command: SubcommandId) -> Option<&'static BuiltInSubcommand> {
    BUILT_IN_SUBCOMMANDS
        .iter()
        .find(|subcommand| subcommand.matches(command))
}

fn run_tui(runtime: &Runtime) -> SubcommandFuture<'_> {
    Box::pin(crate::subcommands::tui::run(runtime))
}

fn run_swarm(runtime: &Runtime) -> SubcommandFuture<'_> {
    Box::pin(crate::subcommands::swarm::run(runtime))
}

fn run_food(runtime: &Runtime) -> SubcommandFuture<'_> {
    Box::pin(crate::subcommands::food::run(runtime))
}
