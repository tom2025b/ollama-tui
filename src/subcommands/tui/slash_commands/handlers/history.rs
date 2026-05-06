mod report;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub(super) use self::report::history_report;
use super::session::{CommandContext, CommandOutput, HistoryExport, HistoryView, RulesContext};

pub fn handle_history_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + HistoryView + HistoryExport + RulesContext + ?Sized,
{
    let mut args = command.args().iter().map(String::as_str);
    let subcommand = args.next().map(|arg| arg.to_ascii_lowercase());

    match subcommand.as_deref() {
        None | Some("show") => {
            context.append_local_message(command.raw(), history_report(context));
            context.set_status("Displayed history.".to_string());
        }
        Some("save") => {
            let requested_path = args.next();
            let report = history_report(context);

            match context.save_history_report(&report, requested_path) {
                Ok(path) => {
                    context.append_local_message(
                        command.raw(),
                        format!("Saved history to {}.", path.display()),
                    );
                    context.set_status("Saved history.".to_string());
                }
                Err(error) => {
                    context.append_local_message(
                        command.raw(),
                        format!("Could not save history: {error}"),
                    );
                    context.set_status("Failed to save history.".to_string());
                }
            }
        }
        _ => {
            context.append_local_message(
                command.raw(),
                "Usage: /history [show|save [path]]".to_string(),
            );
            context.set_status("Unknown /history command.".to_string());
        }
    }
}

pub fn execute_history_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_history_command(context, command);
}
