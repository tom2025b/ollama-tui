use super::spec::Subcommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BuiltInSubcommand {
    name: &'static str,
}

impl BuiltInSubcommand {
    const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl Subcommand for BuiltInSubcommand {
    fn name(&self) -> &'static str {
        self.name
    }
}

const BUILT_IN_SUBCOMMANDS: &[BuiltInSubcommand] = &[
    BuiltInSubcommand::new("tui"),
    BuiltInSubcommand::new("swarm"),
    BuiltInSubcommand::new("food"),
];

pub fn names() -> impl Iterator<Item = &'static str> {
    BUILT_IN_SUBCOMMANDS.iter().map(|command| command.name())
}

pub fn contains(name: &str) -> bool {
    names().any(|registered_name| registered_name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lists_expected_initial_subcommands() {
        let names = names().collect::<Vec<_>>();

        assert_eq!(names, ["tui", "swarm", "food"]);
    }

    #[test]
    fn registry_checks_subcommand_names() {
        assert!(contains("tui"));
        assert!(contains("swarm"));
        assert!(contains("food"));
        assert!(!contains("unknown"));
    }
}
