use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::app::App;

pub const DEFAULT_TEXT_SCALE: f32 = 1.25;
pub const MIN_TEXT_SCALE: f32 = 1.0;
pub const MAX_TEXT_SCALE: f32 = 1.8;
const TEXT_SCALE_STEP: f32 = 0.1;

pub struct GuiPreferences {
    pub text_scale: f32,
}

impl GuiPreferences {
    pub fn load() -> Self {
        let text_scale = preferences_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| parse_text_scale(&content))
            .unwrap_or(DEFAULT_TEXT_SCALE);

        Self {
            text_scale: clamp_text_scale(text_scale),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = preferences_path().ok_or_else(|| "Could not resolve config path".to_string())?;
        let parent = path
            .parent()
            .ok_or_else(|| "Could not resolve config directory".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        fs::write(
            path.as_path(),
            format!("text_scale = {:.2}\n", self.text_scale),
        )
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
    }
}

impl App {
    pub(crate) fn increase_text_size(&mut self) {
        self.set_text_scale(self.text_scale + TEXT_SCALE_STEP);
    }

    pub(crate) fn decrease_text_size(&mut self) {
        self.set_text_scale(self.text_scale - TEXT_SCALE_STEP);
    }

    pub(crate) fn text_size(&self, size: f32) -> f32 {
        crate::theme::scaled_size(size, self.text_scale)
    }

    pub(crate) fn text_scale_label(&self) -> String {
        format!("{}%", (self.text_scale * 100.0).round() as i32)
    }

    fn set_text_scale(&mut self, text_scale: f32) {
        self.text_scale = clamp_text_scale(text_scale);
        let preferences = GuiPreferences {
            text_scale: self.text_scale,
        };
        self.status = match preferences.save() {
            Ok(()) => format!("Text size {}", self.text_scale_label()),
            Err(error) => format!("Text size {} ({error})", self.text_scale_label()),
        };
    }
}

fn clamp_text_scale(text_scale: f32) -> f32 {
    if text_scale.is_finite() {
        text_scale.clamp(MIN_TEXT_SCALE, MAX_TEXT_SCALE)
    } else {
        DEFAULT_TEXT_SCALE
    }
}

fn parse_text_scale(content: &str) -> Option<f32> {
    content.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if key.trim() != "text_scale" {
            return None;
        }
        value.trim().parse::<f32>().ok()
    })
}

fn preferences_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(Path::new(&config_home).join("ai-suite").join("gui.toml"));
    }

    std::env::var_os("HOME").map(|home| {
        Path::new(&home)
            .join(".config")
            .join("ai-suite")
            .join("gui.toml")
    })
}
