use crate::command::parser::ParsedCommand;

use super::session::{CommandContext, Setting, SettingEdit};

const USAGE: &str = "Usage: /voice [on|off|speed <0.5-2.0>|mode <auto|dictation|command>|status]";

pub fn handle_voice_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    let mut args = command.args().iter().map(String::as_str);
    let Some(action) = args.next().map(str::to_ascii_lowercase) else {
        show_voice(context, command.raw());
        return;
    };

    match action.as_str() {
        "show" | "status" => show_voice(context, command.raw()),
        "on" => apply_voice_setting(context, command.raw(), SettingEdit::VoiceEnabled(true)),
        "off" => apply_voice_setting(context, command.raw(), SettingEdit::VoiceEnabled(false)),
        "speed" => match args.next() {
            Some(speed) => {
                apply_voice_setting(context, command.raw(), SettingEdit::VoiceSpeed(speed))
            }
            None => show_voice(context, command.raw()),
        },
        "mode" => match args.next() {
            Some(mode) => apply_voice_setting(context, command.raw(), SettingEdit::VoiceMode(mode)),
            None => show_voice(context, command.raw()),
        },
        _ => show_usage(context, command.raw()),
    }
}

fn show_voice(context: &mut dyn CommandContext, input: &str) {
    let report = context.setting_report(Setting::Voice);
    finish(context, input, report);
}

fn apply_voice_setting(context: &mut dyn CommandContext, input: &str, setting: SettingEdit<'_>) {
    let result = context.set_setting(setting);
    apply_voice_result(context, input, result);
}

fn show_usage(context: &mut dyn CommandContext, input: &str) {
    context.append_local_message(input, USAGE.to_string());
    context.set_status("Unknown voice command.".to_string());
}

fn apply_voice_result(
    context: &mut dyn CommandContext,
    input: &str,
    result: Result<String, String>,
) {
    match result {
        Ok(message) => finish(context, input, message),
        Err(error) => {
            context.append_local_message(input, error);
            context.set_status("Invalid voice setting.".to_string());
        }
    }
}

fn finish(context: &mut dyn CommandContext, input: &str, message: String) {
    context.append_local_message(input, message.clone());
    context.set_status(message);
}
