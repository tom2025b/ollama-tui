use std::fmt::Write as _;
use std::fs;

use crate::runtime::default_config_template;
use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::session::ExternalAction;

/// `/config`              — show all effective settings and where they came from.
/// `/config edit`         — open the config file in $EDITOR (creates it from a
///                          template if missing). Changes take effect next launch.
/// `/config path`         — print the config file location.
pub fn config_command(app: &mut App, command: &ParsedCommand) {
    let sub = command.args().first().map(|arg| arg.to_ascii_lowercase());

    match sub.as_deref() {
        None | Some("show") | Some("status") => show_config(app, command.raw()),
        Some("edit") => edit_config(app, command.raw()),
        Some("path") => show_path(app, command.raw()),
        _ => {
            app.append_local_message(command.raw(), "Usage: /config [show|edit|path]".to_string());
            app.ui.status = "Unknown /config subcommand.".to_string();
        }
    }
}

fn show_config(app: &mut App, raw: &str) {
    let report = build_config_report(app);
    app.append_local_message(raw, report);
    app.ui.status = "Showed effective configuration.".to_string();
}

fn show_path(app: &mut App, raw: &str) {
    let path = app.runtime.paths().config_file_path();
    let body = if path.exists() {
        format!("Config file: {} (exists)", path.display())
    } else {
        format!(
            "Config file: {} (not created yet — `/config edit` will create it)",
            path.display()
        )
    };
    app.append_local_message(raw, body);
    app.ui.status = "Showed config path.".to_string();
}

fn edit_config(app: &mut App, raw: &str) {
    if app.session.waiting_for_model {
        app.append_local_message(
            raw,
            "Wait for the current model response to finish before editing the config.".to_string(),
        );
        app.ui.status = "Cannot edit config while a model is answering.".to_string();
        return;
    }

    let path = app.runtime.paths().config_file_path().to_path_buf();

    if let Err(error) = ensure_config_file(&path) {
        app.append_local_message(
            raw,
            format!(
                "Could not prepare config file at {}: {error}",
                path.display()
            ),
        );
        app.ui.status = "Failed to prepare config file.".to_string();
        return;
    }

    app.commands
        .queue_external_action(ExternalAction::EditConfig { path: path.clone() });
    app.ui.status = format!("Opening editor for config at {}.", path.display());
}

/// Create parent directory and file with the default template if it doesn't
/// exist, so $EDITOR opens something sensible.
fn ensure_config_file(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        fs::write(path, default_config_template())?;
    }
    Ok(())
}

/// Called by the external-action runner once `$EDITOR` exits.
pub fn complete_config_edit(
    app: &mut App,
    path: std::path::PathBuf,
    editor_result: Result<(), String>,
) {
    match editor_result {
        Ok(()) => {
            app.append_local_message(
                "/config",
                format!(
                    "Saved {}. Changes take effect on next launch.",
                    path.display()
                ),
            );
            app.ui.status = "Config saved (restart to apply).".to_string();
        }
        Err(error) => {
            app.append_local_message(
                "/config",
                format!("Could not edit config at {}.\n{}", path.display(), error),
            );
            app.ui.status = "Failed to edit config.".to_string();
        }
    }
}

fn build_config_report(app: &App) -> String {
    let runtime = &app.runtime;
    let config = runtime.config();
    let models = config.models();
    let context = config.context();

    let rows: Vec<(String, String, String)> = vec![
        (
            "ollama_fast_model".into(),
            models.fast_ollama_model_setting().value().clone(),
            models.fast_ollama_model_setting().source().label(),
        ),
        (
            "anthropic_model".into(),
            models.anthropic().model_setting().value().clone(),
            models.anthropic().model_setting().source().label(),
        ),
        (
            "openai_model".into(),
            models.openai().model_setting().value().clone(),
            models.openai().model_setting().source().label(),
        ),
        (
            "xai_model".into(),
            models.xai().model_setting().value().clone(),
            models.xai().model_setting().source().label(),
        ),
        (
            "context_turns".into(),
            context.context_turns_setting().value().to_string(),
            context.context_turns_setting().source().label(),
        ),
        (
            "stored_turns".into(),
            context.stored_turns_setting().value().to_string(),
            context.stored_turns_setting().source().label(),
        ),
    ];

    // Compute column widths so output is aligned and readable.
    let key_width = rows.iter().map(|(key, _, _)| key.len()).max().unwrap_or(0);
    let value_width = rows
        .iter()
        .map(|(_, value, _)| value.len())
        .max()
        .unwrap_or(0);

    let mut report = String::new();
    let _ = writeln!(report, "Effective configuration");
    let _ = writeln!(report, "───────────────────────");
    let path = runtime.paths().config_file_path();
    match runtime.config_source_path() {
        Some(p) => {
            let _ = writeln!(report, "Config file : {} (loaded)", p.display());
        }
        None if path.exists() => {
            let _ = writeln!(
                report,
                "Config file : {} (present but failed to parse — using defaults)",
                path.display()
            );
        }
        None => {
            let _ = writeln!(
                report,
                "Config file : {} (not created — run `/config edit` to create it)",
                path.display()
            );
        }
    }
    let _ = writeln!(report);

    for (key, value, source) in rows {
        let _ = writeln!(
            report,
            "  {key:<key_width$}  {value:<value_width$}  ({source})",
        );
    }

    let _ = write!(
        report,
        "\nProviders ready: {}",
        ready_providers_summary(models)
    );
    report
}

fn ready_providers_summary(models: &crate::runtime::ModelRuntimeConfig) -> String {
    let mut entries = vec!["Ollama (always)".to_string()];
    for (label, cloud) in [
        ("Anthropic", models.anthropic()),
        ("OpenAI", models.openai()),
        ("xAI", models.xai()),
    ] {
        if cloud.configured() {
            entries.push(label.to_string());
        }
    }
    entries.join(", ")
}
