# Final Refactor Review Summary

This file is the review handoff for Claude. The highest-priority issues found
during the migration review have been addressed. No remaining high-priority
item from this list is intentionally left open.

## Completed Work

Addressed six review findings from the initial migration pass:

- Public/private boundary leakage.
- Oversized TUI theme module.
- Over-broad slash command execution context.
- Provider execution coupled to the TUI.
- Missing runtime/config architecture layer.
- Subcommand registry and CLI dispatch duplication.

## Code Review Pass (2026-05-06)

Addressed seven findings from the post-migration code review pass:

### 1. `is_local_message` boolean field (was: string sentinel)

- Added `is_local_message: bool` to `ChatMessage` and `HistoryEntry`.
- Local slash command results set it `true`; model turns set it `false`.
- Replaced all four `model_name != "ollama-me"` string comparisons with the
  boolean field in `conversation/context.rs`, `history_output.rs`, and
  `context_memory/report.rs`.

Review files:

- `src/subcommands/tui/app/types.rs`
- `src/subcommands/tui/app/conversation/local.rs`
- `src/subcommands/tui/app/conversation/context.rs`
- `src/subcommands/tui/app/prompt.rs`
- `src/subcommands/tui/slash_commands/handlers/session/context.rs`
- `src/subcommands/tui/app/context.rs`
- `src/subcommands/tui/slash_commands/handlers/history_output.rs`
- `src/subcommands/tui/slash_commands/handlers/context_memory/report.rs`

### 2. Dead match arm in `badge_fg`

- Collapsed `badge_fg` to a direct `Color::Black` return.
- The `"mono"` arm was identical to the wildcard arm and added noise.

Review files:

- `src/subcommands/tui/ui/theme/colors.rs`

### 3. `println!` in `storage/history`

- Removed `println!("Exported history to ...")` from `save_report`.
- The function returns the saved `PathBuf`; callers own all user-facing output.

Review files:

- `src/storage/history.rs`

### 4. App name in paths, labels, and templates

- Updated all filesystem paths from `ollama-me` to `ai-suite`:
  - Global config: `~/.config/ai-suite/rules.md`
  - Project rules: `<root>/.ai-suite/rules.md`
  - History export: `~/.local/share/ai-suite/history/`
- Updated TUI window title, compact header label, history report header,
  rules file template, and system prompt preamble.

Review files:

- `src/runtime/paths.rs`
- `src/prompt_rules/target.rs`
- `src/prompt_rules/storage.rs`
- `src/prompt_rules/prompt.rs`
- `src/subcommands/tui/slash_commands/handlers/history/report.rs`
- `src/subcommands/tui/ui/theme/blocks.rs`
- `src/subcommands/tui/ui/chrome/header.rs`

### 5. Editor env read moved into `RuntimePaths`

- Added `editor: OsString` field to `RuntimePaths`, resolved from
  `$VISUAL`, `$EDITOR`, or `vi` during `from_environment`.
- Added `App::editor_command()` delegating to `runtime.paths().editor()`.
- Removed the direct `std::env::var_os` calls from `external.rs`.

Review files:

- `src/runtime/paths.rs`
- `src/subcommands/tui/app/state.rs`
- `src/subcommands/tui/external.rs`

### 6. Provider submodule visibility

- Changed `pub mod` to `pub(crate) mod` for all five provider backends in
  `providers/mod.rs`.
- The `providers` module is private in `lib.rs`; the `pub` declarations were
  silently downgraded anyway and were inconsistent with `execution`'s
  existing `pub(crate)`.

Review files:

- `src/providers/mod.rs`

### 7. Inline `crate::...` paths in `prompt.rs`

- Replaced two long inline `crate::subcommands::tui::slash_commands::...`
  expressions with `use` imports at the top of the file.

Review files:

- `src/subcommands/tui/app/prompt.rs`

## Verification

Last full verification passed:

```text
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

Observed test result:

```text
112 passed; 0 failed; 4 ignored
```

## Review Notes For Claude

- `swarm` and `food` are intentionally stubbed top-level subcommands.
- Provider-neutral tools and public/private extension hooks are initial
  architecture surfaces only; no built-in tools or private extensions are
  registered in the public code.
- The public build should not expose private slash commands, private filesystem
  paths, or local pairing/config secrets.
- The `model_name` field on local messages still contains `"ollama-me"` as a
  human-readable label; the `is_local_message` boolean is now the canonical
  way to distinguish local command output from model turns.
