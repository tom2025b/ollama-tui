use super::super::App;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VoiceMode {
    Auto,
    Dictation,
    Command,
}

impl VoiceMode {
    fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dictation => "dictation",
            Self::Command => "command",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "dictation" | "dictate" => Some(Self::Dictation),
            "command" | "commands" => Some(Self::Command),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::app) struct VoiceSettings {
    enabled: bool,
    speed: f32,
    mode: VoiceMode,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            speed: 1.0,
            mode: VoiceMode::Auto,
        }
    }
}

impl App {
    pub fn voice_report(&self) -> String {
        format!(
            "Voice: {}\nMode: {}\nSpeed: {:.1}x",
            if self.voice.enabled { "on" } else { "off" },
            self.voice.mode.label(),
            self.voice.speed
        )
    }

    pub fn set_voice_enabled(&mut self, enabled: bool) -> String {
        self.voice.enabled = enabled;
        format!("Voice turned {}.", if enabled { "on" } else { "off" })
    }

    pub fn set_voice_speed(&mut self, requested: &str) -> Result<String, String> {
        let speed: f32 = requested
            .parse()
            .map_err(|_| "Usage: /voice speed <0.5-2.0>".to_string())?;
        if !(0.5..=2.0).contains(&speed) {
            return Err("Voice speed must be between 0.5 and 2.0.".to_string());
        }

        self.voice.speed = (speed * 10.0).round() / 10.0;
        Ok(format!("Voice speed set to {:.1}x.", self.voice.speed))
    }

    pub fn set_voice_mode(&mut self, requested: &str) -> Result<String, String> {
        let mode = VoiceMode::parse(&requested.to_ascii_lowercase())
            .ok_or_else(|| "Usage: /voice mode [auto|dictation|command]".to_string())?;
        self.voice.mode = mode;
        Ok(format!("Voice mode set to {}.", mode.label()))
    }
}
