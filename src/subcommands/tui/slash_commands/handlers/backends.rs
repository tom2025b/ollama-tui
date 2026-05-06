use crate::llm::Provider;

use super::session::{CommandContext, CommandOutput, ModelCatalog};
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub fn handle_backends_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + ModelCatalog + ?Sized,
{
    let report = backends_report(context);
    context.append_local_message(command.raw(), report);
    context.set_status("Listed backend status.".to_string());
}

pub fn execute_backends_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_backends_command(context, command);
}

fn backends_report<C>(context: &C) -> String
where
    C: ModelCatalog + ?Sized,
{
    [
        Provider::Ollama,
        Provider::Anthropic,
        Provider::OpenAi,
        Provider::Xai,
    ]
    .iter()
    .map(|provider| {
        let models = context
            .models()
            .iter()
            .filter(|model| model.provider == *provider)
            .collect::<Vec<_>>();
        let enabled_count = models.iter().filter(|model| model.enabled).count();
        let status = if enabled_count > 0 {
            "available"
        } else {
            "not configured"
        };
        let notes = models
            .iter()
            .filter_map(|model| model.disabled_reason.as_deref())
            .collect::<Vec<_>>();
        let note = if notes.is_empty() {
            "ready".to_string()
        } else {
            notes.join("; ")
        };

        format!(
            "{}: {} ({}/{}) - {}",
            provider.label(),
            status,
            enabled_count,
            models.len(),
            note
        )
    })
    .collect::<Vec<_>>()
    .join("\n")
}
