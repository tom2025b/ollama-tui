use crate::errors;
use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

/// `/debug` — toggle full-error display. With debug on, model failures show
/// the full error chain instead of the friendly summary.
pub fn debug_command(app: &mut App, command: &ParsedCommand) {
    let now_on = errors::toggle_debug_mode();
    let body = if now_on {
        "Debug mode ON. Errors will show the full technical chain.\n\
         Tip: set AI_SUITE_DEBUG=1 to make this the default."
    } else {
        "Debug mode OFF. Errors will show short, friendly summaries."
    };
    app.append_local_message(command.raw(), body.to_string());
    app.ui.status = if now_on {
        "Debug mode on.".to_string()
    } else {
        "Debug mode off.".to_string()
    };
}
