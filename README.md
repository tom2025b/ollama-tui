# ai-suite

`ai-suite` is a command-deck for terminal AI work.

It routes plain prompts across local Ollama, Claude Code, and Codex without API-key setup inside the app. Simple and private work stays local. Deep coding work launches Claude Code in the project root. Current-context, creative, and general terminal work launches Codex. Project memory, prompt rules, slash commands, and exports are built into the TUI so the tool keeps its bearings across sessions.

The design rule is simple: the terminal is the interface, the project is the context, and the router should make the obvious backend choice before you have to think about it.

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

## Optional Model Names

Claude Code and Codex are launched as terminal apps, not API backends. You can override the labels shown in the router:

```sh
export CLAUDE_CODE_MODEL=claude-sonnet-4-20250514
export CODEX_MODEL=codex
export OLLAMA_FAST_MODEL=llama3:latest
```

## Routing Behavior

- Sensitive prompts are forced to Ollama.
- Short prompts use the fast local Ollama model.
- Coding, complex, and deep reasoning prompts launch Claude Code.
- Current, public-context, general, and creative prompts prefer Codex.
- If you pin Claude Code or Codex with `/model`, new prompts launch that terminal app directly.
- Local Ollama remains the fallback route for private or simple prompts.

The router shows the chosen model and route reason in the conversation.

## Project Memory

`ai-suite` keeps long-term memory per project under:

```text
<project-root>/.ai-suite/memory.json
```

Use `/bookmark` to persist the latest completed model turn. Use `/pin <note>` to persist a durable project note such as architectural rules, repository quirks, or user preferences. Future prompts receive a bounded mix of project memory and recent session context, so remembered facts survive restarts without turning every transcript into permanent baggage.

Project rules live beside memory at:

```text
<project-root>/.ai-suite/rules.md
```

Together, memory and rules form the local constitution for the project.

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
/pin <note>
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
- `/backend` lists route target readiness.
- `/rules` opens the current project rules in `$VISUAL`, then `$EDITOR`, then `vi` when a project is detected, otherwise global rules. Global rules live at `~/.config/ai-suite/rules.md`; project rules live at `<project-root>/.ai-suite/rules.md`.
- `/rules off`, `/rules on`, and `/rules toggle` control whether rules are applied to new prompts.
- `/history` shows the current session transcript. `/history save` writes a text file under `~/.local/share/ai-suite/history/`.
- `/summary` shows a compact session summary.
- `/export` saves a formatted history report to a chosen path or the default history directory.
- `/context`, `/tokens`, `/bookmark`, `/memory`, and `/pin` inspect and control which turns and project notes are carried forward into future prompts.
- `/pin <note>` writes a project memory item that survives future TUI sessions.
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

Live provider smoke tests are ignored by default because they require local services or paid provider credentials.
