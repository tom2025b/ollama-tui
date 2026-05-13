use std::fmt::Write as _;

use tokio::runtime::Handle;
use tokio::task::block_in_place;

use crate::providers::ollama::{OllamaClient, OllamaModel};
use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;
use crate::{Result, friendly_error};

pub fn ollama_command(app: &mut App, command: &ParsedCommand) {
    if !is_list_command(command) {
        app.append_local_message(
            command.raw(),
            "Usage: /ollama\n       /ollama list\n\n\
             List local Ollama models with their name, size, and modified time."
                .to_string(),
        );
        app.ui.status = "Usage: /ollama [list].".to_string();
        return;
    }

    match list_ollama_models() {
        Ok(models) => {
            let report = render_models_table(&models);
            app.append_local_message(command.raw(), report);
            app.ui.status = if models.is_empty() {
                "No local Ollama models found.".to_string()
            } else {
                format!("Listed {} Ollama models.", models.len())
            };
        }
        Err(error) => {
            app.append_local_message(
                command.raw(),
                format!("Could not list Ollama models: {}", friendly_error(&error)),
            );
            app.ui.status = "Could not list Ollama models.".to_string();
        }
    }
}

pub fn use_command(app: &mut App, command: &ParsedCommand) {
    let Some(name) = command.args().first() else {
        app.append_local_message(
            command.raw(),
            "Usage: /use <model-name>. Example: /use qwen2.5:7b".to_string(),
        );
        app.ui.status = "Missing model name.".to_string();
        return;
    };

    match app.pin_model_by_name(name) {
        Ok(label) => {
            app.append_local_message(
                command.raw(),
                format!("Pinned to {label}. New prompts will skip the router."),
            );
            app.ui.status = format!("Pinned to {label}.");
        }
        Err(error) => {
            app.append_local_message(command.raw(), error);
            app.ui.status = "Could not pin model.".to_string();
        }
    }
}

fn is_list_command(command: &ParsedCommand) -> bool {
    command.args().is_empty()
        || command
            .args()
            .first()
            .is_some_and(|arg| arg.eq_ignore_ascii_case("list"))
}

fn list_ollama_models() -> Result<Vec<OllamaModel>> {
    let client = OllamaClient::from_environment()?;
    let handle = Handle::current();

    block_in_place(move || handle.block_on(client.list_models()))
}

fn render_models_table(models: &[OllamaModel]) -> String {
    let rows = models
        .iter()
        .map(|model| ModelRow {
            name: model.name.clone(),
            size: format_size(model.size),
            modified: format_modified_at(&model.modified_at),
        })
        .collect::<Vec<_>>();

    let widths = column_widths(&rows);

    let mut report = String::new();
    let _ = writeln!(
        report,
        "{:<name_width$} | {:<size_width$} | {:<modified_width$}",
        "NAME",
        "SIZE",
        "MODIFIED",
        name_width = widths.name,
        size_width = widths.size,
        modified_width = widths.modified,
    );
    let _ = writeln!(
        report,
        "{}-+-{}-+-{}",
        "-".repeat(widths.name),
        "-".repeat(widths.size),
        "-".repeat(widths.modified),
    );

    if rows.is_empty() {
        let _ = writeln!(report, "No local Ollama models installed.");
        return report;
    }

    for row in rows {
        let _ = writeln!(
            report,
            "{:<name_width$} | {:<size_width$} | {:<modified_width$}",
            row.name,
            row.size,
            row.modified,
            name_width = widths.name,
            size_width = widths.size,
            modified_width = widths.modified,
        );
    }

    report
}

fn column_widths(rows: &[ModelRow]) -> ColumnWidths {
    let name = rows
        .iter()
        .map(|row| row.name.len())
        .chain(std::iter::once("NAME".len()))
        .max()
        .unwrap_or("NAME".len());
    let size = rows
        .iter()
        .map(|row| row.size.len())
        .chain(std::iter::once("SIZE".len()))
        .max()
        .unwrap_or("SIZE".len());
    let modified = rows
        .iter()
        .map(|row| row.modified.len())
        .chain(std::iter::once("MODIFIED".len()))
        .max()
        .unwrap_or("MODIFIED".len());

    ColumnWidths {
        name,
        size,
        modified,
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    let formatted = format!("{value:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed} {}", UNITS[unit])
}

fn format_modified_at(modified_at: &str) -> String {
    let modified_at = modified_at.trim();
    if modified_at.is_empty() {
        return "-".to_string();
    }

    if let Some((date, time)) = modified_at.split_once('T') {
        let time = time.strip_suffix('Z').unwrap_or(time);
        let time = time.split('.').next().unwrap_or(time);
        return format!("{date} {time}");
    }

    modified_at.to_string()
}

struct ModelRow {
    name: String,
    size: String,
    modified: String,
}

struct ColumnWidths {
    name: usize,
    size: usize,
    modified: usize,
}
