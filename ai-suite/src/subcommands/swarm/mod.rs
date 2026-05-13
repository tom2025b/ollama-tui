use crate::Result;
use crate::routing::{ModelRouter, Router};
use crate::runtime::Runtime;

pub async fn run(runtime: &Runtime) -> Result<()> {
    let router = ModelRouter::new(runtime.config().models());
    let tools = crate::subcommands::capabilities::public_tool_registry()?;
    let tool_definitions = crate::subcommands::capabilities::sorted_tool_definitions(&tools);

    println!("Swarm readiness");
    println!("Models:");
    for model in router.models() {
        let state = if model.enabled { "ready" } else { "setup" };
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
