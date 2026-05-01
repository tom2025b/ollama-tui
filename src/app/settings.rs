mod layout;
mod theme;
mod voice;

pub(super) use layout::LayoutMode;
pub(super) use theme::UiTheme;
pub(super) use voice::VoiceSettings;

use super::App;
use crate::command::handlers::{Setting, SettingEdit};

impl App {
    pub fn setting_report(&self, setting: Setting) -> String {
        match setting {
            Setting::Theme => self.theme_report(),
            Setting::Layout => self.layout_report(),
            Setting::Voice => self.voice_report(),
        }
    }

    pub fn set_setting(&mut self, setting: SettingEdit<'_>) -> Result<String, String> {
        match setting {
            SettingEdit::Theme(value) => self.set_theme(value),
            SettingEdit::Layout(value) => self.set_layout_mode(value),
            SettingEdit::VoiceEnabled(enabled) => Ok(self.set_voice_enabled(enabled)),
            SettingEdit::VoiceSpeed(value) => self.set_voice_speed(value),
            SettingEdit::VoiceMode(value) => self.set_voice_mode(value),
        }
    }
}
