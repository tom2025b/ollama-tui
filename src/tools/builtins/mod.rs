use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use super::registry::{ToolRegistry, ToolRegistryError};
use super::{
    execution::{ToolInvocation, ToolOutput},
    spec::{Tool, ToolDefinition},
};

/// Register built-in provider-neutral tools.
pub fn register(registry: &mut ToolRegistry) -> Result<(), ToolRegistryError> {
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
            .context("system clock is before Unix epoch")?
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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn builtin_registration_adds_public_tools() {
        let mut registry = ToolRegistry::new();

        register(&mut registry).expect("built-in registration should succeed");

        assert!(registry.contains("utc_timestamp"));
        assert!(registry.contains("build_info"));
    }

    #[test]
    fn utc_timestamp_returns_digits() {
        let output = UtcTimestampTool
            .execute(ToolInvocation::new("utc_timestamp", Value::Null))
            .expect("timestamp should be available");

        assert!(
            output
                .content()
                .chars()
                .all(|character| character.is_ascii_digit())
        );
    }

    #[test]
    fn build_info_uses_public_package_name() {
        let output = BuildInfoTool
            .execute(ToolInvocation::new("build_info", Value::Null))
            .expect("build info should be available");

        assert!(output.content().contains("ai-suite"));
    }
}
