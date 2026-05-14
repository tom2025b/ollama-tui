pub type ToolInput = serde_json::Value;

/// Structured request data passed to a resolved tool.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolInvocation {
    name: String,
    input: ToolInput,
}

impl ToolInvocation {
    pub fn new(name: impl Into<String>, input: ToolInput) -> Self {
        Self {
            name: name.into(),
            input,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn input(&self) -> &ToolInput {
        &self.input
    }
}

/// Text output from a tool execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolOutput {
    content: String,
}

impl ToolOutput {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}
