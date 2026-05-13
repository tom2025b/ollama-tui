use std::{env, ffi::OsString};

use ai_suite::{Error, Result};

pub fn run_from_env() -> Result<()> {
    let args = env::args_os().collect::<Vec<_>>();

    if should_launch_gui(&args) {
        return ai_suite_gui::run();
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| Error::terminal(format!("failed to build CLI tokio runtime: {error}")))?;
    rt.block_on(ai_suite::run())
}

fn should_launch_gui(args: &[OsString]) -> bool {
    if explicit_gui_subcommand(args) {
        return true;
    }

    args.len() == 1 && graphical_display_available()
}

fn explicit_gui_subcommand(args: &[OsString]) -> bool {
    args.get(1)
        .and_then(|arg| arg.to_str())
        .map(|arg| matches!(arg, "gui" | "desktop"))
        .unwrap_or(false)
}

fn graphical_display_available() -> bool {
    if env::var_os("AI_SUITE_FORCE_TUI").is_some() {
        return false;
    }

    if env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some() {
        return true;
    }

    cfg!(target_os = "windows") || cfg!(target_os = "macos")
}
