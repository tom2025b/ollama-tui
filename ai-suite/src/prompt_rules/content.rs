use std::path::PathBuf;

use super::storage::RulesFile;

#[derive(Clone, Debug)]
pub(super) struct RuleSection {
    pub(super) label: &'static str,
    pub(super) short_label: &'static str,
    pub(super) path: PathBuf,
    pub(super) content: String,
}

pub(super) fn active_section(
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

pub(super) fn active_rule_content(raw_content: &str) -> String {
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
