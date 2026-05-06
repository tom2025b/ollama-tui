use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::code_block::last_code_block;

pub fn explain_command(app: &mut App, command: &ParsedCommand) {
    let Some(code_block) = last_code_block(app) else {
        app.append_local_message(
            command.raw(),
            "No fenced code block was found in the recent conversation. \
             Paste some code into a prompt first, then run /explain again."
                .to_string(),
        );
        app.ui.status = "No code block to explain.".to_string();
        return;
    };

    app.commands.stage_prompt(explain_prompt(&code_block));
    app.ui.status = "Asking the model to explain the last code block...".to_string();
}

fn explain_prompt(code_block: &str) -> String {
    format!(
        "Explain this code to an intermediate developer who hasn't seen it before.\n\
         \n\
         Cover these four things:\n\
         1. Purpose — one sentence: what problem does this solve?\n\
         2. How it works — the core logic and data flow. Focus on the non-obvious \
         parts; skip anything self-explanatory from the names alone.\n\
         3. Key decisions — why is it written this way? Call out any patterns, \
         idioms, or trade-offs a reader needs to understand to work with this \
         code confidently.\n\
         4. Watch out for — hidden assumptions, gotchas, or invariants that must \
         hold. What would surprise someone modifying this code for the first time?\n\
         \n\
         Be direct. Don't narrate what the code already says through good naming. \
         Focus on insight, not description.\n\
         \n\
         ```\n{code_block}\n```"
    )
}
