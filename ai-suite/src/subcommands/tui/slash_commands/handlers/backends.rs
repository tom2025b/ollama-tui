use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub fn backends_command(app: &mut App, command: &ParsedCommand) {
    let report = backends_report(app);
    app.append_local_message(command.raw(), report);
    app.ui.status = "Listed backend status.".to_string();
}

fn backends_report(app: &App) -> String {
    let models = app.models();
    let count = models.len();
    let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();

    format!(
        "Ollama: available ({count} model{s})\n  {list}",
        s = if count == 1 { "" } else { "s" },
        list = names.join(", ")
    )
}
