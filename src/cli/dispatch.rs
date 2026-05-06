use anyhow::Result;

use super::args::{Cli, CliCommand};

pub async fn dispatch(cli: Cli) -> Result<()> {
    let command = cli.command.unwrap_or(CliCommand::Tui);
    debug_assert!(crate::subcommands::registry::contains(command.name()));

    match command {
        CliCommand::Tui => crate::subcommands::tui::run().await,
        CliCommand::Swarm => pending_subcommand("swarm"),
        CliCommand::Food => pending_subcommand("food"),
    }
}

fn pending_subcommand(name: &str) -> Result<()> {
    println!("{name} is not implemented yet");
    Ok(())
}
