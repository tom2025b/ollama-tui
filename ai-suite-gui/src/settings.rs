use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use ai_suite::{Error, Result, friendly_error};

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
        let text_scale = load_text_scale()
            .ok()
            .flatten()
            .unwrap_or(DEFAULT_TEXT_SCALE);

        Self {
            text_scale: clamp_text_scale(text_scale),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = preferences_path()?;
        let parent = path
            .parent()
            .ok_or_else(|| Error::invariant("GUI preferences path has no parent directory"))?;
        fs::create_dir_all(parent)
            .map_err(|error| Error::io("creating GUI config directory", parent, error))?;
        fs::write(
            path.as_path(),
            format!("text_scale = {:.2}\n", self.text_scale),
        )
        .map_err(|error| Error::io("writing GUI preferences", path, error))
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
            Err(error) => format!(
                "Text size {} ({})",
                self.text_scale_label(),
                friendly_error(&error)
            ),
        };
    }
}

fn load_text_scale() -> Result<Option<f32>> {
    let path = preferences_path()?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io("reading GUI preferences", &path, error)),
    };

    let value = parse_text_scale(&content).ok_or_else(|| {
        Error::configuration(format!(
            "GUI preferences at {} contain an invalid `text_scale` value",
            path.display()
        ))
    })?;

    Ok(Some(value))
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

fn preferences_path() -> Result<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(Path::new(&config_home).join("ai-suite").join("gui.toml"));
    }

    std::env::var_os("HOME")
        .map(|home| {
            Path::new(&home)
                .join(".config")
                .join("ai-suite")
                .join("gui.toml")
        })
        .ok_or_else(|| {
            Error::configuration("could not resolve GUI config path from HOME or XDG_CONFIG_HOME")
        })
}
