use anyhow::Result;

use crate::runtime::Runtime;

use super::args::Cli;

pub async fn dispatch(cli: Cli, runtime: Runtime) -> Result<()> {
    let command = cli
        .command
        .unwrap_or_else(crate::subcommands::registry::default_command);

    crate::subcommands::registry::run(command, &runtime).await
}
