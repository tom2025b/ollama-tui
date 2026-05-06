# ai-suite Architecture

A short tour of how the codebase is organized and how a prompt travels from a
keystroke to a streamed answer. Aimed at anyone (including future me) opening
the repo for the first time.

## High-level shape

ai-suite is a single Rust binary built around a small number of well-separated
modules:

```
main.rs          → bootstrap → cli → subcommands → tui (default)
                                                    ├── app (state)
                                                    ├── slash_commands
                                                    ├── ui (rendering)
                                                    └── model_task → providers
                                ↑                                         ↑
                                │                                         │
                              runtime ───────── routing ──── llm (types)
                              prompt_rules
```

- **`runtime`** loads config and paths once at startup.
- **`routing`** picks a model for each prompt.
- **`providers`** know how to actually talk to Ollama / Anthropic / OpenAI / xAI.
- **`subcommands::tui`** is the default user-facing app.
- **`prompt_rules`** wraps prompts with persistent global/project guidance.

## Startup path

1. `src/main.rs` — `#[tokio::main]` entry point. Calls `ai_suite::run()`.
2. `src/bootstrap.rs::run()` — loads the runtime, then dispatches the CLI.
3. `src/runtime/mod.rs::Runtime::load()` — reads env vars (API keys, model
   overrides like `OLLAMA_FAST_MODEL`) and resolves runtime paths
   (rules files, editor command).
4. `src/cli/mod.rs::dispatch()` — parses args via `clap`. With no subcommand it
   defaults to the TUI.
5. `src/subcommands/registry.rs` — small built-in registry that resolves
   `SubcommandId` (`Tui`, `Swarm`, `Food`) to its async runner.

This keeps `main` trivial and isolates configuration loading from any business
logic.

## Lifecycle of a user prompt

When the user types something in the TUI and presses Enter:

1. **Input** (`tui/input/`) translates the key event into an action on `App`.
2. **`App::submit_prompt`** (`tui/app/prompt.rs`):
   - If the input starts with `/`, it's parsed as a slash command and either
     handled locally or used to *stage* a real prompt for the model.
   - Otherwise the raw text becomes the prompt.
3. **Routing** — `route_prompt()` either honors a `/model`-pinned model or asks
   the `ModelRouter` to choose one. Returns a `RouteDecision { model, reason }`.
4. **Context assembly** (`tui/app/conversation/context.rs`) — pulls the last
   `MAX_CONTEXT_TURNS` user/assistant turns from history, skipping in-progress,
   failed, or user-excluded entries.
5. **Rule wrapping** (`prompt_rules/prompt.rs`) — `prompt_with_rules()` prepends
   active global + project rules.
6. **Dispatch** — a `PendingRequest { prompt, route, context }` is returned to
   the TUI loop, which calls `model_task::spawn_model_request`.
7. **Provider call** — `providers::execution::stream_model_request` matches on
   `Provider` and forwards to the right backend's `stream(...)` function.
8. **Streaming back to the UI** — provider tokens flow through an
   `mpsc::UnboundedSender<ModelEvent>` (`Token`, `Finished`, `Failed`). The
   render loop drains these on every tick and appends them to the active
   `ChatMessage`.

The TUI loop never blocks on HTTP. Drawing, key polling, and event drainage all
happen on the main task; the model request runs in a `tokio::spawn`.

## Routing

Located in `src/routing/`. Three small files:

- **`profile.rs`** — `PromptProfile::from_prompt()` extracts cheap features:
  is the prompt short, does it contain privacy/sensitive keywords, does it ask
  for current/public context, does it look like deep reasoning or code, etc.
- **`selection.rs`** — applies an ordered set of rules over the profile:
  1. **Privacy** → primary local Ollama (never leaves the box).
  2. **Current context** → prefers xAI, then OpenAI, Anthropic, Ollama.
  3. **Deep reasoning / code** → prefers Anthropic, then OpenAI, xAI, Ollama.
  4. **Simple/short** → fast local Ollama model.
  5. **Creative / general cloud** → prefers OpenAI, then Anthropic, xAI, Ollama.
  6. **Fallback** → general-purpose plan with a guaranteed Ollama endpoint.
- **`catalog.rs`** — builds the `Vec<LanguageModel>` from `RuntimeConfig`,
  marking cloud models disabled when their API key is missing.

Every route returns a human-readable `reason` string that's shown next to the
answer in the UI. Routing always falls back to local Ollama if no preferred
cloud backend is configured, so the app stays usable offline.

The `Router` trait makes the rule-based router swappable; a future learned
router can implement the same trait without touching the TUI.

## Provider layer

Each backend lives under `src/providers/<name>/` and exposes a single
`stream(model_name, context, prompt, on_token) -> Result<String>` function.

- `ollama` — local HTTP API, default fallback for everything.
- `anthropic` — Claude Messages API with its own SSE parser.
- `openai` and `xai` — both speak the OpenAI-compatible chat completions
  protocol, sharing parsing/streaming code in `providers/openai_compatible/`.

`providers/execution.rs` is the single switchboard:

```rust
match request.model.provider {
    Provider::Ollama    => ollama::stream(...).await,
    Provider::Anthropic => anthropic::stream(...).await,
    Provider::OpenAi    => openai::stream(...).await,
    Provider::Xai       => xai::stream(...).await,
}
```

Adding a new provider is: implement `stream`, add a `Provider` variant in
`llm.rs`, register a model in `routing/catalog.rs`, and add an arm here.

UTF-8 streaming is handled centrally in `llm.rs` via `append_utf8_chunk` /
`finish_utf8_stream` so partial multi-byte characters never reach the UI.

## TUI architecture

