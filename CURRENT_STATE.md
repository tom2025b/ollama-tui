# Current State After Completing All 12 Migration Steps

This document captures the actual project structure after completing all 12
steps of the migration plan. It describes what exists now, what remains stubbed,
and the verification state after the final cleanup pass.

## Summary

The `ollama-tui` codebase has been migrated into a modular Rust library and
binary architecture for `ai-suite`.

The package is still named `ollama-me`, but the public binary target is now
`ai-suite` and the library crate is `ai_suite`.

The current top-level command surface is:

```text
ai-suite
ai-suite tui
ai-suite swarm
ai-suite food
```

The TUI remains the only implemented user-facing experience. `swarm` and
`food` are real top-level subcommand modules, but they are intentionally stubbed
and currently print not-implemented messages.

Shared AI concerns have been split away from the TUI:

- Provider-neutral LLM types live under `src/llm.rs` and `src/llm/`.
- Concrete model providers live under `src/providers/`.
- Routing lives under `src/routing/`.
- Prompt rule logic lives under `src/prompt_rules/`.
- Conversation history lives under `src/storage/`.
- Runtime config and path decisions live under `src/runtime/`.
- Provider-neutral tool interfaces live under `src/tools/`.
- Public/private extension hooks live under `src/extensions/`.

The final cleanup pass is complete:

- `cargo fmt --check` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- `cargo check` passes.
- `cargo test` passes with `112 passed; 0 failed; 4 ignored`.
- All Rust source files are under the 400-line project limit.

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
9. Added real stub modules for `swarm` and `food`.
10. Added initial provider-neutral tool architecture interfaces.
11. Added initial extension architecture interfaces.
12. Completed the final cleanup pass.

Latest committed migration step before the current Step 12 work:

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

`swarm` and `food` are parsed and dispatched through real stub subcommand
modules. Their feature implementations have not been created yet.

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

  runtime/
    mod.rs
    environment.rs
    config.rs
    paths.rs

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
        theme/
          mod.rs
          colors.rs
          styles.rs
          blocks.rs
      slash_commands/

    swarm/
      mod.rs
      run.rs

    food/
      mod.rs
      run.rs

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
    execution.rs
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

  tools/
    mod.rs
    spec.rs
    registry.rs
    execution.rs
    builtins/
      mod.rs

  extensions/
    mod.rs
    api.rs
    public.rs
    registry.rs
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
  - Builds the process `Runtime`.
  - Parses CLI args through `cli`.
  - Dispatches to the selected command.

### Runtime

- `src/runtime/mod.rs`
  - Defines the `Runtime` value passed from bootstrap into command execution.
  - Groups process-derived config and paths.

- `src/runtime/environment.rs`
  - Defines `RuntimeEnvironment`.
  - Provides the process-backed implementation for env and current-dir reads.

- `src/runtime/config.rs`
  - Owns model catalog configuration derived from env.
  - Centralizes `OLLAMA_FAST_MODEL`, cloud model override, and cloud API-key
    presence checks used by routing.

- `src/runtime/paths.rs`
  - Owns home/current/project path decisions.
  - Provides global/project rules paths, history export paths, and user-path
    expansion.

### CLI

- `src/cli/args.rs`
  - Defines `Cli`.
  - Owns `clap` parsing.

- `src/cli/dispatch.rs`
  - Thin adapter from parsed CLI args to the subcommand registry.
  - Receives the process `Runtime`.
  - Uses the registry for default command selection and execution.

- `src/cli/mod.rs`
  - Small module facade for CLI parsing and dispatch.

### Subcommands

- `src/subcommands/spec.rs`
  - Defines the top-level `SubcommandId` parsed by the CLI.
  - Defines focused runner type aliases for subcommand execution.

- `src/subcommands/registry.rs`
  - Owns built-in command names, default command selection, and runner
    entrypoints:
    - `tui`
    - `swarm`
    - `food`
  - Includes registry tests.

- `src/subcommands/tui/`
  - Owns all current TUI behavior.
  - Contains the app state, UI rendering, input handling, terminal setup,
    external actions, model task orchestration, and TUI slash commands.

- `src/subcommands/swarm/`
  - Stub top-level `swarm` command module.
  - Currently prints `swarm is not implemented yet`.

- `src/subcommands/food/`
  - Stub top-level `food` command module.
  - Currently prints `food is not implemented yet`.

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
  - Builds its catalog from `RuntimeConfig` instead of reading env vars.

- `src/prompt_rules/`
  - Prompt rule loading, storage, reporting, editing, and prompt injection.
  - Loads and edits rules from `RuntimePaths`.

- `src/storage/`
  - Shared persistence modules.
  - Currently contains conversation history.
  - History export path handling is driven by `RuntimePaths`.

### Tools

- `src/tools/spec.rs`
  - Defines provider-neutral tool metadata through `ToolDefinition`.
  - Defines the initial `Tool` trait.

- `src/tools/execution.rs`
  - Defines `ToolInvocation`, `ToolInput`, and `ToolOutput`.
  - Keeps execution request and response types independent of any provider.

- `src/tools/registry.rs`
  - Defines `ToolRegistry`.
  - Supports registering tools, resolving tools by name, listing definitions,
    and rejecting duplicate names.

- `src/tools/builtins/`
  - Provides the built-in tool registration entrypoint.
  - Currently registers no tools.

### Extensions

- `src/extensions/api.rs`
  - Defines the initial `ExtensionPack` trait.
  - Allows extension packs to register provider-neutral tools.

- `src/extensions/public.rs`
  - Defines the public extension pack for the clean public build.
  - Currently registers no tools.

- `src/extensions/registry.rs`
  - Defines `ExtensionRegistry`.
  - Provides a `public()` registry containing only the public extension pack.
  - Applies registered extension packs to a `ToolRegistry`.

## Verification State

After the subcommand registry cleanup:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo check
cargo test
find src -name '*.rs' -exec wc -l {} +
```

All passed.

Last observed test result:

```text
112 passed; 0 failed; 4 ignored
```

Strict clippy now passes with warnings treated as errors.

Largest observed Rust source file:

```text
169 src/subcommands/tui/app/context.rs
```

All Rust source files are under the 400-line project limit.

## Working Tree Notes

Known uncommitted context files:

```text
AGENTS.md
MIGRATION_PLAN.md
SESSION_SUMMARY.md
CURRENT_STATE.md
```

Current migration cleanup and follow-up refactor source changes are also
uncommitted.

## Next Planned Step

No remaining high-priority review item from `FIXES.md` is pending.
