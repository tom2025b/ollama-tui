# Error Audit Change Tracker

This file tracks the files changed during the centralized error-handling audit
and related prerequisite fixes in this session. It will be updated as more
modules are completed.

## Status

- Audit modules completed so far: `0`, `1`, `2`, `3`, `4`, `5`, `6`, `7`, `8`, `9`, `10`, `11`
- New module work is currently paused after Module 11
- Current changed-file inventory matches the working tree

## Current Changed Files And Notes

### Module 0: OpenAI-Compatible Streaming Error Test

- `ai-suite/src/providers/openai_compatible/client.rs`
- `ai-suite/src/providers/openai_compatible/tests.rs`

Notes:
Added a provider regression test for streaming connection failure propagation
and a test-only OpenAI-compatible client constructor so the test can target a
local endpoint without API keys.

### Module 1: Error Foundation

- `Cargo.lock`
- `ai-suite/Cargo.toml`
- `ai-suite/src/errors.rs`
- `ai-suite/src/lib.rs`

Notes:
Added `thiserror`, introduced the centralized root `Error` enum and
`Result<T>`, kept `friendly_error` compatible with both typed errors and
existing `anyhow` callers, and re-exported the new error surface from
`ai-suite/src/lib.rs`.

### Module 2: `llm.rs`

- `ai-suite/src/llm.rs`

Notes:
Removed `anyhow` from the UTF-8 stream assembly helpers, converted failures to
typed `crate::Error` variants, removed the production `expect`, and added
targeted unit tests for invalid UTF-8 and truncated stream endings.

### Module 3: `runtime/*`

- `ai-suite/src/runtime/config.rs`
- `ai-suite/src/runtime/environment.rs`
- `ai-suite/src/runtime/file_config.rs`
- `ai-suite/src/runtime/mod.rs`
- `ai-suite/src/runtime/paths.rs`

Notes:
Converted runtime path and config-loading failure boundaries to use the
centralized `Error` type internally while preserving non-fatal startup
fallbacks. Added warnings for `HOME` and `current_dir` fallback cases, trimmed
empty config/env values, and added regression coverage for malformed/unreadable
config files and runtime-path fallback behavior.

### Module 4: `prompt_rules/*`

- `ai-suite/src/prompt_rules/mod.rs`
- `ai-suite/src/prompt_rules/state.rs`
- `ai-suite/src/prompt_rules/storage.rs`
- `ai-suite/src/prompt_rules/tests.rs`

Notes:
Converted prompt-rules file I/O boundaries to use the centralized `Error` type
internally, preserved non-fatal unreadable-rules behavior by translating typed
errors into load warnings, and added regression coverage for warning
translation, edit-template creation, and typed prepare-edit failures.

### Module 5: `storage/*`

- `ai-suite/src/storage/history.rs`
- `ai-suite/src/storage/mod.rs`

Notes:
Converted history report persistence to use the centralized `Error` type
internally, preserved existing save/export behavior while attaching operation
and path context to failures, removed the remaining storage `expect`, and
added regression coverage for a blocked-parent path error.

### Module 6: `tools/*` and `extensions/*`

- `ai-suite/src/extensions/mod.rs`
- `ai-suite/src/tools/builtins/mod.rs`
- `ai-suite/src/tools/mod.rs`
- `ai-suite/src/tools/registry.rs`
- `ai-suite/src/tools/spec.rs`

Notes:
Removed `anyhow` from the tool and extension interfaces, converted tool
execution and registry construction onto the centralized `Error`/`Result`
types, replaced the custom tool-registry error with `Error::Tool`, cleaned up
module-local `expect` calls, and tightened module documentation.

### Module 7: `routing/*`

- `ai-suite/src/routing/explain.rs`
- `ai-suite/src/routing/mod.rs`
- `ai-suite/src/routing/selection.rs`
- `ai-suite/src/routing/tests.rs`
- `ai-suite/src/routing/tests/fallbacks.rs`
- `ai-suite/src/routing/tests/invariants.rs`
- `ai-suite/src/routing/tests/privacy.rs`
- `ai-suite/src/stream.rs`
- `ai-suite/src/subcommands/tui/app/prompt.rs`
- `ai-suite/src/subcommands/tui/app/state.rs`
- `ai-suite/src/subcommands/tui/slash_commands/handlers/route.rs`

