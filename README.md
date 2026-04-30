# ollama-me

`ollama-me` is a terminal chat app that routes prompts between local Ollama and optional cloud LLM backends. It streams responses token by token, keeps a bounded conversation context, and forces sensitive prompts to stay local.

## Requirements

- Rust toolchain with Cargo
- Ollama running locally for the default setup
- A local Llama 3 model:

```sh
ollama pull llama3
```

Start Ollama if it is not already running:

```sh
ollama serve
```

## Run

```sh
cargo run
```

By default, the app expects Ollama at `http://localhost:11434`. To use a different Ollama host:

```sh
export OLLAMA_HOST=http://127.0.0.1:11434
cargo run
```

## Optional API Keys

Cloud backends are disabled until their API key environment variables are present.

```sh
export ANTHROPIC_API_KEY=your_anthropic_key
export OPENAI_API_KEY=your_openai_key
export XAI_API_KEY=your_xai_key
```

You can also override model names:

```sh
export ANTHROPIC_MODEL=claude-sonnet-4-20250514
export OPENAI_MODEL=gpt-4o
export XAI_MODEL=grok-4.20-reasoning
export OLLAMA_FAST_MODEL=llama3:latest
```

## Routing Behavior

- Sensitive prompts are forced to Ollama.
- Short prompts use the fast local Ollama model.
- Coding and deep reasoning prompts prefer Anthropic when configured.
- Current or public-context prompts prefer xAI when configured.
- General and creative prompts prefer OpenAI when configured.
- If a preferred cloud backend is unavailable, routing falls back to local Ollama.

The router shows the chosen model and route reason in the conversation.

## Commands

Type these into the prompt box and press Enter:

```text
/clear
/models
/backends
/rules [global|project|show|off|on|toggle]
/history [show|save [path]|email [subject]]
```

- `/clear` clears the visible conversation. It is blocked while a model is streaming.
- `/models` opens an interactive picker. Use Up/Down to navigate, Enter to pin a model (every new prompt skips the router and goes to that model), and Esc to cancel. Pick `Auto` to hand routing back to the router.
- `/backends` lists backend readiness.
- `/rules` opens the current project rules in `nano` when a project is detected, otherwise global rules. Global rules live at `~/.config/ollama-me/rules.md`; project rules live at `<project-root>/.ollama-me/rules.md`.
- `/rules off`, `/rules on`, and `/rules toggle` control whether rules are applied to new prompts.
- `/history` shows the current session transcript. `/history save` writes a text file under `~/.local/share/ollama-me/history/`, and `/history email` pipes it through `~/bin/send-report`.

Press `?` with an empty prompt to open the help screen. Press `q`, `Esc`, `?`, or `Ctrl-C` to close it.

## Keyboard

- `Enter`: send prompt or command
- `Ctrl-U`: clear current input
- `Esc`: quit from the main screen
- `Ctrl-C`: quit from the main screen, close help from the help screen
- `?`: open help when the prompt is empty
- `q`: close help when the help screen is open
- `Up/Down`: scroll chat history one line at a time
- `PageUp/PageDown`: scroll chat history by half a screen
- `Home/End`: jump to top/bottom of chat history

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Live provider smoke tests are ignored by default because they require local services or paid API keys.
