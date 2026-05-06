use crate::tools::registry::{ToolRegistry, ToolRegistryError};

use super::api::ExtensionPack;

/// Public extension pack for the clean open-source build.
#[derive(Clone, Copy, Debug, Default)]
pub struct PublicExtensionPack;

impl ExtensionPack for PublicExtensionPack {
    fn register_tools(&self, _tools: &mut ToolRegistry) -> Result<(), ToolRegistryError> {
        Ok(())
    }
}

pub fn pack() -> PublicExtensionPack {
    PublicExtensionPack
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_pack_registers_no_tools_initially() {
        let mut tools = ToolRegistry::new();

        pack()
            .register_tools(&mut tools)
            .expect("public extension pack should register");

        assert!(tools.is_empty());
    }
}
