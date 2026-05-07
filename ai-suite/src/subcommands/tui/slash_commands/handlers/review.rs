use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::code_block::last_code_block;

pub fn review_command(app: &mut App, command: &ParsedCommand) {
    let Some(code_block) = last_code_block(app) else {
        app.append_local_message(
            command.raw(),
            "No fenced code block was found in the recent conversation. \
             Paste code into a prompt first, then run /review again."
                .to_string(),
        );
        app.ui.status = "No code block to review.".to_string();
        return;
    };

    app.commands.stage_prompt(review_prompt(&code_block));
    app.ui.status = "Asking the model to review the last code block...".to_string();
}

fn review_prompt(code_block: &str) -> String {
    format!(
        "Review the following code as a senior engineer doing a brutal-but-fair code review.\n\
         \n\
         Call out, with specifics:\n\
         - Bugs, off-by-one errors, missing error handling, and broken edge cases.\n\
         - Security or safety issues (injection, unsafe defaults, leaked secrets, panics).\n\
         - Performance pitfalls (allocations in hot paths, accidental quadratic work).\n\
         - Readability, naming, and structural problems worth fixing.\n\
         \n\
         Quote the offending line or expression when you flag something. Be concrete\n\
         and actionable. If the code is genuinely fine, say so and explain why instead\n\
         of inventing concerns.\n\
         \n\
         ```\n\
         {code_block}\n\
         ```"
    )
}
