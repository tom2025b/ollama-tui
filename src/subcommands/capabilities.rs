use anyhow::Result;

use crate::tools::{registry::ToolRegistry, spec::ToolDefinition};

pub(super) fn public_tool_registry() -> Result<ToolRegistry> {
    Ok(ToolRegistry::with_builtins()?)
}

pub(super) fn sorted_tool_definitions(tools: &ToolRegistry) -> Vec<ToolDefinition> {
    let mut definitions = tools.definitions();
    definitions.sort_by(|left, right| left.name().cmp(right.name()));
    definitions
}
