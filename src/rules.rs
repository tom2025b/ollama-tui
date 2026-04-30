use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

const PROJECT_RULES_DIR: &str = ".ollama-me";
const PROJECT_RULES_FILE: &str = "rules.md";

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "Gemfile",
    "Makefile",
];

/// Which rules file the user wants to edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RulesTarget {
    /// User-wide rules loaded from `~/.config/ollama-me/rules.md`.
    Global,

    /// Project-local rules loaded from `<project-root>/.ollama-me/rules.md`.
    Project,
}

impl RulesTarget {
    /// Human-readable label used in command output.
    pub fn label(self) -> &'static str {
        match self {
            RulesTarget::Global => "global rules",
            RulesTarget::Project => "project rules",
        }
    }
}

/// Loaded rules and the paths where they live.
#[derive(Clone, Debug)]
pub struct RulesState {
    enabled: bool,
    global_path: PathBuf,
    global_rules: Option<RulesFile>,
    project_root: Option<PathBuf>,
    project_rules: Option<RulesFile>,
    load_warnings: Vec<String>,
}

#[derive(Clone, Debug)]
struct RulesFile {
    path: PathBuf,
    raw_content: String,
}

impl RulesState {
    /// Load global rules and, when the process is inside a project, project
    /// rules from the nearest project root.
    pub fn load() -> Self {
        let home = home_dir();
        let global_path = global_rules_path(&home);
        let cwd = env::current_dir().unwrap_or_else(|_| home.clone());
        let project_root = find_project_root(&cwd);

        let mut load_warnings = Vec::new();
        let global_rules = read_optional_rules(&global_path, &mut load_warnings);
        let project_rules = project_root
            .as_ref()
            .map(|root| project_rules_path(root))
            .and_then(|path| read_optional_rules(&path, &mut load_warnings));

        Self {
            enabled: true,
            global_path,
            global_rules,
            project_root,
            project_rules,
            load_warnings,
        }
    }

    /// Return whether rules will be applied to new prompts.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Turn all rules on or off.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Preserve the current on/off state after a reload.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Current project root, if one was detected.
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Create a missing rules file and return the path to open in the editor.
    pub fn prepare_edit(&self, target: RulesTarget) -> io::Result<PathBuf> {
        let path = match target {
            RulesTarget::Global => self.global_path.clone(),
            RulesTarget::Project => self.project_edit_path(),
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !path.exists() {
            fs::write(&path, default_rules_template(target))?;
        }

        Ok(path)
    }

    /// Wrap a user prompt with the active rules, if any are enabled.
    pub fn prompt_with_rules(&self, prompt: &str) -> String {
        let sections = self.active_rule_sections();

        if !self.enabled || sections.is_empty() {
            return prompt.to_string();
        }

        let mut wrapped = String::from(
            "Use these persistent ollama-me rules for this answer. If the current user request explicitly conflicts with a style preference, follow the current request.\n\n",
        );

        for section in sections {
            wrapped.push_str(&format!(
                "[{} from {}]\n{}\n\n",
                section.label,
                section.path.display(),
                section.content
            ));
        }

        wrapped.push_str("Current user request:\n");
        wrapped.push_str(prompt);
        wrapped
    }

    /// Short status suitable for the TUI status panel.
    pub fn status_line(&self) -> String {
        if !self.enabled {
            return "off".to_string();
        }

        let active = self
            .active_rule_sections()
            .into_iter()
            .map(|section| section.short_label)
            .collect::<Vec<_>>();

        if active.is_empty() {
            "on (none loaded)".to_string()
        } else {
            format!("on ({})", active.join(" + "))
        }
    }

    /// Summary appended to route explanations when rules are applied.
    pub fn application_summary(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let active = self
            .active_rule_sections()
            .into_iter()
            .map(|section| section.short_label)
            .collect::<Vec<_>>();

        if active.is_empty() {
            None
        } else {
            Some(format!("Applied {} rules.", active.join(" and ")))
        }
    }

    /// Full local report for `/rules show`.
    pub fn report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "Rules are {}.\n\n",
            if self.enabled { "on" } else { "off" }
        ));

        report.push_str(&format!(
            "Global rules: {} ({})\n",
            self.global_path.display(),
            rules_file_state(self.global_rules.as_ref())
        ));

        if let Some(root) = &self.project_root {
            report.push_str(&format!("Project root: {}\n", root.display()));
            report.push_str(&format!(
                "Project rules: {} ({})\n",
                project_rules_path(root).display(),
                rules_file_state(self.project_rules.as_ref())
            ));
        } else {
            report.push_str("Project root: none detected from the current directory\n");
            report.push_str(&format!(
                "Project rules edit path: {}\n",
                self.project_edit_path().display()
            ));
        }

        if !self.load_warnings.is_empty() {
            report.push_str("\nLoad warnings:\n");
            for warning in &self.load_warnings {
                report.push_str("- ");
                report.push_str(warning);
                report.push('\n');
            }
        }

        let sections = self.loaded_rule_sections();
        if sections.is_empty() {
            report.push_str("\nNo rule text is loaded. Lines beginning with # are ignored.\n");
        } else {
            report.push_str("\nLoaded rule text:\n");
            for section in sections {
                report.push_str(&format!("\n[{}]\n{}\n", section.label, section.content));
            }
        }

        report.push_str(
            "\nCommands: /rules global, /rules project, /rules show, /rules off, /rules on, /rules toggle\n",
        );

        report
    }

    fn project_edit_path(&self) -> PathBuf {
        self.project_root
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| home_dir()))
            .join(PROJECT_RULES_DIR)
            .join(PROJECT_RULES_FILE)
    }

    fn active_rule_sections(&self) -> Vec<RuleSection> {
        if !self.enabled {
            return Vec::new();
        }

        self.loaded_rule_sections()
    }

    fn loaded_rule_sections(&self) -> Vec<RuleSection> {
        let mut sections = Vec::new();

        if let Some(global_rules) = active_section("Global rules", "global", &self.global_rules) {
            sections.push(global_rules);
        }

        if let Some(project_rules) = active_section("Project rules", "project", &self.project_rules)
        {
            sections.push(project_rules);
        }

        sections
    }
}

