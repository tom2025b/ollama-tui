use crate::llm::Provider;

use super::session::CommandContext;
use crate::command::parser::ParsedCommand;

pub fn handle_backends_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    let report = backends_report(context);
    context.append_local_message(command.raw(), report);
    context.set_status("Listed backend status.".to_string());
}

fn backends_report(context: &dyn CommandContext) -> String {
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