`subcommands/tui/run.rs` is the event loop. It:

1. Builds an `App` from `Runtime`.
2. Drains queued model events.
3. Calls `app.tick()` (spinner, status updates).
4. Draws a frame via `ratatui`.
5. Polls for a key event with a 50ms timeout.
6. Runs any external action queued by a slash command (e.g. open `$EDITOR`).

### State decomposition

`App` (in `app/state.rs`) deliberately *doesn't* hold one giant flat state.
Instead it groups related fields into small structs:

- **`SessionState`** — current input string, chat history, in-flight flags.
- **`RoutingState`** — `ModelRouter` + optional `pinned_model`.
- **`CommandState`** — slash-command registry, suggestion index, queued
  external actions, and a *staged prompt* slot for commands that produce a
  follow-up model request (`/fix`, `/explain`, `/review`).
- **`UiState`** — overlays, scroll offset, theme, layout mode, status line.
- **`RulesState`** — loaded global/project rules (see below).

This makes each method easy to read and avoids the "100-field god object"
pattern. The event loop only sees `App`; submodules touch only the substate
they need.

### Rendering

`tui/ui/` owns rendering. `chrome/`, `theme/`, `help.rs`, `history.rs`,
`model_picker.rs`, `palette.rs` etc. are pure functions of `&App` returning
ratatui widgets. State and presentation never mix.

## Slash command system

Files: `subcommands/tui/slash_commands/`.

- **`parser.rs`** — turns input that starts with `/` into a `ParsedCommand`.
- **`registry/definitions.rs`** — a `&[CommandDefinition]` table listing every
  command's display name, aliases, help text, and execute function.
- **`registry.rs`** — wraps the table with lookup, suggestion-prefix filtering,
  and help generation.
- **`handlers/`** — one file per command (`fix`, `review`, `explain`, `clear`,
  `rules`, `history`, `context_memory`, `code_block`, `ui_quality`, `session`,
  `backends`).

Two execution shapes:

- **Local commands** mutate `App` directly (e.g. `/clear`, `/help`, `/model`,
  `/rules`, `/context`). They may queue an `ExternalAction` (e.g. open the
  rules file in `$EDITOR`) that the run loop executes between frames.
- **Prompt-producing commands** (`/fix`, `/explain`, `/review`) call
  `commands.stage_prompt(...)`. `submit_prompt` drains the staged prompt and
  treats it as if the user had typed it, so it goes through routing, context,
  and rules like any other prompt.

Adding a command = add a handler function + one `CommandDefinition` row.

## Prompt rules and context management

Two independent mechanisms shape what a model actually sees:

### Rules (`src/prompt_rules/`)

Persistent guidance loaded from two markdown files at startup:

- A **global** rules file (resolved by `RuntimePaths::global_rules_path`).
- A **project** rules file in the nearest project root, if one exists.

`RulesState::prompt_with_rules(prompt)` prepends a short preamble plus each
active section, then the user's request. The wrapper explicitly tells the model
that the *current request wins* if it conflicts with a style preference, so
rules can't lock the model into ignoring the user.

`/rules` lets the user toggle, edit, or inspect them. When rules are active,
their summary is appended to the route reason in the UI so it's obvious they
applied.

### Conversation context

Context is bounded, not unlimited:

- `MAX_CONTEXT_TURNS` — how many past turns are sent to the model.
- `MAX_STORED_TURNS` — how many are kept in memory (older entries are dropped).

`conversation_context()` walks history newest-first, includes only completed,
non-failed, user-included turns, takes up to `MAX_CONTEXT_TURNS`, then reverses
to chronological order.

Users control context surgically:

- `/context` toggles whether the latest answer participates in future turns.
- `/clear` (and friends) drop older turns from the active context window.

Context selection lives in `tui/app/conversation/context.rs` and is the only
place that reasons about what gets sent.

## Key design decisions

- **Trait-based routing.** The `Router` trait isolates "decide which model"
  from "talk to that model", so the rule engine can be swapped without UI
  changes.
- **Provider-neutral request type.** `ModelRequest` is the single shape every
  backend receives. Provider modules are interchangeable behind a `match`.
- **Always-available local fallback.** The router *cannot* return a model with
  no backend configured; missing API keys silently degrade to Ollama with a
  clear reason string.
- **Substate over god-objects.** `App` composes small purpose-built structs,
  each owning one concern.
- **No blocking in the UI loop.** All model I/O is `tokio::spawn`ed and results
  flow back over an `mpsc` channel.
- **Slash commands as first-class citizens.** Some commands are pure UI;
  others *produce* prompts. Both share one parser, registry, and dispatch path.
- **Honest streaming.** UTF-8 framing is handled centrally so partial bytes
  from any provider never corrupt the chat view.
- **Rules are advisory, not coercive.** The wrapper instructs the model to
  defer to the user's current request when it conflicts with stored style
  preferences.
- **Bounded memory.** History is capped (`MAX_STORED_TURNS`) and the context
  window is capped separately (`MAX_CONTEXT_TURNS`), keeping long sessions
  cheap and predictable.

## Where to start reading

If you want to understand the codebase quickly, read these files in order:

1. `src/lib.rs` — module map.
2. `src/bootstrap.rs` and `src/cli/mod.rs` — startup path.
3. `src/subcommands/tui/run.rs` — the main loop.
4. `src/subcommands/tui/app/state.rs` — state shape.
5. `src/subcommands/tui/app/prompt.rs` — the lifecycle of a submission.
6. `src/routing/selection.rs` — how a model is chosen.
7. `src/providers/execution.rs` — how a model is actually called.
8. `src/prompt_rules/prompt.rs` — how rules are layered onto a prompt.
