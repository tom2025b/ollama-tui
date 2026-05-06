use std::{error::Error, fmt};

use super::spec::{Tool, ToolDefinition};

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

    pub fn with_builtins() -> Result<Self, ToolRegistryError> {
        let mut registry = Self::new();
        super::builtins::register(&mut registry)?;
        Ok(registry)
    }

    pub fn register<T>(&mut self, tool: T) -> Result<(), ToolRegistryError>
    where
        T: Tool + 'static,
    {
        let definition = tool.definition();
        let name = definition.name().to_string();
        if self.contains(&name) {
            return Err(ToolRegistryError::DuplicateName(name));
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolRegistryError {
    DuplicateName(String),
}

impl fmt::Display for ToolRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName(name) => write!(formatter, "tool '{name}' is already registered"),
        }
    }
}

impl Error for ToolRegistryError {}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::tools::execution::{ToolInvocation, ToolOutput};

    struct StaticTool {
        name: &'static str,
    }

    impl StaticTool {
        fn new(name: &'static str) -> Self {
            Self { name }
        }
    }

    impl Tool for StaticTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition::new(self.name, "Static test tool")
        }

        fn execute(&self, invocation: ToolInvocation) -> Result<ToolOutput> {
            Ok(ToolOutput::text(format!("ran {}", invocation.name())))
        }
    }

    #[test]
    fn new_registry_starts_empty() {
        let registry = ToolRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(!registry.contains("search"));
    }

    #[test]
    fn with_builtins_starts_empty_until_tools_are_added() {
        let registry = ToolRegistry::with_builtins().expect("built-in registration should succeed");

        assert!(registry.is_empty());
    }

    #[test]
    fn registry_registers_and_resolves_tools() {
        let mut registry = ToolRegistry::new();
        registry
            .register(StaticTool::new("static"))
            .expect("tool should register");

        let tool = registry.resolve("static").expect("tool should resolve");
        let output = tool
            .execute(ToolInvocation::new("static", serde_json::Value::Null))
            .expect("tool should execute");

        assert!(registry.contains("static"));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.definitions()[0].name(), "static");
        assert_eq!(output.content(), "ran static");
    }

    #[test]
    fn registry_rejects_duplicate_names() {
        let mut registry = ToolRegistry::new();
        registry
            .register(StaticTool::new("static"))
            .expect("first tool should register");

        let error = registry
            .register(StaticTool::new("static"))
            .expect_err("duplicate tool should fail");

        assert_eq!(
            error,
            ToolRegistryError::DuplicateName("static".to_string())
        );
    }
}
