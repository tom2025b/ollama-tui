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
