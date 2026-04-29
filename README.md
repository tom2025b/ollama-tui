# ollama-me

Multi-backend LLM chat client with a Rich terminal UI, powered by local Ollama/Llama3.

## What It Does

- **Rich TUI** — styled chat interface with streaming markdown output
- **Ollama/Llama3 backend** — fully working local inference via Ollama's REST API
- **Router pattern** — swap backends by changing one string; stubs ready for Claude, GPT-4o, Grok, and small local models
- **Multi-turn conversations** — maintains full chat history for context-aware responses

## Requirements

- [Ollama](https://ollama.com) installed and running (`ollama serve`)
- Llama3 pulled (`ollama pull llama3`)
- Python 3.10+
- `rich` and `requests` (`pip install rich requests`)

## Run

```bash
python3 main.py
```

## Commands

| Command     | Action                        |
|-------------|-------------------------------|
| `/quit`     | Exit the chat                 |
| `/clear`    | Reset conversation history    |
| `/models`   | List available Ollama models  |
| `/backends` | Show all registered backends  |

## Project Structure

```
main.py     — Rich TUI chat loop
router.py   — Model routing (Ollama live, others stubbed)
```

## Roadmap

- [ ] Connect Claude API backend
- [ ] Connect GPT-4o backend
- [ ] Connect Grok backend
- [ ] Add small fast local model option
- [ ] Backend switching mid-conversation
- [ ] System prompt configuration
