pub mod backends;
pub mod clear;
pub mod code_block;
pub mod context_memory;
pub mod explain;
pub mod fix;
pub mod history;
pub mod history_output;
pub mod review;
pub mod rules;
pub mod session;
pub mod ui_quality;

pub use rules::complete_rules_edit;
pub use session::ExternalAction;

/// Truncate text to 80 characters for display, appending `...` when cut.
fn preview(text: &str) -> String {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        head
    }
}
