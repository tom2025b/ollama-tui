# Design: ai-suite-gui — egui Desktop GUI

**Date:** 2026-05-07  
**Status:** Approved

---

## Overview

Add a native desktop GUI frontend for ai-suite using `eframe`/`egui`. The GUI lives in a new workspace crate (`ai-suite-gui`) and talks to the existing backend through a thin new public API added to the `ai-suite` library. The existing CLI (`ai-suite-cli`) is untouched.

---

## Architecture

```
workspace/
  ai-suite/          ← library (one new public function added)
  ai-suite-cli/      ← existing CLI binary (no changes)
  ai-suite-gui/      ← NEW egui desktop crate
```

### Integration approach

The `providers::execution::stream_model_request` function and routing internals are `pub(crate)`. Rather than exposing those directly or duplicating code, we add a single thin public wrapper in the library:

```rust
// ai_suite::stream_prompt (new public export)
pub async fn stream_prompt(
    prompt: String,
    context: Vec<ConversationTurn>,
    on_token: impl FnMut(String),
) -> Result<String>
```

This calls the existing `routing` → `providers::execution` pipeline unchanged. The GUI crate depends only on this public function and the already-public `ConversationTurn` type.

---

## Library Changes (`ai-suite`)

**New file:** `src/stream.rs`

```rust
pub async fn stream_prompt(
    prompt: String,
    context: Vec<ConversationTurn>,
    on_token: impl FnMut(String),
) -> anyhow::Result<(String, String)>  // (full_response_text, model_name)
```

Internals:
1. Load `Runtime` (reads env vars / config — same as CLI startup)
2. Call `routing::route_prompt(&prompt, &runtime)` → `RouteDecision`
3. Build a `ModelRequest` and call `providers::execution::stream_model_request`
4. Return `(full_response_text, model_name)` — the caller gets the model name so the UI can display it

**Return type refined:** Returns `(String, String)` — `(full_text, model_name)` — so the GUI top bar can show the routed model after each response.

**`src/lib.rs` additions:**
```rust
pub mod stream;
pub use stream::stream_prompt;
pub use llm::ConversationTurn;  // already public, ensure re-exported
```

---

## GUI Crate (`ai-suite-gui`)

### `Cargo.toml`

Dependencies:
- `ai-suite = { path = "../ai-suite" }`
- `eframe = "0.29"` (includes egui)
- `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`
- `anyhow = "1"`

### File structure

```
ai-suite-gui/
  Cargo.toml
  src/
    main.rs       ← tokio + eframe entry point
    app.rs        ← App struct, egui update() loop
    backend.rs    ← spawn_request(): calls stream_prompt, feeds channel
```

### `main.rs`

Builds a `tokio` runtime explicitly using `tokio::runtime::Builder::new_multi_thread().build()` and stores it in an `Arc` that is passed into `App`. eframe takes over the main thread so tokio cannot use `#[tokio::main]` — the runtime is kept alive for the full duration of the eframe window and dropped after `run_native` returns. `backend::spawn_request` receives a `tokio::runtime::Handle` (via `runtime.handle().clone()`) and calls `handle.spawn(...)` to fire off backend tasks.

### `app.rs` — App state

```rust
struct App {
    messages: Vec<ChatMessage>,   // full conversation history
    input: String,                // current text box content
    current_model: String,        // shown in top bar ("Ready" until first response)
    rx: Option<mpsc::UnboundedReceiver<BackendEvent>>,  // token stream
    streaming: bool,              // true while a response is in-flight
}

struct ChatMessage {
    role: Role,      // User | Assistant
    content: String,
    complete: bool,  // false = currently streaming
}

enum BackendEvent {
    Token(String),
    Done { full_text: String, model_name: String },
    Error(String),
}
```

### `backend.rs` — `spawn_request`

```rust
pub fn spawn_request(
    prompt: String,
    context: Vec<ConversationTurn>,
    tx: mpsc::UnboundedSender<BackendEvent>,
)
```

Spawns a `tokio::task` that calls `ai_suite::stream_prompt`, forwarding each token as `BackendEvent::Token`. On completion sends `BackendEvent::Done { full_text, model_name }`.

---

## UI Layout

```
┌─────────────────────────────────────────────────┐
│  ai-suite                  [claude-sonnet-4-6]  │  ← top bar, model name from last RouteDecision
├─────────────────────────────────────────────────┤
│                                                 │
│              You: tell me about rust lifetimes  │  ← user bubble, right-aligned
│                                                 │
│  Assistant: Lifetimes in Rust describe how      │  ← assistant bubble, left-aligned
│  long references are valid...▌                  │    streaming cursor while in-flight
│                                                 │
│  (ScrollArea, auto-scrolls to bottom)           │
├─────────────────────────────────────────────────┤
│  [ Type a message...                    ] [▶]   │  ← TextEdit + Send button
└─────────────────────────────────────────────────┘
```

### Visual design

- **Background:** `#1e1e2e` (dark, Catppuccin-inspired)
- **User bubbles:** indigo/blue tint, right-aligned
- **Assistant bubbles:** dark grey (`#313244`), left-aligned
- **Streaming cursor:** blinking `▌` appended to in-progress assistant text
- **Top bar:** app name left, current model name right (live-updated after each `Done` event)
- **Font:** egui default (Inter), 14px body
- **Send button:** disabled while streaming

### Keyboard

- `Enter` → send message (same as clicking Send)
- `Shift+Enter` → newline in input (multi-line support)

---

## Data Flow

```
User types + hits Enter
  → App pushes User message to messages[]
  → App calls backend::spawn_request(prompt, context, tx)
  → streaming = true, input cleared
  → App pushes empty Assistant message to messages[]

Each egui frame:
  → App drains rx channel
  → BackendEvent::Token(t) → append t to last message content
  → BackendEvent::Done { full_text, model_name }
      → finalize last message, set complete=true
      → update current_model, streaming = false
  → BackendEvent::Error(e) → show error in last message, streaming = false

egui::Context::request_repaint() called from backend task on each token
  to wake the render loop without busy-waiting
```

---

## Error Handling

- Network/API errors surface as an error bubble in the chat (same styling as assistant, red accent)
- No crash — the app stays open; the user can retry
- Missing API keys: error message says which key is missing (same as CLI behavior, from `Runtime::load()`)

---

## Workspace changes

`Cargo.toml` (root): add `"ai-suite-gui"` to `members`.

`.gitignore`: no changes needed (build artifacts already covered by `target/`).

---

## What is NOT in scope

- Slash commands (TUI feature only — the GUI sends raw prompts)
- History persistence between sessions (stateless for v1)
- Multiple conversations / tabs
- Settings UI (model selection, API key entry) — use env vars same as CLI
- System tray / notifications
