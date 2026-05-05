use crate::command::parser::ParsedCommand;

use super::code_block::last_code_block;
use super::session::CommandContext;

pub fn handle_explain_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    let Some(code_block) = last_code_block(context) else {
        context.append_local_message(
            command.raw(),
            "No fenced code block was found in the recent conversation. \
             Paste some code into a prompt first, then run /explain again."
                .to_string(),
        );
        context.set_status("No code block to explain.".to_string());
        return;
    };

    context.stage_prompt_for_model(explain_prompt(&code_block));
    context.set_status(
        "Asking the model to explain the last code block...".to_string(),
    );
}

fn explain_prompt(code_block: &str) -> String {
    format!(
        "Explain the following code clearly, as if walking a junior engineer through it.\n\
         \n\
         Cover:\n\
         - What it does at a high level (one short paragraph).\n\
         - A line-by-line or block-by-block breakdown of the non-trivial parts.\n\
         - Any non-obvious behavior, edge cases, or gotchas a reader might miss.\n\
         - The language or framework if it can be inferred.\n\
         \n\
         Be concrete. Skip filler. Do not rewrite the code unless asked.\n\
         \n\
         ```\n\
         {code_block}\n\
         ```"
    )
}