Notes:
Converted routing decisions and explanations onto the centralized
`Error`/`Result` types, removed the last production `expect` from `routing/*`,
preserved existing provider/fallback behavior, added regression coverage for a
missing primary Ollama fallback model, and translated typed routing failures at
the direct router call sites.

### Module 8: `providers/*`

- `ai-suite/src/providers/anthropic/http.rs`
- `ai-suite/src/providers/anthropic/mod.rs`
- `ai-suite/src/providers/anthropic/stream_parser.rs`
- `ai-suite/src/providers/execution.rs`
- `ai-suite/src/providers/ollama/client.rs`
- `ai-suite/src/providers/ollama/http.rs`
- `ai-suite/src/providers/ollama/mod.rs`
- `ai-suite/src/providers/ollama/models.rs`
- `ai-suite/src/providers/ollama/stream.rs`
- `ai-suite/src/providers/ollama/tests/model_tests.rs`
- `ai-suite/src/providers/openai.rs`
- `ai-suite/src/providers/openai_compatible/client.rs`
- `ai-suite/src/providers/openai_compatible/http.rs`
- `ai-suite/src/providers/openai_compatible/stream.rs`
- `ai-suite/src/providers/openai_compatible/tests.rs`
- `ai-suite/src/providers/xai.rs`

Notes:
Converted the shared provider dispatcher and all production provider internals
onto the centralized `Error`/`Result` types, replaced the remaining
provider-layer `anyhow`/`bail!` paths with stable typed variants, and updated
provider regression tests to assert typed missing-model and streaming errors.

### Module 9: stream/bootstrap/CLI/subcommand surfaces

- `ai-suite/src/bootstrap.rs`
- `ai-suite/src/cli/mod.rs`
- `ai-suite/src/stream.rs`
- `ai-suite/src/subcommands/capabilities.rs`
- `ai-suite/src/subcommands/food/mod.rs`
- `ai-suite/src/subcommands/registry.rs`
- `ai-suite/src/subcommands/spec.rs`
- `ai-suite/src/subcommands/swarm/mod.rs`
- `ai-suite/src/subcommands/tui/external.rs`
- `ai-suite/src/subcommands/tui/run.rs`
- `ai-suite/src/subcommands/tui/terminal.rs`

Notes:
Removed the remaining production `anyhow` usage from the public stream helper
and command-execution layer, converted explicit model-selection failures to
typed validation errors, and mapped terminal I/O boundaries onto centralized
error variants without changing user-facing startup or slash-command behavior.

### Module 10: `stream.rs` follow-up hardening

- `ai-suite/src/stream.rs`

Notes:
Extracted pure helper functions for explicit model selection and route
formatting so the typed validation and routing-error behavior can be exercised
directly in unit tests, and switched public route-error formatting to use
`friendly_error` for consistency with the rest of the surface.

### Module 11: bootstrap + CLI core follow-up hardening

- `ai-suite/src/bootstrap.rs`
- `ai-suite/src/cli/mod.rs`

Notes:
Added small pure helpers around startup warning rendering, fatal error
rendering, default-command selection, and clap parsing so the already-migrated
bootstrap/CLI boundary now has focused regression coverage without changing
runtime behavior.

## Verification Completed So Far

- `cargo test -p ai-suite test_stream_error_propagates -- --nocapture`
- `cargo test -p ai-suite openai_compatible -- --nocapture`
- `cargo check -p ai-suite --lib`
- `cargo test -p ai-suite utf8_ --lib`
- `cargo test -p ai-suite runtime:: --lib`
- `cargo test -p ai-suite prompt_rules:: --lib`
- `cargo test -p ai-suite storage:: --lib`
- `cargo test -p ai-suite tools:: --lib`
- `cargo test -p ai-suite extensions:: --lib`
- `cargo test -p ai-suite routing:: --lib`
- `cargo check -p ai-suite`
- `cargo test -p ai-suite providers:: --lib`
- `cargo test -p ai-suite subcommands:: --lib`
- `cargo test -p ai-suite stream:: --lib`
- `cargo test -p ai-suite cli:: --lib`
- `cargo test -p ai-suite bootstrap:: --lib`
- `cargo fmt --all`
