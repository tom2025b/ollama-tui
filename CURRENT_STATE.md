# Current State After Step 8

This document captures the actual project structure after completing migration
Step 8. It describes what exists now, not the full future architecture.

## Completed Migration Steps

1. Added `src/lib.rs` and `src/bootstrap.rs`.
2. Moved the TUI runner under `src/subcommands/tui/`.
3. Moved TUI internals under `src/subcommands/tui/`.
4. Moved provider code under `src/providers/`.
5. Renamed routing and prompt rule modules:
   - `router` -> `routing`
   - `rules` -> `prompt_rules`
6. Moved history under `src/storage/`.
7. Added top-level CLI parsing with `clap`.
8. Added basic subcommand contracts and registry.

Latest migration commit:

```text
08743cd refactor: step 8 - add subcommand contracts
```

## Cargo Layout

The package is still named `ollama-me`.

```toml
[package]
name = "ollama-me"

[lib]
name = "ai_suite"
path = "src/lib.rs"

[[bin]]
name = "ai-suite"
path = "src/main.rs"
```

The public binary target is now `ai-suite`.

## Runtime Behavior

Current supported command forms:

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

`swarm` and `food` are parsed and dispatched, but their real subcommand modules
have not been created yet. That is planned for Step 9.

## Current Folder Structure

```text
src/
  main.rs
  lib.rs
  bootstrap.rs

  cli/
    mod.rs
    args.rs
    dispatch.rs

  subcommands/
    mod.rs
    spec.rs
    registry.rs

    tui/
      mod.rs
      run.rs
      input.rs
      input/
        tests.rs
        tests/
      terminal.rs
      external.rs
      model_task.rs

      app/
      ui/
      slash_commands/

  llm.rs
  llm/
    model.rs
    provider.rs
    route.rs
    turn.rs

  providers/
    mod.rs
    anthropic/
    ollama/
    openai_compatible/
    openai.rs
    xai.rs

  routing/
    mod.rs
    catalog.rs
    profile.rs
    profile/
    selection.rs
    tests.rs
    tests/

  prompt_rules/
    mod.rs
    content.rs
    paths.rs
    prompt.rs
    report.rs
    state.rs
    state/
    storage.rs
    target.rs
    tests.rs

  storage/
    mod.rs
    history.rs
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
  - Top-level application runner.
  - Parses CLI args through `cli`.
  - Dispatches to the selected command.

### CLI

- `src/cli/args.rs`
  - Defines `Cli` and `CliCommand`.
  - Owns `clap` parsing.

- `src/cli/dispatch.rs`
  - Converts parsed CLI commands into execution.
  - Dispatch remains explicit for now.
  - Uses the subcommand registry as a consistency check.

- `src/cli/mod.rs`
  - Small module facade for CLI parsing and dispatch.

### Subcommands

- `src/subcommands/spec.rs`
  - Defines the initial simple `Subcommand` trait:

```rust
pub trait Subcommand {
    fn name(&self) -> &'static str;
}
```

- `src/subcommands/registry.rs`
  - Lists built-in command names:
    - `tui`
    - `swarm`
    - `food`
  - Provides `names()` and `contains()`.
  - Includes registry tests.

- `src/subcommands/tui/`
  - Owns all current TUI behavior.
  - Contains the app state, UI rendering, input handling, terminal setup,
    external actions, model task orchestration, and TUI slash commands.

### Core AI Modules

- `src/llm.rs` and `src/llm/`
  - Provider-neutral LLM types and routing-facing abstractions.

- `src/providers/`
  - Concrete provider implementations:
    - Anthropic
    - Ollama
    - OpenAI-compatible
    - OpenAI
    - xAI

- `src/routing/`
  - Model selection and routing rules.

- `src/prompt_rules/`
  - Prompt rule loading, storage, reporting, editing, and prompt injection.

- `src/storage/`
  - Shared persistence modules.
  - Currently contains conversation history.

## Not Created Yet

These folders are part of the target architecture but do not exist yet:

```text
src/runtime/
src/config/
src/tools/
src/extensions/
src/subcommands/swarm/
src/subcommands/food/
```

## Verification State

After Step 8:

```text
cargo fmt --check
cargo check
cargo test
```

All passed.

Last observed test result:

```text
97 passed; 0 failed; 4 ignored
```

Known warnings still exist from pre-existing TUI code:

- unused `register_commands` macro
- unused `mut` in `ui/palette.rs`
- unused `clear_conversation_command`
- unused `CommandSpec` type alias

## Working Tree Notes

Known uncommitted context files:

```text
AGENTS.md
MIGRATION_PLAN.md
SESSION_SUMMARY.md
CURRENT_STATE.md
```

These files are documentation/context only and are not part of the committed
Step 8 source migration.

## Next Planned Step

Step 9 should create real stub modules for `swarm` and `food`:

```text
src/subcommands/swarm/mod.rs
src/subcommands/swarm/run.rs
src/subcommands/food/mod.rs
src/subcommands/food/run.rs
```

Then CLI dispatch should call those modules instead of printing the placeholder
messages directly from `src/cli/dispatch.rs`.
