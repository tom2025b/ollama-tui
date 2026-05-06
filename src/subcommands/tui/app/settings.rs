mod layout;
mod theme;

pub(super) use layout::LayoutMode;
pub(super) use theme::UiTheme;

use super::App;
use crate::subcommands::tui::slash_commands::handlers::{Setting, SettingEdit};

impl App {
    pub fn setting_report(&self, setting: Setting) -> String {
        match setting {
            Setting::Theme => self.theme_report(),
            Setting::Layout => self.layout_report(),
        }
    }

    pub fn set_setting(&mut self, setting: SettingEdit<'_>) -> Result<String, String> {
        match setting {
            SettingEdit::Theme(value) => self.set_theme(value),
            SettingEdit::Layout(value) => self.set_layout_mode(value),
        }
    }
}
