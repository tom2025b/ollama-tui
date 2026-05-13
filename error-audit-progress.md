# Error Audit Progress

Snapshot date: 2026-05-12

## Current Scope

The centralized error-handling audit is currently complete through Module 13.
No work has started yet on Modules 14 and later.
The audit is paused here due to token limit.

Completed:

- Module 0: OpenAI-compatible streaming error propagation regression test
- Module 1: Error foundation in `ai-suite`
- Module 2: `ai-suite/src/llm.rs`
- Module 3: `ai-suite/src/runtime/*`
- Module 4: `ai-suite/src/prompt_rules/*`
- Module 5: `ai-suite/src/storage/*`
- Module 6: `ai-suite/src/tools/*` and `ai-suite/src/extensions/*`
- Module 7: `ai-suite/src/routing/*`
- Module 8: `ai-suite/src/providers/*`
- Module 9: stream/bootstrap/CLI/subcommand execution surfaces
- Module 10: `ai-suite/src/stream.rs` follow-up hardening
- Module 11: bootstrap + CLI core follow-up hardening
- Module 12: `ai-suite-cli/*`
- Module 13: `ai-suite-gui/*` entry surfaces

Paused:

- Module 14 and beyond

## What Changed

### Module 0

- Added a regression test that verifies OpenAI-compatible streaming failures
  propagate instead of being swallowed.
- Added a test-only OpenAI-compatible client constructor so tests can target a
  local endpoint without API keys or external services.

Changed files:

- `ai-suite/src/providers/openai_compatible/client.rs`
- `ai-suite/src/providers/openai_compatible/tests.rs`

### Module 1

- Added `thiserror` to the core crate dependency set.
- Introduced the centralized root `ai_suite::Error` enum and
  `ai_suite::Result<T>`.
- Kept `friendly_error` compatible with both typed errors and the still-present
  `anyhow` call sites, which allows incremental migration.
- Re-exported the new error surface from the library root.

Changed files:

- `Cargo.lock`
- `ai-suite/Cargo.toml`
- `ai-suite/src/errors.rs`
- `ai-suite/src/lib.rs`

### Module 2

- Removed `anyhow` from `llm.rs`.
- Changed UTF-8 stream helper failures to return typed `crate::Error`.
- Replaced the production `expect` on UTF-8 prefix decoding with an invariant
  error.
- Added focused unit tests for invalid UTF-8 and truncated stream endings.

Changed files:

- `ai-suite/src/llm.rs`

### Module 3

- Hardened runtime environment probing so `current_dir()` failures become typed
  errors internally and non-fatal startup warnings externally.
- Hardened path resolution so missing `HOME` and current-directory fallback
  scenarios are surfaced as warnings instead of being silently ignored.
- Refactored config-file loading to convert I/O and TOML parse failures into
  centralized `Error` variants before translating them into the runtime's
  existing warning mechanism.
- Trimmed whitespace-only env/config string values so blank overrides do not
  silently win.
- Added regression tests for runtime-path fallback behavior and unreadable
  config-file handling.

Changed files:

- `ai-suite/src/runtime/config.rs`
- `ai-suite/src/runtime/environment.rs`
- `ai-suite/src/runtime/file_config.rs`
- `ai-suite/src/runtime/mod.rs`
- `ai-suite/src/runtime/paths.rs`

### Module 4

- Converted prompt-rules file loading and edit preparation to use the
  centralized `Error`/`Result` internally.
- Preserved the existing non-fatal behavior for unreadable rules files by
  translating typed load failures into `RulesState` warnings.
- Added regression coverage for unreadable rules files, default-template
  creation during `/rules` edit prep, and typed error propagation when parent
  directory creation fails.

Changed files:

- `ai-suite/src/prompt_rules/mod.rs`
- `ai-suite/src/prompt_rules/state.rs`
- `ai-suite/src/prompt_rules/storage.rs`
- `ai-suite/src/prompt_rules/tests.rs`

### Module 5

