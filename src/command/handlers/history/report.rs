use std::fmt::Write as _;

use super::CommandContext;

pub(in crate::command::handlers) fn history_report(context: &dyn CommandContext) -> String {
    let conversation = context
        .history_entries()
        .into_iter()
        .filter(|message| message.include_in_context)
        .collect::<Vec<_>>();
    let mut report = String::new();

    write_report_header(context, &mut report);

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

fn write_report_header(context: &dyn CommandContext, report: &mut String) {
    let _ = writeln!(report, "ollama-me history");
    let _ = writeln!(report, "Rules: {}", context.rules_status_line());

    if let Some(project_root) = context.project_root() {
        let _ = writeln!(report, "Project: {}", project_root.display());
    }

    let _ = writeln!(report);
}
