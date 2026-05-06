# ai-suite

`ai-suite` is a terminal chat app that routes prompts between local Ollama and optional cloud LLM backends. It streams responses token by token, keeps a bounded conversation context, and forces sensitive prompts to stay local.

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

Supported command forms:

```sh
cargo run
cargo run -- tui
cargo run -- swarm
cargo run -- food
```

`swarm` currently reports model/tool readiness. `food` prints a local starter meal plan without sending private food data anywhere.

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
/model
/backend
/rules [global|project|show|off|on|toggle]
/history [show|save [path]]
/summary
/export [path]
/context
/tokens
/bookmark
/memory
/explain
/fix
/review
/theme
/resize
/help
/quit
```

- `/clear` clears the visible conversation. It is blocked while a model is streaming.
- `/model` opens an interactive picker. Use Up/Down to navigate, Enter to pin a model (every new prompt skips the router and goes to that model), and Esc to cancel. Pick `Auto` to hand routing back to the router.
- `/backend` lists backend readiness.
- `/rules` opens the current project rules in `$VISUAL`, then `$EDITOR`, then `vi` when a project is detected, otherwise global rules. Global rules live at `~/.config/ai-suite/rules.md`; project rules live at `<project-root>/.ai-suite/rules.md`.
- `/rules off`, `/rules on`, and `/rules toggle` control whether rules are applied to new prompts.
- `/history` shows the current session transcript. `/history save` writes a text file under `~/.local/share/ai-suite/history/`.
- `/summary` shows a compact session summary.
- `/export` saves a formatted history report to a chosen path or the default history directory.
- `/context`, `/tokens`, `/bookmark`, and `/memory` inspect and control which turns are carried forward into future prompts.
- `/explain`, `/fix`, and `/review` stage follow-up prompts based on the most recent code or assistant output.
- `/theme` and `/resize` adjust the TUI presentation.
- `/help` opens the help overlay. `/quit` and `/exit` leave the app.

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
