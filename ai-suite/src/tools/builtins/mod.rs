//! Built-in public tools shipped with the open-source `ai-suite` build.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;

use super::{
    execution::{ToolInvocation, ToolOutput},
    registry::ToolRegistry,
    spec::{Tool, ToolDefinition},
};
use crate::Result;

/// Register built-in provider-neutral tools.
pub fn register(registry: &mut ToolRegistry) -> Result<()> {
    registry.register(UtcTimestampTool)?;
    registry.register(BuildInfoTool)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct UtcTimestampTool;

impl Tool for UtcTimestampTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "utc_timestamp",
            "Return the current UTC Unix timestamp from the local machine",
        )
    }

    fn execute(&self, _invocation: ToolInvocation) -> Result<ToolOutput> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("system clock is before Unix epoch: {e}"))?
            .as_secs();

        Ok(ToolOutput::text(timestamp.to_string()))
    }
}

#[derive(Clone, Copy, Debug)]
struct BuildInfoTool;

impl Tool for BuildInfoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("build_info", "Return public ai-suite build metadata")
    }

    fn execute(&self, _invocation: ToolInvocation) -> Result<ToolOutput> {
        Ok(ToolOutput::text(format!(
            "ai-suite {} ({})",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_NAME")
        )))
    }
}
