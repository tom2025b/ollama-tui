#[derive(Clone, Copy, Debug)]
pub struct CommandSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub hint: &'static str,
    pub detail: &'static str,
    pub usage: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCommand {
    raw: String,
    name: String,
    args: Vec<String>,
}

impl ParsedCommand {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "/clear",
        aliases: &[],
        hint: "Clear visible conversation",
        detail: "Clear the chat transcript in this window.",
        usage: "/clear",
    },
    CommandSpec {
        name: "/model",
        aliases: &[],
        hint: "Open or set model",
        detail: "Open the model picker, choose Auto, or pin an enabled model by name.",
        usage: "/model [auto|model name]",
    },
    CommandSpec {
        name: "/models",
        aliases: &["/backend", "/backends"],
        hint: "List backend readiness",
        detail: "Show every configured model and why unavailable backends are disabled.",
        usage: "/models",
    },
    CommandSpec {
        name: "/route",
        aliases: &[],
        hint: "Trace router choice",
        detail: "Run the router on a prompt without calling a model.",
        usage: "/route <prompt>",
    },
    CommandSpec {
        name: "/summary",
        aliases: &[],
        hint: "Summarize session",
        detail: "Show turn counts, active routing mode, and last model used.",
        usage: "/summary",
    },
    CommandSpec {
        name: "/debug",
        aliases: &[],
        hint: "Toggle verbose errors",
        detail: "Switch between concise and full backend error messages.",
        usage: "/debug",
    },
    CommandSpec {
        name: "/help",
        aliases: &[],
        hint: "Show command help",
        detail: "Open the command reference overlay.",
        usage: "/help",
    },
    CommandSpec {
        name: "/quit",
        aliases: &["/exit", "/q"],
        hint: "Quit the app",
        detail: "Close the desktop window.",
        usage: "/quit",
    },
];

pub fn parse_slash_command(input: &str) -> Option<ParsedCommand> {
    let raw = input.trim();
    if !raw.starts_with('/') {
        return None;
    }

    let mut parts = raw.split_whitespace();
    let name = parts.next().unwrap_or_default().to_ascii_lowercase();
    let args = parts.map(str::to_string).collect();

    Some(ParsedCommand {
        raw: raw.to_string(),
        name,
        args,
    })
}

pub fn suggestions(input: &str) -> Vec<&'static CommandSpec> {
    let Some(prefix) = command_prefix(input) else {
        return Vec::new();
    };

    let prefix = prefix.to_ascii_lowercase();
    COMMANDS
        .iter()
        .filter(|command| command.matches_prefix(&prefix))
        .collect()
}

pub fn find_command(name: &str) -> Option<&'static CommandSpec> {
    COMMANDS
        .iter()
        .find(|command| command.name == name || command.aliases.contains(&name))
}

pub fn command_body(command: &ParsedCommand) -> String {
    command.args().join(" ")
}

pub fn unquote(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn command_prefix(input: &str) -> Option<&str> {
    if !input.starts_with('/') || input.chars().any(char::is_whitespace) {
        return None;
    }
    Some(input)
}

impl CommandSpec {
    pub fn needs_argument(self) -> bool {
        self.usage.contains('<')
    }

    fn matches_prefix(&self, prefix: &str) -> bool {
        self.name.starts_with(prefix) || self.aliases.iter().any(|alias| alias.starts_with(prefix))
    }
}
