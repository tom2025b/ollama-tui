#!/usr/bin/env python3
# main.py — Rich TUI chat interface for ollama-me
#
# A terminal-based chat UI that talks to LLMs through the router.
# Currently wired to Ollama/Llama3 for local inference.
# Rich handles all the pretty formatting — panels, markdown, colors.

import sys                      # For clean exit handling
from rich.console import Console  # Core Rich object — all output goes through this
from rich.panel import Panel      # Bordered boxes for visual separation
from rich.markdown import Markdown  # Renders markdown (bold, code blocks, lists) in terminal
from rich.text import Text        # Styled text objects for fine-grained formatting
from rich.live import Live        # Live-updating display — redraws in place as tokens arrive
from rich.rule import Rule        # Horizontal divider lines

import router                   # Our model routing layer — handles all LLM communication


def show_banner(console, model_info):
    """Display the startup banner with model details.

    Shows a styled panel with the connected model's name, size,
    and parameter count so the user knows what they're talking to.
    """
    # Build the info lines — only show fields we actually got back
    lines = ["[bold cyan]ollama-me[/] — Local LLM Chat\n"]

    if model_info:
        lines.append(f"[green]Model:[/]   {model_info['name']}")
        lines.append(f"[green]Params:[/]  {model_info['parameters']}")
        lines.append(f"[green]Size:[/]    {model_info['size_gb']} GB")
        lines.append(f"[green]Quant:[/]   {model_info['quantization']}")
    else:
        lines.append("[yellow]No model info available[/]")

    # Rich markup uses bracket tags like [bold], [green], etc.
    # They work inside Panel content just like HTML tags.
    lines.append("")
    lines.append("[dim]Type your message and press Enter. Commands:[/]")
    lines.append("[dim]  /quit — exit    /clear — reset chat    /models — list models[/]")

    console.print(Panel(
        "\n".join(lines),
        border_style="cyan",
        padding=(1, 2),
    ))


def stream_response(console, messages):
    """Stream the LLM response token-by-token with a live-updating display.

    Uses Rich's Live context manager to redraw the output panel
    as each token arrives from the model. This gives the "typing"
    effect that makes chat feel responsive.

    Returns:
        The complete response text (for adding to conversation history).
    """
    full_response = ""

    # Live() takes over a region of the terminal and redraws it.
    # refresh_per_second controls how often the display updates —
    # 15 is smooth without burning CPU.
    with Live(console=console, refresh_per_second=15, vertical_overflow="visible") as live:
        for token in router.route(messages, backend="ollama"):
            full_response += token
            # Re-render the entire response as Markdown on each token.
            # This means formatting (bold, code blocks) appears correctly
            # even mid-stream, not just after the response finishes.
            md = Markdown(full_response)
            panel = Panel(md, title="[bold green]llama3[/]", border_style="green", padding=(0, 1))
            live.update(panel)

    return full_response


def handle_command(command, console, history):
    """Process slash commands. Returns True if the command was handled.

    Slash commands let the user control the app without leaving
    the chat flow — /quit to exit, /clear to reset, etc.
    """
    cmd = command.strip().lower()

    if cmd in ("/quit", "/exit", "/q"):
        console.print("\n[dim]Goodbye![/]\n")
        sys.exit(0)

    elif cmd == "/clear":
        # Reset conversation history but keep the system prompt.
        # history is a list, so .clear() empties it in place —
        # no need to return a new list.
        history.clear()
        console.clear()
        console.print("[dim]Chat history cleared.[/]\n")
        return True

    elif cmd == "/models":
        # Show what models are available locally in Ollama
        models = router.ollama_list_models()
        if models:
            console.print(Panel(
                "\n".join(f"  • {m}" for m in models),
                title="[bold]Available Models[/]",
                border_style="blue",
                padding=(0, 1),
            ))
        else:
            console.print("[yellow]No models found — is Ollama running?[/]")
        return True

    elif cmd == "/backends":
        # Show all registered backends (live + stubs)
        backends = router.list_backends()
        console.print(Panel(
            "\n".join(f"  • {b}" for b in backends),
            title="[bold]Registered Backends[/]",
            border_style="blue",
            padding=(0, 1),
        ))
        return True

    elif cmd.startswith("/"):
        console.print(f"[yellow]Unknown command: {cmd}[/]")
        return True

    # Not a command — return False so the caller sends it to the LLM
    return False


def main():
    """Main chat loop — handles startup, input, and response streaming."""
    console = Console()

    # ── Startup checks ──
    # Verify Ollama is reachable before entering the chat loop.
    # Better to fail fast with a clear message than hang on the first prompt.
    if not router.ollama_available():
        console.print(Panel(
            "[bold red]Cannot connect to Ollama![/]\n\n"
            "Make sure Ollama is running:\n"
            "  [cyan]ollama serve[/]\n\n"
            "Then try again.",
            border_style="red",
            padding=(1, 2),
        ))
        sys.exit(1)

    # Fetch model details for the banner display
    model_info = router.ollama_model_info()
    show_banner(console, model_info)

    # ── Conversation history ──
    # Ollama's chat API is stateless — we send the full conversation
    # each time so the model has context. This list accumulates
    # user messages and assistant responses.
    history = []

    # ── Main loop ──
    # Simple read-eval-print loop: get input, check for commands,
    # send to model, display response, repeat.
    while True:
        try:
            # Rich's input() adds styling to the prompt text itself.
            # The \n before the prompt gives visual breathing room.
            console.print()
            user_input = console.input("[bold cyan]You:[/] ").strip()

            # Skip empty input — don't send blank messages to the model
            if not user_input:
                continue

            # Check if it's a slash command before sending to the LLM
            if handle_command(user_input, console, history):
                continue

            # Add the user's message to conversation history
            history.append({"role": "user", "content": user_input})

            # Stream the model's response and capture the full text
            console.print()  # Spacing before the response panel
            response = stream_response(console, history)

            # Add the assistant's response to history for context
            # in future turns — this is how multi-turn chat works.
            history.append({"role": "assistant", "content": response})

        except KeyboardInterrupt:
            # Ctrl+C during input or generation — exit cleanly
            console.print("\n\n[dim]Interrupted. Goodbye![/]\n")
            break
        except EOFError:
            # Ctrl+D (end of input) — also exit cleanly
            console.print("\n[dim]Goodbye![/]\n")
            break


# Standard Python entry point guard — only runs main() when
# this file is executed directly, not when imported.
if __name__ == "__main__":
    main()


# ─── Learning Notes ──────────────────────────────────────────────────
#
# 1. Rich Console: The central object for terminal output. Using
#    console.print() instead of print() enables markup like
#    [bold cyan]text[/] for styled output.
#
# 2. Rich Live: A context manager that takes over a terminal region
#    and redraws it. Perfect for streaming LLM output — update the
#    panel on each token and Live handles the redraw.
#
# 3. Markdown rendering: Rich can render markdown in the terminal,
#    so code blocks, bold text, and lists from the LLM look proper.
#
# 4. Conversation history: Ollama's chat API is stateless. We maintain
#    a list of {"role": ..., "content": ...} dicts and send the full
#    list on each request so the model has context.
#
# 5. Generator consumption: stream_response() iterates over the
#    generator from router.route(), consuming tokens one at a time.
#    This is Python's lazy evaluation pattern — no buffering needed.
