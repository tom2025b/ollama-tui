use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub fn clear_command(app: &mut App, _command: &ParsedCommand) {
    if app.session.waiting_for_model {
        app.ui.status = "Cannot clear while a model is answering.".to_string();
        return;
    }

    app.session.history.clear();
    app.session.active_model_name = None;
    app.ui.status = "Conversation cleared.".to_string();
}
