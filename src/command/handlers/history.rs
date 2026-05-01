use std::fmt::Write as _;

use crate::command::parser::ParsedCommand;
use crate::history as history_io;

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

fn history_report(context: &dyn CommandContext) -> String {
    let conversation = context
        .history_entries()
        .into_iter()
        .filter(|message| message.include_in_context)
        .collect::<Vec<_>>();

    let mut report = String::new();
    let _ = writeln!(report, "ollama-me history");
    let _ = writeln!(report, "Rules: {}", context.rules_status_line());

    if let Some(project_root) = context.project_root() {
        let _ = writeln!(report, "Project: {}", project_root.display());
    }

    let _ = writeln!(report);

    if conversation.is_empty() {
        report.push_str("No model conversation history yet.\n");
        return report;
    }

    for (index, message) in conversation.iter().enumerate() {
        let _ = writeln!(report, "## Turn {}", index + 1);
        let _ = writeln!(report, "Model: {}", message.model_name);
        let _ = writeln!(report, "Route: {}", message.route_reason);

        if message.failed {
            let _ = writeln!(report, "Status: failed");
        } else if message.in_progress {
            let _ = writeln!(report, "Status: streaming");
        }

        let _ = writeln!(report);
        let _ = writeln!(report, "User:");
        let _ = writeln!(report, "{}", message.prompt);
        let _ = writeln!(report);
        let _ = writeln!(report, "Assistant:");
        let answer = if message.answer.trim().is_empty() {
            "(no answer yet)"
        } else {
            message.answer
        };
        let _ = writeln!(report, "{answer}");
        let _ = writeln!(report);
    }

    report
}
