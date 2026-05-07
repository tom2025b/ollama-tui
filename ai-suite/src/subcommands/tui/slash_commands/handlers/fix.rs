use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::code_block::{last_assistant_message, last_code_block};

pub fn fix_command(app: &mut App, command: &ParsedCommand) {
    if let Some(code_block) = last_code_block(app) {
        app.commands.stage_prompt(fix_code_prompt(&code_block));
        app.ui.status = "Asking the model to fix the last code block...".to_string();
        return;
    }

    if let Some(answer) = last_assistant_message(app) {
        app.commands.stage_prompt(fix_message_prompt(&answer));
        app.ui.status = "Asking the model to correct its last answer...".to_string();
        return;
    }

    app.append_local_message(
        command.raw(),
        "Nothing to fix yet. Send a prompt or paste some code first, then run /fix.".to_string(),
    );
    app.ui.status = "No prior message to fix.".to_string();
}

fn fix_code_prompt(code_block: &str) -> String {
    format!(
        "Analyze this code for bugs. Check specifically for:\n\
         - Logic errors and incorrect control flow\n\
         - Off-by-one errors and boundary conditions\n\
         - Unhandled null, empty, missing, or error cases\n\
         - Incorrect assumptions about the input or caller\n\
         - Resource leaks, missing cleanup, or unsafe state\n\
         \n\
         Return:\n\
         1. The corrected code as a fenced code block (unchanged if no bugs found)\n\
         2. A concise bullet list of each bug and why it was wrong — \
         or \"No bugs found\" with a one-line justification\n\
         \n\
         Don't suggest style changes or refactors unless they directly mask a \
         correctness issue.\n\
         \n\
         ```\n{code_block}\n```"
    )
}

fn fix_message_prompt(answer: &str) -> String {
    format!(
        "Check the following message for factual errors, logical mistakes, or \
         incorrect technical claims. For each problem found:\n\
         - Quote the specific claim that is wrong\n\
         - Explain why it is wrong\n\
         - Give the correct version\n\
         \n\
         If the message is accurate, say so in one sentence and briefly explain why.\n\
         \n\
         ---\n{answer}\n---"
    )
}
