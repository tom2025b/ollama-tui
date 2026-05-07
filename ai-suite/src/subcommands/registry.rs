use anyhow::{Context, Result, bail};

use crate::runtime::Runtime;

use super::spec::{SubcommandFuture, SubcommandId, SubcommandRunner};

#[derive(Clone, Copy)]
struct BuiltInSubcommand {
    command: SubcommandId,
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
        (self.runner)(runtime)
            .await
            .with_context(|| format!("{} subcommand failed", self.name))
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
        bail!("unregistered subcommand: {command:?}");
    };

    subcommand.run(runtime).await
}

fn find(command: SubcommandId) -> Option<&'static BuiltInSubcommand> {
    BUILT_IN_SUBCOMMANDS
        .iter()
        .find(|subcommand| subcommand.matches(command))
}

#[cfg(test)]
fn names() -> impl Iterator<Item = &'static str> {
    BUILT_IN_SUBCOMMANDS
        .iter()
        .map(|subcommand| subcommand.name)
}

#[cfg(test)]
fn contains(name: &str) -> bool {
    names().any(|registered_name| registered_name == name)
}

#[cfg(test)]
fn name_for(command: SubcommandId) -> Option<&'static str> {
    find(command).map(|subcommand| subcommand.name)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lists_expected_initial_subcommands() {
        let names = names().collect::<Vec<_>>();

        assert_eq!(names, ["tui", "swarm", "food"]);
    }

    #[test]
    fn registry_checks_subcommand_names() {
        assert!(contains("tui"));
        assert!(contains("swarm"));
        assert!(contains("food"));
        assert!(!contains("unknown"));
    }

    #[test]
    fn registry_resolves_cli_commands() {
        assert_eq!(default_command(), SubcommandId::Tui);
        assert_eq!(name_for(SubcommandId::Tui), Some("tui"));
        assert_eq!(name_for(SubcommandId::Swarm), Some("swarm"));
        assert_eq!(name_for(SubcommandId::Food), Some("food"));
    }
}
