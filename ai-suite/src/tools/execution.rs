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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn invocation_exposes_name_and_input() {
        let invocation = ToolInvocation::new("search", json!({ "query": "ToolRegistry" }));

        assert_eq!(invocation.name(), "search");
        assert_eq!(invocation.input(), &json!({ "query": "ToolRegistry" }));
    }

    #[test]
    fn text_output_exposes_content() {
        let output = ToolOutput::text("done");

        assert_eq!(output.content(), "done");
    }
}
