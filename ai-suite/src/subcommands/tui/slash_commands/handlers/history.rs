use std::fmt::Write as _;

use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

pub fn history_command(app: &mut App, command: &ParsedCommand) {
    let mut args = command.args().iter().map(String::as_str);
    let subcommand = args.next().map(|arg| arg.to_ascii_lowercase());

    match subcommand.as_deref() {
        None | Some("show") => {
            app.append_local_message(command.raw(), history_report(app));
            app.ui.status = "Displayed history.".to_string();
        }
        Some("save") => {
            let requested_path = args.next();
            let report = history_report(app);

            match crate::storage::history::save_report(app.runtime.paths(), &report, requested_path)
            {
                Ok(path) => {
                    app.append_local_message(
                        command.raw(),
                        format!("Saved history to {}.", path.display()),
                    );
                    app.ui.status = "Saved history.".to_string();
                }
                Err(error) => {
                    app.append_local_message(
                        command.raw(),
                        format!("Could not save history: {error}"),
                    );
                    app.ui.status = "Failed to save history.".to_string();
                }
            }
        }
        _ => {
            app.append_local_message(
                command.raw(),
                "Usage: /history [show|save [path]]".to_string(),
            );
            app.ui.status = "Unknown /history command.".to_string();
        }
    }
}

pub(super) fn history_report(app: &App) -> String {
    let conversation = app
        .session
        .history
        .iter()
        .filter(|message| message.include_in_context)
        .collect::<Vec<_>>();
    let mut report = String::new();

    write_report_header(app, &mut report);

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
            &message.answer
        };
        let _ = writeln!(report, "{answer}");
        let _ = writeln!(report);
    }

    report
}

fn write_report_header(app: &App, report: &mut String) {
    let _ = writeln!(report, "ai-suite history");
    let _ = writeln!(report, "Rules: {}", app.rules.status_line());

    if let Some(project_root) = app.rules.project_root() {
        let _ = writeln!(report, "Project: {}", project_root.display());
    }

    let _ = writeln!(report);
}
