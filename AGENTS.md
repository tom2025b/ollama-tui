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
