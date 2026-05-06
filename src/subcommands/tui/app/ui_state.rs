use super::settings::{LayoutMode, UiTheme};

/// Presentation state for overlays, scrolling, and visual preferences.
pub(crate) struct UiState {
    /// Short status line shown near the input box.
    pub(crate) status: String,

    /// True when the help overlay should be shown.
    pub(crate) show_help: bool,

    /// How many lines the user has scrolled up from the bottom of the chat
    /// history. Zero means pinned to the newest content.
    pub(crate) scroll_offset: usize,

    /// True while the interactive `/model` picker overlay is visible.
    pub(crate) show_models_picker: bool,

    pub(in crate::subcommands::tui::app) models_picker_index: usize,
    pub(in crate::subcommands::tui::app) theme: UiTheme,
    pub(in crate::subcommands::tui::app) layout_mode: LayoutMode,
}

impl UiState {
    pub(super) fn new() -> Self {
        Self {
            status: "Type a prompt. Press Enter to send. Press Esc or Ctrl-C to quit.".to_string(),
            show_help: false,
            scroll_offset: 0,
            show_models_picker: false,
            models_picker_index: 0,
            theme: UiTheme::Dark,
            layout_mode: LayoutMode::Normal,
        }
    }
}
