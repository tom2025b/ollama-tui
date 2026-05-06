use crate::tools::registry::{ToolRegistry, ToolRegistryError};

use super::{api::ExtensionPack, public};

/// Registry of extension packs enabled for this build.
#[derive(Default)]
pub struct ExtensionRegistry {
    packs: Vec<Box<dyn ExtensionPack>>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn public() -> Self {
        let mut registry = Self::new();
        registry.register(public::pack());
        registry
    }

    pub fn register<T>(&mut self, pack: T)
    where
        T: ExtensionPack + 'static,
    {
        self.packs.push(Box::new(pack));
    }

    pub fn register_tools(&self, tools: &mut ToolRegistry) -> Result<(), ToolRegistryError> {
        for pack in &self.packs {
            pack.register_tools(tools)?;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.packs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.packs.len()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::tools::{
        execution::{ToolInvocation, ToolOutput},
        spec::{Tool, ToolDefinition},
    };

    struct StaticExtensionPack;

    impl ExtensionPack for StaticExtensionPack {
        fn register_tools(&self, tools: &mut ToolRegistry) -> Result<(), ToolRegistryError> {
            tools.register(StaticTool)
        }
    }

    struct StaticTool;

    impl Tool for StaticTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition::new("static", "Static test tool")
        }

        fn execute(&self, _invocation: ToolInvocation) -> Result<ToolOutput> {
            Ok(ToolOutput::text("static"))
        }
    }

    #[test]
    fn new_registry_starts_empty() {
        let registry = ExtensionRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn public_registry_contains_public_pack_only() {
        let registry = ExtensionRegistry::public();
        let mut tools = ToolRegistry::new();

        registry
            .register_tools(&mut tools)
            .expect("public extensions should register");

        assert_eq!(registry.len(), 1);
        assert!(tools.is_empty());
    }

    #[test]
    fn registry_applies_registered_extension_packs() {
        let mut registry = ExtensionRegistry::new();
        let mut tools = ToolRegistry::new();
        registry.register(StaticExtensionPack);

        registry
            .register_tools(&mut tools)
            .expect("extension pack should register tools");

        assert!(tools.contains("static"));
    }
}