#[derive(Clone, Debug)]
struct RuleSection {
    label: &'static str,
    short_label: &'static str,
    path: PathBuf,
    content: String,
}

fn active_section(
    label: &'static str,
    short_label: &'static str,
    rules_file: &Option<RulesFile>,
) -> Option<RuleSection> {
    let rules_file = rules_file.as_ref()?;
    let content = active_rule_content(&rules_file.raw_content);

    if content.is_empty() {
        return None;
    }

    Some(RuleSection {
        label,
        short_label,
        path: rules_file.path.clone(),
        content,
    })
}

fn read_optional_rules(path: &Path, warnings: &mut Vec<String>) -> Option<RulesFile> {
    match fs::read_to_string(path) {
        Ok(raw_content) => Some(RulesFile {
            path: path.to_path_buf(),
            raw_content,
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            warnings.push(format!("failed to read {}: {error}", path.display()));
            None
        }
    }
}

fn rules_file_state(rules_file: Option<&RulesFile>) -> &'static str {
    match rules_file {
        None => "not created",
        Some(file) if active_rule_content(&file.raw_content).is_empty() => "empty",
        Some(_) => "active",
    }
}

fn active_rule_content(raw_content: &str) -> String {
    let mut lines = Vec::new();
    let mut in_html_comment = false;

    for line in raw_content.lines() {
        let trimmed = line.trim();

        if in_html_comment {
            if trimmed.contains("-->") {
                in_html_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                in_html_comment = true;
            }
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        lines.push(line);
    }

    lines.join("\n").trim().to_string()
}

fn default_rules_template(target: RulesTarget) -> String {
    format!(
        "# ollama-me {}\n# Lines beginning with # are ignored by ollama-me.\n# Add persistent instructions below. Leave blank to disable this file.\n",
        target.label()
    )
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        if PROJECT_MARKERS
            .iter()
            .any(|marker| current.join(marker).exists())
        {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

fn global_rules_path(home: &Path) -> PathBuf {
    home.join(".config").join("ollama-me").join("rules.md")
}

fn project_rules_path(project_root: &Path) -> PathBuf {
    project_root
        .join(PROJECT_RULES_DIR)
        .join(PROJECT_RULES_FILE)
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_rule_content_ignores_template_comments() {
        let raw = "# heading\n\n- Prefer short answers.\n<!-- hidden -->\nUse local context.";

        assert_eq!(
            active_rule_content(raw),
            "- Prefer short answers.\nUse local context."
        );
    }

    #[test]
    fn prompt_is_unchanged_when_rules_are_disabled() {
        let mut rules = RulesState {
            enabled: false,
            global_path: PathBuf::from("/tmp/rules.md"),
            global_rules: Some(RulesFile {
                path: PathBuf::from("/tmp/rules.md"),
                raw_content: "Prefer short answers.".to_string(),
            }),
            project_root: None,
            project_rules: None,
            load_warnings: Vec::new(),
        };

        assert_eq!(rules.prompt_with_rules("hello"), "hello");
        rules.set_enabled(true);
        assert!(
            rules
                .prompt_with_rules("hello")
                .contains("Prefer short answers.")
        );
    }
}
