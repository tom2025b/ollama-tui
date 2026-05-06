# Session Summary

## Full Content of AGENTS.md

# ai-suite Core Philosophy

This project will become a clean, professional Rust application.

**Non-negotiable rules:**
- No god files. Ever.
- Every file must have a single, clear responsibility.
- We will refactor code repeatedly until it is excellent.
- Readability and maintainability beat clever code every time.
- Strong separation of concerns is mandatory.

# Repository Guidelines

## Project Structure & Module Organization
`ollama-tui` is a single Rust crate. Main code lives in `src/`, with behavior grouped by concern:
- `src/app/` for app state, routing, prompts, and app tests
- `src/command/` for command parsing, registry, and handlers
- `src/ollama/`, `src/openai_compatible/`, and `src/anthropic/` for provider clients and streaming
- `src/ui/` for the TUI chrome, layout, palette, and help screens
- `src/router/` and `src/rules/` for routing and prompt-rule logic

## Build, Test, and Development Commands
- `cargo run` starts the TUI locally.
- `cargo fmt --check` verifies formatting.
- `cargo clippy --all-targets -- -D warnings` runs lint checks across the crate.
- `cargo test` runs the full test suite.

The app expects a local Ollama server. Use `ollama pull llama3` and `ollama serve` before launching if needed.

## Coding Style & Naming Conventions
Follow standard Rust formatting and `rustfmt` output. Use `snake_case` for modules, files, and functions, and keep command names in `/verb` form such as `/clear` or `/models`. Prefer small, focused modules that match the existing directory structure.

## Testing Guidelines
Tests live alongside the code in `src/*/tests.rs` and nested `src/*/tests/` modules. Add coverage next to the feature you change, especially for command parsing, routing, and provider streaming. Live provider tests are ignored by default because they require local services or API keys.

## Commit & Pull Request Guidelines
Recent commits use short imperative prefixes like `feat:` and `fix:` followed by a focused summary. Keep PRs similarly scoped. Include a clear description, validation notes, and screenshots or short recordings for UI changes. Mention any required environment variables such as `OLLAMA_HOST` or provider API keys.

## Configuration Notes
Cloud backends stay disabled until the relevant API key is set. Common environment variables include `OLLAMA_HOST`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, and `XAI_API_KEY`.

## Core Philosophy Section

This section is the guiding standard for future work:

# ai-suite Core Philosophy

This project will become a clean, professional Rust application.

**Non-negotiable rules:**
- No god files. Ever.
- Every file must have a single, clear responsibility.
- We will refactor code repeatedly until it is excellent.
- Readability and maintainability beat clever code every time.
- Strong separation of concerns is mandatory.

## Project Direction: ai-suite Public and Private Versions

We are turning `ollama-tui` into `ai-suite`: a modular Rust application with multiple subcommands instead of a single-purpose TUI. The public version should provide the clean open-source foundation: shared architecture, local Ollama support, generic provider interfaces, stable CLI/TUI behavior, tests, and contributor standards.

The private version should layer on personal or proprietary capabilities without contaminating the public core. Private code may include sensitive prompts, API keys, custom workflows, experimental commands, private agent configurations, and integrations that should not ship publicly.

The architecture must keep these versions separate. Shared behavior belongs in reusable public modules. Private behavior should be isolated behind explicit modules, feature flags, configuration, or extension points. Nothing private should be hardcoded into the public path.

The long-term standard is a professional Rust codebase: small files, narrow responsibilities, clear module boundaries, strong tests, and repeated refactoring until the design is simple to understand and maintain.
