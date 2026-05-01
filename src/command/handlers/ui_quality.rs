use crate::command::parser::ParsedCommand;

use super::session::{CommandContext, Setting, SettingEdit};

pub fn handle_theme_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_setting_command(
        context,
        command,
        Setting::Theme,
        "Displayed theme.",
        "Unknown theme command.",
    );
}

pub fn handle_resize_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_setting_command(
        context,
        command,
        Setting::Layout,
        "Displayed layout.",
        "Unknown resize command.",
    );
}

fn handle_setting_command(
    context: &mut dyn CommandContext,
    command: &ParsedCommand,
    setting: Setting,
    displayed_status: &str,
    error_status: &str,
) {
    let requested = command.args().first().map(String::as_str);

    if is_report_request(requested) {
        context.append_local_message(command.raw(), context.setting_report(setting));
        context.set_status(displayed_status.to_string());
        return;
    }

    let result = match setting {
        Setting::Theme => context.set_setting(SettingEdit::Theme(requested)),
        Setting::Layout => context.set_setting(SettingEdit::Layout(requested)),
        Setting::Voice => Err("Unknown setting.".to_string()),
    };

    match result {
        Ok(message) => {
            context.append_local_message(command.raw(), message.clone());
            context.set_status(message);
        }
        Err(error) => {
            context.append_local_message(command.raw(), error);
            context.set_status(error_status.to_string());
        }
    }
}

fn is_report_request(requested: Option<&str>) -> bool {
    requested.is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "show" | "status"))
}
