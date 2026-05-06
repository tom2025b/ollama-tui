# Current State For Final Review

This document is the current architecture handoff for Claude. It reflects the
migrated `ollama-tui` codebase after the cleanup, follow-up refactor passes,
and the final code review fix pass on 2026-05-06.

## Summary

The codebase has been moved to a modular Rust application called `ai-suite`.

- Cargo package name: `ai-suite`.
- Public binary target: `ai-suite`.
- Library crate: `ai_suite`.
- Current implemented user-facing experience: the TUI.
- Current stub top-level commands: `swarm` and `food`.

Supported command forms:

```text
ai-suite
ai-suite tui
ai-suite swarm
ai-suite food
```

Behavior:

- `ai-suite` defaults to the TUI.
- `ai-suite tui` runs the TUI.
- `ai-suite swarm` prints `swarm is not implemented yet`.
- `ai-suite food` prints `food is not implemented yet`.

## Completed Work

Migration and cleanup completed across all passes:

1. Added `src/lib.rs` and `src/bootstrap.rs`.
2. Moved the TUI runner and internals under `src/subcommands/tui/`.
3. Moved provider code under `src/providers/`.
4. Renamed `router` to `routing` and `rules` to `prompt_rules`.
5. Moved history under `src/storage/`.
6. Added top-level CLI parsing with `clap`.
7. Added real stub modules for `swarm` and `food`.
8. Added provider-neutral tool interfaces under `src/tools/`.
9. Added public/private extension hooks under `src/extensions/`.
10. Removed private slash commands and private machine-specific paths from the
    public TUI command surface.
11. Split the oversized TUI theme module into focused files.
12. Narrowed slash command handler context through capability traits.
13. Moved provider execution out of the TUI and into the provider layer.
14. Added explicit runtime/config/path architecture under `src/runtime/`.
15. Made the subcommand registry the source of truth for top-level command
    execution.
16. Added `.gitignore` coverage for local HomeKit/Homebridge JSON dumps.
17. Collapsed dead match arm in `badge_fg`.
18. Removed `println!` side effect from `storage/history::save_report`.
19. Renamed all user-facing paths, labels, and templates to `ai-suite`.
20. Moved editor command resolution (`$VISUAL`/`$EDITOR`/`vi`) into
    `RuntimePaths` to close the last direct env-var read outside `Runtime`.
21. Changed provider submodule declarations to `pub(crate)` to match their
    actual crate-internal visibility.
22. Replaced inline `crate::...` paths in `prompt.rs` with `use` imports.
23. Replaced the old `model_name` string sentinels with a structural
    `is_local_message: bool` field on `ChatMessage` and `HistoryEntry`.

## Cargo Layout

```toml
[package]
name = "ai-suite"

[lib]
name = "ai_suite"
path = "src/lib.rs"

[[bin]]
name = "ai-suite"
path = "src/main.rs"
```

## Current Source Layout

```text
src/
  main.rs
  lib.rs
  bootstrap.rs

  cli/
    args.rs
    dispatch.rs
    mod.rs

  runtime/
    config.rs
    environment.rs
    mod.rs
    paths.rs

  subcommands/
    spec.rs
    registry.rs
    tui/
    swarm/
    food/

  llm.rs
  llm/

  providers/
    anthropic/
    ollama/
    openai_compatible/
    execution.rs
    openai.rs
    xai.rs

  routing/
  prompt_rules/
  storage/
  tools/
  extensions/
```

## Module Responsibilities

### Entrypoint

- `src/main.rs`
  - Thin binary entrypoint.
  - Calls `ai_suite::run().await`.

- `src/lib.rs`
  - Library crate root.
  - Owns top-level module declarations.
  - Re-exports `bootstrap::run`.

- `src/bootstrap.rs`
  - Builds the process `Runtime`.
  - Parses CLI args through `cli`.
  - Dispatches to the selected command.

### CLI And Subcommands

- `src/cli/args.rs`
  - Defines `Cli`.
  - Owns `clap` parsing.
  - Parses top-level subcommands into `SubcommandId`.

- `src/cli/dispatch.rs`
  - Thin adapter from parsed CLI args to the subcommand registry.
  - Receives the process `Runtime`.
  - Uses the registry for default command selection and execution.

