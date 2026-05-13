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
    let command = selected_command(cli.command);

    crate::subcommands::registry::run(command, &runtime).await
}

fn selected_command(command: Option<SubcommandId>) -> SubcommandId {
    command.unwrap_or_else(crate::subcommands::registry::default_command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_command_defaults_to_registry_default() {
        assert_eq!(selected_command(None), SubcommandId::Tui);
    }

    #[test]
    fn selected_command_keeps_explicit_choice() {
        assert_eq!(
            selected_command(Some(SubcommandId::Swarm)),
            SubcommandId::Swarm
        );
    }

    #[test]
    fn clap_parser_accepts_explicit_subcommand() {
        let cli = Cli::try_parse_from(["ai-suite", "food"]).expect("food subcommand should parse");
        assert_eq!(cli.command, Some(SubcommandId::Food));
    }

    #[test]
    fn clap_parser_allows_missing_subcommand() {
        let cli = Cli::try_parse_from(["ai-suite"]).expect("default cli parse should succeed");
        assert_eq!(cli.command, None);
    }
}
