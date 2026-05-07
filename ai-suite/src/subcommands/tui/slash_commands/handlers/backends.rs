use crate::llm::Provider;
use crate::subcommands::tui::app::App;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub fn backends_command(app: &mut App, command: &ParsedCommand) {
    let report = backends_report(app);
    app.append_local_message(command.raw(), report);
    app.ui.status = "Listed backend status.".to_string();
}

fn backends_report(app: &App) -> String {
    [
        Provider::Ollama,
        Provider::Anthropic,
        Provider::OpenAi,
        Provider::Xai,
    ]
    .iter()
    .map(|provider| {
        let models = app
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
