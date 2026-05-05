use crate::command::parser::ParsedCommand;

use super::code_block::{last_assistant_message, last_code_block};
use super::session::CommandContext;

pub fn handle_fix_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    if let Some(code_block) = last_code_block(context) {
        context.stage_prompt_for_model(fix_code_prompt(&code_block));
        context.set_status("Asking the model to fix the last code block...".to_string());
        return;
    }

    if let Some(answer) = last_assistant_message(context) {
        context.stage_prompt_for_model(fix_message_prompt(&answer));
        context.set_status("Asking the model to correct its last answer...".to_string());
        return;
    }

    context.append_local_message(
        command.raw(),
        "Nothing to fix yet. Send a prompt or paste some code first, then run /fix.".to_string(),
    );
    context.set_status("No prior message to fix.".to_string());
}

fn fix_code_prompt(code_block: &str) -> String {
    format!(
        "Find and fix all bugs in this code. Return the corrected version as a \
         fenced code block, then a short explanation of every bug you found and \
         what the fix does. If the code is already correct, say so directly.\n\
         \n\
         ```\n{code_block}\n```"
    )
}

fn fix_message_prompt(answer: &str) -> String {
    format!(
        "Find and fix all factual or logical mistakes in the message below. \
         Return the corrected version, then a short explanation of each error \
         and how you fixed it. If the message is already correct, say so directly.\n\
         \n\
         ---\n{answer}\n---"
    )
}
