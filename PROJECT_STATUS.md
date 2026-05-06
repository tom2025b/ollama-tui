# ai-suite Project Status

Philosophy and non-negotiable rules live in `AGENTS.md`. This document is a current-state snapshot.

# Current State

`ai-suite` is a modular Rust app centered on the TUI experience.

- Cargo package: `ai-suite`. Public binary: `ai-suite`. Library crate: `ai_suite`.
- Implemented user-facing commands: `tui`, `swarm`.
- Provider-neutral built-in tools registered: `build_info`, `utc_timestamp`.

Supported command forms:

```text
ai-suite          # defaults to TUI
ai-suite tui      # interactive terminal UI
ai-suite swarm    # readiness report for configured models and tools
ai-suite swarm "<task>"   # route a task and stream it through Ollama
```

# Architecture

```text
src/
  main.rs
  lib.rs
  bootstrap.rs       # one-line entry point
  cli/               # top-level CLI parsing and dispatch
  runtime/           # config + paths, derived once at startup
  subcommands/       # tui, swarm, shared capabilities
  llm.rs             # core types: Provider, LanguageModel, ConversationTurn
  providers/         # ollama (streaming) + provider-neutral execution
  routing/           # prompt → model selection (route plans, fallbacks)
  prompt_rules/      # editable prompt rules
  storage/           # history + persistent project memory
  tools/             # provider-neutral tool registry + built-ins
```

# Canonical Rules

- Public TUI slash commands do not include `/claude`, `/codex`, or `/cost`. Enforced by registry tests.
- Public code must not hard-code Tom-specific paths or private report-mail hooks.
- Rules editing uses `$VISUAL`, `$EDITOR`, then `vi`, resolved once at startup in `RuntimePaths`.
- Slash command handlers run directly against `App`; there is no capability-trait dispatcher.
- Provider execution is outside the TUI.
- Runtime-derived config and paths are created once and passed down.
- Top-level command parsing and dispatch live in `cli::dispatch`.
- `is_local_message: bool` on `ChatMessage` is the canonical way to distinguish local command output from model turns.
- Streaming only supports Ollama. Terminal-app routes (`ClaudeCode`, `Codex`) suspend the TUI and hand off to the external CLI before reaching the streaming layer.

# Notable Design Choices

- `ClaudeCode` and `Codex` are terminal app routes: the TUI suspends and launches the CLI in the project root rather than streaming over an API. No API keys, no cloud credentials stored or transmitted.
- Persistent project memory lives in `.ai-suite/memory.json`. `MemoryItem` is a proper `Turn | Note` sum type — no sentinel strings. Notes inject as a prompt prefix; turns inject as conversation context.
- Routing is structured as named route plans with shared reason constants. Privacy routing explicitly chooses the primary local Ollama model.
- `ChatMessage` owns its message constructors, streaming state transitions, model-turn predicates, and context-turn projection.
- Conversation context deduplicates exact turns across persistent project memory and current session history.

# Verification

```text
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

Last run: `119 passed; 0 failed; 1 ignored`.

# Final Cleanup Pass

- Removed the entire `food` subcommand. It was a hardcoded `println!` stub presented as a feature.
- Deleted the `extensions/` module (`ExtensionRegistry`, `ExtensionPack` trait, `PublicProfileTool`). One trait implementation, no test mocks, placeholder tool — exactly the abstract-for-the-future ban from `AGENTS.md`. Public capabilities now go directly through `ToolRegistry::with_builtins()`.
- Replaced `unreachable!()` in `providers/execution.rs` for terminal-app providers with a `debug_assert!` and a doc comment naming the upstream invariant. The streaming path now reads as a single Ollama call instead of pretending to be a multi-provider dispatch.
- Added a header comment in `slash_commands/registry/definitions.rs` naming the public/private command rule so future edits don't drift.
- Trimmed this document from a 253-line changelog to a current-state snapshot. Historical pass-by-pass notes live in `git log`.
