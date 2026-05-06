use crate::tools::registry::{ToolRegistry, ToolRegistryError};

/// Extension point for registering provider-neutral capabilities.
pub trait ExtensionPack: Send + Sync {
    fn register_tools(&self, tools: &mut ToolRegistry) -> Result<(), ToolRegistryError>;
}
