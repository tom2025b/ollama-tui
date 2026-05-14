use anyhow::anyhow;

use super::spec::{Tool, ToolDefinition};
use crate::Result;

/// Registry for provider-neutral tools shared by top-level subcommands.
#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<RegisteredTool>,
}

struct RegisteredTool {
    definition: ToolDefinition,
    tool: Box<dyn Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtins() -> Result<Self> {
        let mut registry = Self::new();
        super::builtins::register(&mut registry)?;
        Ok(registry)
    }

    pub fn register<T>(&mut self, tool: T) -> Result<()>
    where
        T: Tool + 'static,
    {
        let definition = tool.definition();
        let name = definition.name().to_string();
        if self.contains(&name) {
            return Err(anyhow!("tool '{name}' is already registered"));
        }

        self.tools.push(RegisteredTool {
            definition,
            tool: Box::new(tool),
        });
        Ok(())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }

    pub fn resolve(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|registered| registered.definition.name() == name)
            .map(|registered| registered.tool.as_ref())
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .map(|registered| registered.definition.clone())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }
}
