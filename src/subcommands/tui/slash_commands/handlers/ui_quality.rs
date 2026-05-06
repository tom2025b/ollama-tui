use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub fn theme_command(app: &mut App, command: &ParsedCommand) {
    let requested = command.args().first().map(String::as_str);
    if is_report_request(requested) {
        app.append_local_message(command.raw(), app.theme_report());
        app.ui.status = "Displayed theme.".to_string();
        return;
    }

    match app.set_theme(requested) {
        Ok(message) => {
            app.append_local_message(command.raw(), message.clone());
            app.ui.status = message;
        }
        Err(error) => {
            app.append_local_message(command.raw(), error);
            app.ui.status = "Unknown theme command.".to_string();
        }
    }
}

pub fn resize_command(app: &mut App, command: &ParsedCommand) {
    let requested = command.args().first().map(String::as_str);
    if is_report_request(requested) {
        app.append_local_message(command.raw(), app.layout_report());
        app.ui.status = "Displayed layout.".to_string();
        return;
    }

    match app.set_layout_mode(requested) {
        Ok(message) => {
            app.append_local_message(command.raw(), message.clone());
            app.ui.status = message;
        }
        Err(error) => {
            app.append_local_message(command.raw(), error);
            app.ui.status = "Unknown resize command.".to_string();
        }
    }
}

fn is_report_request(requested: Option<&str>) -> bool {
    requested.is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "show" | "status"))
}
