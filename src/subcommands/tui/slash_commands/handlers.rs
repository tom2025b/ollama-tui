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
pub use session::{
    AppLifecycle, CommandContext, CommandOutput, ContextMemory, ConversationControl,
    ExternalAction, HelpOverlay, HistoryEntry, HistoryExport, HistoryView, ModelActivity,
    ModelCatalog, ModelPicker, PromptStaging, RulesContext, Setting, SettingEdit, SettingsContext,
};
