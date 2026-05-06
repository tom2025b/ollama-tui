# ai-suite Core Philosophy
# ai-suite Philosophy & Rules

This codebase is being deliberately evolved into a clean, professional, modular Rust application called `ai-suite`.

**Non-Negotiable Rules:**

- No god files. Ever. If a file is over 400 lines, it is too big.
- Every file must have a single, clear responsibility.
- We will refactor code repeatedly until it is excellent. "Good enough" is not acceptable.
- Readability and maintainability beat clever code every single time.
- Strong separation of concerns is mandatory.
- Public version must stay clean and professional.
- My private version can contain my personal chaos tools.

**Our Long-term Goal:**
Turn `ollama-tui` into the main `ai-suite` binary that supports multiple subcommands (`tui`, `swarm`, `food`, etc.) while remaining highly modular.

Any AI working in this codebase must respect these rules.

# Project Status

The codebase has been migrated to `ai-suite` and is currently centered on the TUI experience, with initial non-placeholder top-level command surfaces for `swarm` and `food`.

## Current State

- Cargo package name: `ai-suite`.
- Public binary target: `ai-suite`.
- Library crate: `ai_suite`.
- Implemented user-facing commands: `tui`, `swarm`, and `food`.
- Provider-neutral built-in tools are registered.
- The public extension pack is registered and contributes a public tool.

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
- `ai-suite swarm` prints a readiness report for configured models and registered public tools.
- `ai-suite food` prints a local starter meal plan and public tool count without sending food data anywhere.

The current codebase is organized around a modular runtime, subcommand registry, provider layer, routing layer, prompt rules, storage, provider-neutral tools, and public extension hooks.

## Completed Work

- Added `src/lib.rs` and `src/bootstrap.rs`.
- Moved the TUI runner and internals under `src/subcommands/tui/`.
- Moved provider code under `src/providers/`.
- Renamed `router` to `routing` and `rules` to `prompt_rules`.
- Moved history under `src/storage/`.
- Added top-level CLI parsing with `clap`.
- Added top-level modules for `swarm` and `food`.
- Added provider-neutral tool interfaces under `src/tools/`.
- Added public/private extension hooks under `src/extensions/`.
- Removed private slash commands and private machine-specific paths from the public TUI command surface.
- Split the oversized TUI theme module into focused files.
- Narrowed slash command handler context through capability traits.
- Moved provider execution out of the TUI and into the provider layer.
- Added explicit runtime/config/path architecture under `src/runtime/`.
- Made the subcommand registry the source of truth for top-level command execution.
- Added `.gitignore` coverage for local HomeKit/Homebridge JSON dumps.
- Collapsed dead match arm in `badge_fg`.
- Removed `println!` side effect from `storage/history::save_report`.
- Renamed all user-facing paths, labels, and templates to `ai-suite`.
- Moved editor command resolution (`$VISUAL`/`$EDITOR`/`vi`) into `RuntimePaths`.
- Changed provider submodule declarations to `pub(crate)` to match their actual crate-internal visibility.
- Replaced inline `crate::...` paths in `prompt.rs` with `use` imports.
- Replaced the old `model_name` string sentinels with a structural `is_local_message: bool` field on `ChatMessage` and `HistoryEntry`.
- Removed slash-command `CommandId` duplication; command definitions now resolve directly to registered executors.
- Replaced TUI routing-mode label parsing with explicit pinned-model state.
- Centralized model identity comparison in the TUI model picker state.
- Refactored routing fallback selection into named route plans with shared reason constants.
- Made privacy routing explicitly choose the primary local Ollama model.
- Added built-in provider-neutral tools: `utc_timestamp` and `build_info`.
- Added public extension tool: `public_profile`.
- Added shared subcommand capability loading for built-in tools and public extensions.
- Replaced `swarm` and `food` placeholder output with initial functional command output.
- Added regression coverage proving private slash commands `/claude`, `/codex`, and `/cost` are not exposed.

Verification has passed with:

```text
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

Observed test result:

```text
115 passed; 0 failed; 4 ignored
```

## Resolved Review Items

- `swarm` and `food` are no longer "not implemented yet" stubs.
- Provider-neutral tools are registered through the built-in tool registry.
- Public extensions are registered and contribute a public tool.
- TUI routing state no longer depends on brittle string parsing.
- Routing fallback logic is less duplicated and has explicit route plans.
- Private TUI commands remain absent from resolve, help, and suggestion surfaces.

## Future Expansion

- Expand `swarm` from readiness reporting into real orchestration.
- Expand `food` from local starter planning into a fuller food-planning workflow.
- Add more built-in provider-neutral tools as concrete TUI, swarm, and food workflows need them.
- Keep public/private boundaries enforced as private extension packs are introduced outside the public build.

## Canonical Paths And Layout

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

```text
src/
  main.rs
  lib.rs
  bootstrap.rs

  cli/
  runtime/
  subcommands/
  llm.rs
  llm/
  providers/
  routing/
  prompt_rules/
  storage/
  tools/
  extensions/
```

## Canonical Rules

- Public TUI slash commands do not include `/claude`, `/codex`, or `/cost`.
- Public code must not hard-code Tom-specific paths or private report-mail hooks.
- Rules editing uses `$VISUAL`, `$EDITOR`, then `vi`, resolved once at startup in `RuntimePaths`.
- Slash command handlers depend on focused capability traits instead of a broad app-shaped context.
- Provider execution is outside the TUI.
- Runtime-derived config and paths are created once and passed down.
- Top-level command execution goes through `subcommands::registry`.
- `is_local_message: bool` on `ChatMessage` and `HistoryEntry` is the canonical way to distinguish local command output from model turns.

## Final Status - May 6th

`ai-suite` is now a modular Rust app with:

- `ai-suite` / `ai-suite tui` as the main TUI experience.
- `ai-suite swarm` reporting model/tool readiness instead of being a stub.
- `ai-suite food` providing a local starter meal plan instead of being a stub.
- Routing cleaned up into explicit route plans and safer local privacy handling.
- TUI routing state no longer inferred from label strings.
- Built-in provider-neutral tools registered: `build_info`, `utc_timestamp`.
- Public extension hook registered with `public_profile`.
- Private slash commands remain excluded and covered by tests.
- `PROJECT_STATUS.md` updated to match the current state.

Verification is clean: `fmt`, `check`, `clippy -D warnings`, and `test` all pass with `115 passed; 0 failed; 4 ignored`.

## Major Simplification Pass - May 6th

- Removed the entire slash-command capability-trait layer and dispatcher/executor trampoline.
- Commands now execute directly against `App`.
- Removed 43 Rust source files by collapsing over-split modules.
- Significantly simplified architecture while keeping every file under 600 lines.
- Verification still clean: `113 passed; 0 failed; 4 ignored`.

## Provider Replacement and Persistent Memory - May 6th

- Removed Anthropic, OpenAI, OpenAI-compatible, and xAI REST API provider modules entirely.
- Replaced with `ClaudeCode` and `Codex` as terminal app routes: the TUI suspends and launches the CLI in the project root rather than streaming over an API.
- No API keys required. No cloud credentials stored or transmitted by the app itself.
- Added `MemoryStore` (`src/storage/memory.rs`): project-scoped persistent memory stored in `.ai-suite/memory.json`.
- `MemoryItem` is a proper sum type — `Turn | Note` — with no sentinel strings.
- Turn items inject as persistent conversation context (before session context).
- Note items inject as a prompt prefix (`[Project notes]` block), not as fake conversation exchanges.
- New `/pin <note>` command writes durable project notes.
- `/bookmark` and `/memory clear` now interact with both session and persistent memory.
- `Provider::ClaudeCode` and `Provider::Codex` arms in `execution.rs` use `unreachable!()` since `submit_prompt` intercepts terminal-app routes before streaming.
- Env vars renamed: `CLAUDE_CODE_MODEL` and `CODEX_MODEL` replace the misleading `ANTHROPIC_MODEL`/`OPENAI_MODEL`.
- `Runtime::for_tests()` uses `std::env::temp_dir()` instead of a hardcoded `/tmp`.
- Brand names removed from `COMPLEX_WORK_KEYWORDS` routing classifier.
- Verification clean: `fmt`, `clippy -D warnings`, `120 passed; 0 failed; 1 ignored`.