- Converted history-report persistence to use the centralized `Error`/`Result`
  internally instead of returning raw `std::io::Result`.
- Preserved the existing `/history save` and `/summary export` behavior while
  attaching stable operation and path context to storage failures.
- Removed the remaining storage `expect` call and added regression coverage
  for a blocked-parent path failure.

Changed files:

- `ai-suite/src/storage/history.rs`
- `ai-suite/src/storage/mod.rs`

### Module 6

- Removed `anyhow` from the tool and extension registration/execution surfaces.
- Converted `Tool::execute`, built-in tool registration, extension-pack
  registration, and tool-registry construction onto the centralized
  `Error`/`Result` types.
- Removed the custom tool-registry error type in favor of `Error::Tool`,
  cleaned up the remaining module-local `expect` calls, and tightened module
  docs.

Changed files:

- `ai-suite/src/extensions/mod.rs`
- `ai-suite/src/tools/builtins/mod.rs`
- `ai-suite/src/tools/mod.rs`
- `ai-suite/src/tools/registry.rs`
- `ai-suite/src/tools/spec.rs`

### Module 7

- Converted routing decisions and route explanations onto the centralized
  `Error`/`Result` types instead of relying on a production `expect`.
- Preserved the existing provider-selection and local-privacy fallback rules
  while surfacing a typed routing error if the router is ever built without its
  required primary Ollama fallback model.
- Added regression coverage for the missing-primary-Ollama invariant in both
  `route()` and `explain()`, and translated typed routing failures at the small
  number of user-facing call sites that invoke the router directly.

Changed files:

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

### Module 8

- Converted the shared provider dispatcher plus Anthropic, Ollama, OpenAI, xAI,
  and OpenAI-compatible client internals onto the centralized `Error`/`Result`
  types.
- Replaced remaining provider-layer `anyhow`/`bail!` paths with stable
  `MissingApiKey`, `HttpClientBuild`, `HttpRequest`, `HttpStatus`,
  `ProviderResponse`, `Streaming`, and `Json` variants as appropriate.
- Tightened provider regression coverage so missing-model and truncated-stream
  failures now assert typed error variants instead of plain strings.

Changed files:

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

### Module 9

- Removed the remaining production `anyhow` use from the public stream helpers,
  bootstrap/CLI dispatch, subcommand registry/capability surfaces, and TUI
  terminal/external-action runners.
- Converted explicit model-selection failures in `stream.rs` to typed
  validation errors and mapped terminal I/O boundaries onto centralized
  pathless I/O errors.
- Kept user-facing startup and slash-command behavior unchanged while letting
  typed routing/provider errors propagate cleanly through the command layer.

Changed files:

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

### Module 10

- Extracted pure `stream.rs` helpers for explicit model selection and route
  explanation formatting so the typed validation and routing-error behavior is
  unit-testable without live runtime/provider execution.
- Switched `/route`-style public formatting to pass typed routing failures
  through `friendly_error`, keeping public output consistent with the rest of
  the error surface.
- Added focused regression coverage for unknown model IDs, disabled model
  selection, successful route formatting, and typed routing-failure rendering.

Changed files:

- `ai-suite/src/stream.rs`

### Module 11

- Extracted tiny pure helpers in `bootstrap.rs` and `cli/mod.rs` so startup
  warning rendering, fatal error rendering, default-command selection, and clap
  parsing are directly testable.
- Preserved command-dispatch behavior while adding focused regression coverage
  for omitted subcommands and user-facing bootstrap formatting.

Changed files:

- `ai-suite/src/bootstrap.rs`
- `ai-suite/src/cli/mod.rs`

### Module 12

- Converted the CLI wrapper crate and binary entrypoints from `anyhow` onto
  `ai_suite::Result`.
- Mapped Tokio runtime-construction failure to centralized
  `ai_suite::Error::Terminal`.

Changed files:

- `ai-suite-cli/src/bin/ai.rs`
- `ai-suite-cli/src/lib.rs`
- `ai-suite-cli/src/main.rs`

### Module 13

