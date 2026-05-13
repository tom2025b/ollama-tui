use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;
use crate::providers::ollama::{OllamaClient, OllamaModel};
use tokio::runtime::Handle;
use tokio::task;

pub fn ollama_command(app: &mut App, command: &ParsedCommand) {
    match fetch_ollama_models() {
        Ok(models) if models.is_empty() => {
            app.append_local_message(command.raw(), "No models found. Run `ollama pull <model>`.".to_string());
        }
        Ok(models) => {
            let table = render_table(&models);
            app.append_local_message(command.raw(), table);
            app.ui.status = format!("Listed {} local Ollama models.", models.len());
        }
        Err(e) => {
            app.append_local_message(command.raw(), format!("Failed to reach Ollama: {}", e));
            app.ui.status = "Ollama unreachable.".to_string();
        }
    }
}

pub fn use_command(app: &mut App, command: &ParsedCommand) {
    let Some(name) = command.args().first() else {
        app.append_local_message(command.raw(), "Usage: /use <model-name>".to_string());
        return;
    };
    match app.pin_model_by_name(name) {
        Ok(label) => {
            app.append_local_message(command.raw(), format!("Pinned to {}", label));
            app.ui.status = format!("Pinned to {}", label);
        }
        Err(e) => {
            app.append_local_message(command.raw(), e);
            app.ui.status = "Could not pin model.".to_string();
        }
    }
}

fn fetch_ollama_models() -> Result<Vec<OllamaModel>, String> {
    let client = OllamaClient::from_environment().map_err(|e| format!("{}", e))?;
    task::block_in_place(|| Handle::current().block_on(client.list_models()))
        .map_err(|e| format!("{}", e))
}

fn render_table(models: &[OllamaModel]) -> String {
    let mut lines = vec!["NAME                          SIZE      MODIFIED".to_string()];
    for m in models {
        let size = format_size(m.size);
        let modified = format_modified(&m.modified_at);
        lines.push(format!("{:<30} {:<9} {}", m.name, size, modified));
    }
    lines.join("\n")
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1_000.0;
    const MB: f64 = 1_000_000.0;
    const GB: f64 = 1_000_000_000.0;
    let b = bytes as f64;
    if b >= GB { format!("{:.1}GB", b/GB) }
    else if b >= MB { format!("{:.1}MB", b/MB) }
    else if b >= KB { format!("{:.1}KB", b/KB) }
    else { format!("{}B", bytes) }
}

fn format_modified(raw: &str) -> String {
    if raw.len() >= 16 {
        format!("{} {}", &raw[0..10], &raw[11..16])
    } else if raw.len() >= 10 {
        raw[0..10].to_string()
    } else {
        raw.to_string()
    }
}
