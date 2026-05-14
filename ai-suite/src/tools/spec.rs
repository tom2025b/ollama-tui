use crate::Result;

use super::execution::{ToolInvocation, ToolOutput};

/// Provider-neutral metadata for a reusable tool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDefinition {
    name: String,
    description: String,
}

impl ToolDefinition {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// A provider-neutral tool that can be reused by any top-level subcommand.
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool against structured input and return text output.
    fn execute(&self, invocation: ToolInvocation) -> Result<ToolOutput>;
}