- `src/subcommands/spec.rs`
  - Defines `SubcommandId`.
  - Defines focused runner type aliases for subcommand execution.

- `src/subcommands/registry.rs`
  - Owns built-in command names, default command selection, and runner
    entrypoints for `tui`, `swarm`, and `food`.

- `src/subcommands/tui/`
  - Owns all current TUI behavior.
  - Contains app state, UI rendering, input handling, terminal setup, external
    actions, model task orchestration, and slash commands.

- `src/subcommands/swarm/` and `src/subcommands/food/`
  - Real top-level modules with intentionally stubbed `run` functions.

### Runtime

- `src/runtime/mod.rs`
  - Defines the `Runtime` value passed from bootstrap into command execution.
  - Groups process-derived config and paths.

- `src/runtime/environment.rs`
  - Defines `RuntimeEnvironment`.
  - Provides the process-backed implementation for env and current-dir reads.

- `src/runtime/config.rs`
  - Owns model catalog config derived from env.
  - Centralizes `OLLAMA_FAST_MODEL`, cloud model overrides, and cloud API-key
    presence checks used by routing.

- `src/runtime/paths.rs`
  - Owns home/current/project path decisions.
  - Provides global/project rules paths, history export paths, user-path
    expansion, and the resolved editor command (`$VISUAL`/`$EDITOR`/`vi`).

### Core AI Modules

- `src/llm.rs` and `src/llm/`
  - Provider-neutral LLM types and routing-facing abstractions.

- `src/providers/`
  - Concrete provider implementations for Anthropic, Ollama, OpenAI-compatible,
    OpenAI, and xAI. All submodules are `pub(crate)`.
  - `src/providers/execution.rs` owns provider dispatch for model requests.

- `src/routing/`
  - Model selection and routing rules.
  - Builds its catalog from `RuntimeConfig` instead of reading env vars.

- `src/prompt_rules/`
  - Prompt rule loading, storage, reporting, editing, and prompt injection.
  - Loads and edits rules from `RuntimePaths`.

- `src/storage/`
  - Shared persistence modules.
  - Currently contains conversation history export.
  - History export path handling is driven by `RuntimePaths`.

### Tools And Extensions

- `src/tools/`
  - Defines provider-neutral tool metadata, invocation, output, registry, and
    built-in registration entrypoint.
  - Built-in registration currently adds no tools.

- `src/extensions/`
  - Defines extension pack APIs and the public extension registry.
  - The public registry contains only the public extension pack.
  - The public extension pack currently registers no tools.

## Important Boundary Decisions

- Public TUI slash commands no longer include `/claude`, `/codex`, or `/cost`.
- Public code no longer hard-codes Tom-specific paths or private report-mail
  hooks.
- Rules editing uses `$VISUAL`, `$EDITOR`, then `vi` — resolved once at
  startup in `RuntimePaths`, not read ad hoc at edit time.
- Slash command handlers depend on focused capability traits instead of a broad
  app-shaped context.
- Provider execution is outside the TUI.
- Runtime-derived config and paths are created once and passed down.
- Top-level command execution goes through `subcommands::registry`.
- `is_local_message: bool` on `ChatMessage` and `HistoryEntry` is the
  canonical way to distinguish local command output from model turns. Do not
  use `model_name` string comparisons for this purpose.

## Verification State

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

Largest Rust source file: 169 lines. All files are under the 400-line limit.

## Final Closeout Status

- The cleanup and refactor phase is complete.
- The public README now matches the current `ai-suite` app name, command
  surface, rules/history paths, and editor behavior.
- The repository target state is a clean `main` branch with no local drift from
  `origin/main`.

## Working Tree Notes

No uncommitted documentation/context files are expected after the final
closeout commit.

The ignored local file `0E-7B-77-69-E6-E8.json` is a HomeKit/Homebridge dump
with pairing data. It is intentionally ignored by `.gitignore` and should not
be reviewed or committed.

## Suggested Review Focus

- All issues from the 2026-05-06 code review pass have been addressed.
- The codebase is in a clean, stable state for the next feature phase.
- README and handoff documentation are now aligned with the current public
  behavior.
- Next logical work: implement real `swarm` and `food` subcommand behavior,
  or register built-in tools and the first private extension pack.
