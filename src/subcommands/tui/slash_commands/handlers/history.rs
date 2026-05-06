mod report;

use crate::history as history_io;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub(super) use self::report::history_report;
use super::session::CommandContext;

pub fn handle_history_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
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

            match history_io::save_report(&report, requested_path) {
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
        Some("email") | Some("mail") => {
            let subject = args.collect::<Vec<_>>().join(" ");
            let subject = if subject.trim().is_empty() {
                "ollama-me history"
            } else {
                subject.trim()
            };
            let report = history_report(context);

            match history_io::email_report(&report, subject) {
                Ok(()) => {
                    context.append_local_message(
                        command.raw(),
                        format!("Emailed history with subject: {subject}"),
                    );
                    context.set_status("Emailed history.".to_string());
                }
                Err(error) => {
                    context.append_local_message(
                        command.raw(),
                        format!("Could not email history through send-report: {error}"),
                    );
                    context.set_status("Failed to email history.".to_string());
                }
            }
        }
        _ => {
            context.append_local_message(
                command.raw(),
                "Usage: /history [show|save [path]|email [subject]]".to_string(),
            );
            context.set_status("Unknown /history command.".to_string());
        }
    }
}
