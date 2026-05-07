# ai-suite Agent Guide

This codebase is being deliberately evolved into a clean, professional,
modular Rust application called `ai-suite`.

## Non-Negotiable Rules

- No god files. Ever. If a file is over 400 lines, it is too big.
- Every file must have a single, clear responsibility.
- Refactor repeatedly until the design is excellent. "Good enough" is not the bar.
- Readability and maintainability beat clever code every time.
- Strong separation of concerns is mandatory.
- The public version must stay clean and professional.
- Private chaos tools belong only in a private build or private extension area.

## Long-Term Goal

Turn `ollama-tui` into the main `ai-suite` binary with multiple subcommands
(`tui`, `swarm`, `food`, etc.) while keeping the codebase highly modular.

## Current Workspace Shape

This repository is a Rust workspace:

- `ai-suite/` is the core library crate (`ai_suite`).
- `ai-suite-cli/` builds the public `ai-suite` binary.
- `ai-suite-gui/` builds the optional egui desktop binary.
- `docs/` holds design notes, specs, and plans.

When older docs refer to root-level `src/`, map that to `ai-suite/src/` unless
the docs have been updated to the workspace layout.

Important core areas in `ai-suite/src/`:

- `bootstrap.rs`, `cli/`, and `subcommands/` own startup and command dispatch.
- `subcommands/tui/` owns the terminal UI experience.
- `runtime/` owns config, environment, paths, and rules-file locations.
- `routing/` chooses models and preserves local privacy fallback behavior.
- `providers/` owns backend-specific model calls.
- `prompt_rules/` owns prompt wrapping with global/project guidance.
- `tools/` and `extensions/` own public tool surfaces.
- `storage/` owns history and persistence.

## Environment Rules

- Do not create or commit `.env` files in this repository.
- Do not put API keys, hostnames with credentials, personal paths, or private
  report hooks in tracked files.
- Cloud backends are enabled only through process environment variables:
  `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, and `XAI_API_KEY`.
- Model overrides also come from environment variables:
  `ANTHROPIC_MODEL`, `OPENAI_MODEL`, `XAI_MODEL`, and `OLLAMA_FAST_MODEL`.
- Ollama defaults to `http://localhost:11434`; override with `OLLAMA_HOST`.
- Runtime config belongs in `~/.config/ai-suite/config.toml`, not in repo-local
  secret files.
- User-wide rules live at `~/.config/ai-suite/rules.md`.
- Project rules, when needed, live at `.ai-suite/rules.md` and must not contain
  secrets.

## Standard Commands

Run from the workspace root:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the CLI:

```sh
cargo run -p ai-suite-cli --
cargo run -p ai-suite-cli -- tui
cargo run -p ai-suite-cli -- swarm
cargo run -p ai-suite-cli -- food
```

Run the GUI:

```sh
cargo run -p ai-suite-gui
```

Live provider smoke tests are ignored by default because they require local
services or paid API keys.

## Agent Roles

Every agent should pick the role that matches the task, then stay inside that
responsibility unless the user asks for broader work.

- **Architect:** protect module boundaries, public/private separation, and the
  400-line file limit. Prefer small cohesive modules over convenience piles.
- **Implementer:** make scoped changes that follow existing patterns. Keep
  edits narrow, readable, and covered by focused tests when behavior changes.
- **Reviewer:** lead with bugs, regressions, missing tests, privacy leaks, and
  maintainability risks. Cite exact files and lines.
- **Verification:** run the smallest meaningful command first, then broaden to
  the standard command set when risk or scope justifies it.
- **Documentation:** update docs only when behavior, commands, layout, or
  public contracts changed. Keep docs factual and current.

## Working Discipline

- Read nearby code before editing.
- Preserve unrelated user changes in the worktree.
- Avoid broad rewrites unless they directly serve the requested change.
- Keep public command surfaces free of private slash commands such as
  `/claude`, `/codex`, and `/cost`.
- Provider execution stays outside the TUI.
- Runtime-derived config and paths are created once and passed down.
- Top-level command execution goes through `subcommands::registry`.
- `is_local_message: bool` on `ChatMessage` and `HistoryEntry` is the canonical
  way to distinguish local command output from model turns.

Any AI working in this codebase must respect these rules.
