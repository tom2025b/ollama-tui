use super::spec::{Tool, ToolDefinition};
use crate::{Error, Result};

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
            return Err(Error::tool(format!("tool '{name}' is already registered")));
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

#[cfg(test)]
mod tests {
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
    fn with_builtins_registers_public_tools() {
        let registry = match ToolRegistry::with_builtins() {
            Ok(registry) => registry,
            Err(error) => panic!("built-in registration should succeed: {error}"),
        };

        assert_eq!(registry.len(), 2);
        assert!(registry.contains("utc_timestamp"));
        assert!(registry.contains("build_info"));
    }

    #[test]
    fn registry_registers_and_resolves_tools() {
        let mut registry = ToolRegistry::new();
        if let Err(error) = registry.register(StaticTool::new("static")) {
            panic!("tool should register: {error}");
        }

        let tool = match registry.resolve("static") {
            Some(tool) => tool,
            None => panic!("tool should resolve"),
        };
        let output = match tool.execute(ToolInvocation::new("static", serde_json::Value::Null)) {
            Ok(output) => output,
            Err(error) => panic!("tool should execute: {error}"),
        };

        assert!(registry.contains("static"));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.definitions()[0].name(), "static");
        assert_eq!(output.content(), "ran static");
    }

    #[test]
    fn registry_rejects_duplicate_names() {
        let mut registry = ToolRegistry::new();
        if let Err(error) = registry.register(StaticTool::new("static")) {
            panic!("first tool should register: {error}");
        }

        let error = match registry.register(StaticTool::new("static")) {
            Ok(()) => panic!("duplicate tool should fail"),
            Err(error) => error,
        };

        match error {
            Error::Tool { message } => {
                assert_eq!(message, "tool 'static' is already registered");
            }
            other => panic!("expected tool registration error, got {other}"),
        }
    }
}
