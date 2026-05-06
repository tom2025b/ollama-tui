use std::io::{self, Write as _};

use anyhow::Result;

use crate::llm::Provider;
use crate::providers::ollama;
use crate::routing::{ModelRouter, PRIMARY_OLLAMA_MODEL, Router};
use crate::runtime::Runtime;
use crate::subcommands::spec::SwarmArgs;

pub async fn run(runtime: &Runtime, args: SwarmArgs) -> Result<()> {
    match args.task {
        Some(task) => run_task(runtime, &task).await,
        None => run_readiness_report(runtime),
    }
}

async fn run_task(runtime: &Runtime, task: &str) -> Result<()> {
    let router = ModelRouter::new(runtime.config().models());
    let route = router.route(task);

    let model_name = match route.model.provider {
        // Terminal-app routes cannot be used non-interactively from swarm.
        // Fall back to the primary local Ollama model instead.
        Provider::ClaudeCode | Provider::Codex => {
            eprintln!(
                "note: router selected {} (terminal app), falling back to local Ollama for swarm",
                route.model.display_label()
            );
            PRIMARY_OLLAMA_MODEL.to_string()
        }
        Provider::Ollama => route.model.name.clone(),
    };

    eprintln!("→ {model_name} — {}", route.reason);
    eprintln!();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    ollama::stream(&model_name, &[], task, |token| {
        let _ = handle.write_all(token.as_bytes());
        let _ = handle.flush();
    })
    .await?;

    writeln!(handle)?;
    Ok(())
}

fn run_readiness_report(runtime: &Runtime) -> Result<()> {
    let router = ModelRouter::new(runtime.config().models());
    let tools = crate::subcommands::capabilities::public_tool_registry()?;
    let tool_definitions = crate::subcommands::capabilities::sorted_tool_definitions(&tools);

    println!("Swarm readiness");
    println!("Models:");
    for model in router.models() {
        let state = if model.enabled { "ready" } else { "offline" };
        let detail = model
            .disabled_reason
            .as_deref()
            .unwrap_or_else(|| model.strengths.first().map_or("available", String::as_str));
        println!(
            "- {} {} [{}] - {}",
            model.provider.label(),
            model.name,
            state,
            detail
        );
    }

    println!("Tools:");
    for definition in tool_definitions {
        println!("- {} - {}", definition.name(), definition.description());
    }

    Ok(())
}
