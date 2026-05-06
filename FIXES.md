# Fixes Completed This Session

## Scope Completed

Addressed the six highest-priority review findings:

- Public/private boundary leakage.
- Oversized TUI theme module.
- Over-broad slash command execution context.
- Provider execution coupled to the TUI.
- Missing runtime/config architecture layer.
- Subcommand registry and CLI dispatch duplication.

The public TUI command surface no longer exposes private or machine-specific
commands and helpers, the TUI theme code is now split by responsibility, and
slash command handlers now depend on focused capabilities instead of one large
app-shaped context trait. Concrete provider execution is now owned by the
provider layer instead of the TUI, and process-derived config/path state now
flows through an explicit runtime layer. Top-level subcommand execution now
flows through one registry-owned runner table instead of a second match in the
CLI dispatch layer.

## Public Boundary Changes

- Removed public slash commands:
  - `/claude`
  - `/codex`
  - `/cost`

- Deleted public command handlers for:
  - Claude Code launcher
  - Codex CLI launcher

- Removed the hard-coded private cost tracker path:
  - `/home/tom/projects/claude-cost-tracker`

- Removed the hard-coded public cost tracker dispatch from:
  - `src/subcommands/tui/external.rs`

- Removed Tom-specific history email export through `send-report` from:
  - `src/storage/history.rs`
  - `src/subcommands/tui/slash_commands/handlers/history.rs`

- Changed `/history` public behavior to support only:
  - `show`
  - `save [path]`

- Changed rules editing from hard-coded `nano` to a portable editor lookup:
  - `$VISUAL`
  - `$EDITOR`
  - fallback to `vi`

- Updated command registry tests to match the cleaned public command list.

- Removed the obsolete `/cost` app test.

## Theme Split Changes

- Replaced the deleted flat theme module with focused files:
  - `src/subcommands/tui/ui/theme/colors.rs`
  - `src/subcommands/tui/ui/theme/styles.rs`
  - `src/subcommands/tui/ui/theme/blocks.rs`
  - `src/subcommands/tui/ui/theme/mod.rs`

- Kept `theme::...` as the internal UI facade so existing renderers do not need
  to know how theme internals are organized.

- Moved palette color lookup, semantic style constructors, and `Block`
  constructors into separate modules.

- Restored `src/main.rs` to the documented thin `ai_suite::run()` entrypoint
  after finding an unrelated broken stub during verification.

## Command Context Changes

- Split the old all-purpose `CommandContext` surface into focused capability
  traits:
  - `CommandOutput`
  - `ModelActivity`
  - `ConversationControl`
  - `ModelPicker`
  - `ModelCatalog`
  - `RulesContext`
  - `HistoryView`
  - `HistoryExport`
  - `ContextMemory`
  - `SettingsContext`
  - `HelpOverlay`
  - `AppLifecycle`
  - `PromptStaging`

- Changed `App` to implement those capability traits through separate impl
  blocks in `src/subcommands/tui/app/context.rs`.

- Updated slash command handlers to use generic bounds for only the capability
  groups they need.

- Kept the composite `CommandContext` only as the registry/dispatcher dynamic
  execution boundary, with thin adapter functions bridging registry entries to
  the narrowed handlers.

## Provider Execution Changes

- Added `src/providers/execution.rs`.

- Introduced provider-neutral `ModelRequest` as the handoff into concrete model
  backends.

- Moved the `Provider::{Ollama, Anthropic, OpenAi, Xai}` execution match out of
  `src/subcommands/tui/model_task.rs` and into `providers::execution`.

- Kept `src/subcommands/tui/model_task.rs` responsible only for spawning the
  async task and translating provider execution results into TUI `ModelEvent`s.

- Added a focused unit test for the provider execution request metadata.

## Runtime/Config Changes

- Added a focused runtime module:
  - `src/runtime/mod.rs`
  - `src/runtime/environment.rs`
  - `src/runtime/config.rs`
  - `src/runtime/paths.rs`

- Centralized process environment reads behind `RuntimeEnvironment` and
  `ProcessEnvironment`.

- Added `RuntimeConfig` for model catalog configuration:
  - `OLLAMA_FAST_MODEL`
  - cloud provider model overrides
  - cloud provider API-key presence for routing availability

- Added `RuntimePaths` for path decisions:
  - home directory fallback
  - startup current directory
  - project-root detection
  - global rules path
  - project rules path
  - default history export directory
  - user-path expansion

- Changed bootstrap and CLI dispatch to create and pass a single `Runtime`.

- Changed TUI app construction to receive `Runtime`, then build routing and
  rules state from runtime config/paths.

- Changed routing to consume `ModelRuntimeConfig` instead of reading env vars or
  calling provider configuration helpers.

- Changed prompt rules to load and edit using `RuntimePaths`, then deleted the
  old prompt-rule-specific path/env module.

- Changed history export to use `RuntimePaths` and added `HistoryExport` as the
  narrow slash-command capability for saving reports.

- Added focused tests for runtime path expansion and history export path
  handling.

## Subcommand Registry Changes

- Changed `src/subcommands/registry.rs` into the source of truth for built-in
  subcommand names, default command selection, and execution entrypoints.

- Removed the duplicate `CliCommand` execution match from `src/cli/dispatch.rs`.

- Kept `src/cli/dispatch.rs` as a thin adapter that chooses the default command
  through the registry and calls `subcommands::registry::run`.

- Moved the top-level subcommand identity into `src/subcommands/spec.rs` so the
  registry does not depend back on the CLI parser module.

- Replaced the old name-only subcommand trait with focused runner type aliases
  in `src/subcommands/spec.rs`.

## Verification

All checks passed after the changes:

```text
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

Test result:

```text
112 passed; 0 failed; 4 ignored
```

## Current State For Next Agent

The public/private boundary fix, theme split, `CommandContext` narrowing, and
provider execution move are complete and verified. The runtime/config
architecture layer is also complete and verified. The subcommand registry now
owns top-level command execution, so the highest-priority review findings in
this file are complete.
