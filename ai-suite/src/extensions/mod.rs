//! Public extension registration for provider-neutral tools.

use crate::tools::{
    execution::{ToolInvocation, ToolOutput},
    spec::{Tool, ToolDefinition},
};
use crate::{Result, tools::registry::ToolRegistry};

/// Extension point for registering provider-neutral capabilities.
pub trait ExtensionPack: Send + Sync {
    fn register_tools(&self, tools: &mut ToolRegistry) -> Result<()>;
}

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
        registry.register(PublicExtensionPack);
        registry
    }

    pub fn register<T>(&mut self, pack: T)
    where
        T: ExtensionPack + 'static,
    {
        self.packs.push(Box::new(pack));
    }

    pub fn register_tools(&self, tools: &mut ToolRegistry) -> Result<()> {
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

/// Public extension pack for the clean open-source build.
#[derive(Clone, Copy, Debug, Default)]
struct PublicExtensionPack;

impl ExtensionPack for PublicExtensionPack {
    fn register_tools(&self, tools: &mut ToolRegistry) -> Result<()> {
        tools.register(PublicProfileTool)
    }
}

#[derive(Clone, Copy, Debug)]
struct PublicProfileTool;

impl Tool for PublicProfileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "public_profile",
            "Return the enabled public extension profile",
        )
    }

    fn execute(&self, _invocation: ToolInvocation) -> Result<ToolOutput> {
        Ok(ToolOutput::text("public ai-suite extension profile"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StaticExtensionPack;

    impl ExtensionPack for StaticExtensionPack {
        fn register_tools(&self, tools: &mut ToolRegistry) -> Result<()> {
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

        if let Err(error) = registry.register_tools(&mut tools) {
            panic!("public extensions should register: {error}");
        }

        assert_eq!(registry.len(), 1);
        assert!(tools.contains("public_profile"));
    }

    #[test]
    fn public_pack_registers_public_profile_tool() {
        let mut tools = ToolRegistry::new();

        if let Err(error) = PublicExtensionPack.register_tools(&mut tools) {
            panic!("public extension pack should register: {error}");
        }

        assert!(tools.contains("public_profile"));
    }

    #[test]
    fn registry_applies_registered_extension_packs() {
        let mut registry = ExtensionRegistry::new();
        let mut tools = ToolRegistry::new();
        registry.register(StaticExtensionPack);

        if let Err(error) = registry.register_tools(&mut tools) {
            panic!("extension pack should register tools: {error}");
        }

        assert!(tools.contains("static"));
    }
}
