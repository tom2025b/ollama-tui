mod context;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;
pub use context::{
    AppLifecycle, CommandContext, CommandOutput, ContextMemory, ConversationControl,
    ExternalAction, HelpOverlay, HistoryEntry, HistoryExport, HistoryView, ModelActivity,
    ModelCatalog, ModelPicker, PromptStaging, RulesContext, Setting, SettingEdit, SettingsContext,
};

pub fn open_models_command<C>(context: &mut C, _command: &ParsedCommand)
where
    C: ModelPicker + ?Sized,
{
    context.open_models_picker();
}

pub fn execute_open_models_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    open_models_command(context, command);
}

pub fn open_help_command<C>(context: &mut C, _command: &ParsedCommand)
where
    C: HelpOverlay + ?Sized,
{
    context.open_help_overlay();
}

pub fn execute_open_help_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    open_help_command(context, command);
}

pub fn quit_command<C>(context: &mut C, _command: &ParsedCommand)
where
    C: AppLifecycle + ?Sized,
{
    context.quit();
}

pub fn execute_quit_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    quit_command(context, command);
}

pub fn unknown_command<C>(context: &mut C, command: &ParsedCommand, available_commands: &str)
where
    C: CommandOutput + ?Sized,
{
    context.append_local_message(
        command.raw(),
        format!("Unknown command. Available commands: {available_commands}."),
    );
    context.set_status("Unknown command.".to_string());
}