- Converted the GUI wrapper crate and binary entrypoints from `anyhow` onto
  `ai_suite::Result`.
- Mapped GUI Tokio runtime-construction and `eframe` launch failures to
  centralized `ai_suite::Error::Terminal`.

Changed files:

- `ai-suite-gui/src/bin/ai-gui.rs`
- `ai-suite-gui/src/lib.rs`
- `ai-suite-gui/src/main.rs`

## Verification Run So Far

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
- `cargo check -p ai-suite-cli -p ai-suite-gui`
- `cargo fmt --all`

All of the above completed successfully.

## Current Working-Tree File Inventory

- `Cargo.lock`
- `ai-suite/Cargo.toml`
- `ai-suite/src/errors.rs`
- `ai-suite/src/lib.rs`
- `ai-suite/src/llm.rs`
- `ai-suite/src/providers/openai_compatible/client.rs`
- `ai-suite/src/providers/openai_compatible/tests.rs`
- `ai-suite/src/runtime/config.rs`
- `ai-suite/src/runtime/environment.rs`
- `ai-suite/src/runtime/file_config.rs`
- `ai-suite/src/runtime/mod.rs`
- `ai-suite/src/runtime/paths.rs`
- `ai-suite/src/prompt_rules/mod.rs`
- `ai-suite/src/prompt_rules/state.rs`
- `ai-suite/src/prompt_rules/storage.rs`
- `ai-suite/src/prompt_rules/tests.rs`
- `ai-suite/src/storage/history.rs`
- `ai-suite/src/storage/mod.rs`
- `ai-suite/src/extensions/mod.rs`
- `ai-suite/src/tools/builtins/mod.rs`
- `ai-suite/src/tools/mod.rs`
- `ai-suite/src/tools/registry.rs`
- `ai-suite/src/tools/spec.rs`
- `ai-suite/src/routing/explain.rs`
- `ai-suite/src/routing/mod.rs`
- `ai-suite/src/routing/selection.rs`
- `ai-suite/src/routing/tests.rs`
- `ai-suite/src/routing/tests/fallbacks.rs`
- `ai-suite/src/routing/tests/invariants.rs`
- `ai-suite/src/routing/tests/privacy.rs`
- `ai-suite/src/stream.rs`
- `ai-suite/src/bootstrap.rs`
- `ai-suite/src/cli/mod.rs`
- `ai-suite-cli/src/bin/ai.rs`
- `ai-suite-cli/src/lib.rs`
- `ai-suite-cli/src/main.rs`
- `ai-suite-gui/src/bin/ai-gui.rs`
- `ai-suite-gui/src/lib.rs`
- `ai-suite-gui/src/main.rs`
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
- `ai-suite/src/subcommands/capabilities.rs`
- `ai-suite/src/subcommands/food/mod.rs`
- `ai-suite/src/subcommands/registry.rs`
- `ai-suite/src/subcommands/spec.rs`
- `ai-suite/src/subcommands/swarm/mod.rs`
- `ai-suite/src/subcommands/tui/external.rs`
- `ai-suite/src/subcommands/tui/run.rs`
- `ai-suite/src/subcommands/tui/terminal.rs`
- `ai-suite/src/subcommands/tui/app/prompt.rs`
- `ai-suite/src/subcommands/tui/app/state.rs`
- `ai-suite/src/subcommands/tui/slash_commands/handlers/route.rs`
- `error-addons-.md`
- `error-audit-progress.md`

## Next Planned Modules

The next module in the approved rollout remains:

- Module 14+: remaining workspace crates and follow-up cleanup, if needed

## Notes

- The runtime startup behavior remains intentionally non-fatal for malformed
  config files; those cases now route through typed internal errors and emerge
  as user-facing warnings.
- The core `ai-suite/src` production surfaces no longer rely on `anyhow`;
  `errors.rs` still intentionally understands `anyhow::Error` chains so older
  callers and tests remain friendly while the wider workspace catches up.
- Audit paused due to token limit after Modules 12-13.
