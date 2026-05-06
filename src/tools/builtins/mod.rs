use super::registry::{ToolRegistry, ToolRegistryError};

/// Register built-in provider-neutral tools.
pub fn register(_registry: &mut ToolRegistry) -> Result<(), ToolRegistryError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_registration_is_initially_empty() {
        let mut registry = ToolRegistry::new();

        register(&mut registry).expect("built-in registration should succeed");

        assert!(registry.is_empty());
    }
}
