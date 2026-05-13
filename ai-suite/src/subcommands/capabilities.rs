use crate::{
    Result,
    extensions::ExtensionRegistry,
    tools::{registry::ToolRegistry, spec::ToolDefinition},
};

pub(super) fn public_tool_registry() -> Result<ToolRegistry> {
    let mut tools = ToolRegistry::with_builtins()?;
    ExtensionRegistry::public().register_tools(&mut tools)?;
    Ok(tools)
}

pub(super) fn sorted_tool_definitions(tools: &ToolRegistry) -> Vec<ToolDefinition> {
    let mut definitions = tools.definitions();
    definitions.sort_by(|left, right| left.name().cmp(right.name()));
    definitions
}
