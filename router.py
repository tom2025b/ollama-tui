# router.py — Model routing layer for ollama-me
#
# This module handles all LLM backend communication.
# Currently implements: Ollama/Llama (fully working)
# Stubs ready for: small local model, Claude API, GPT-4o, Grok
#
# The router pattern lets main.py stay backend-agnostic —
# swap models by changing one string, not rewriting the UI.

import requests  # HTTP library for talking to Ollama's REST API
import json      # For parsing streaming JSON responses line-by-line


# ─── Configuration ────────────────────────────────────────────────────
# Ollama runs a local HTTP server on port 11434 by default.
# The /api/chat endpoint handles multi-turn conversations.
OLLAMA_BASE_URL = "http://localhost:11434"
OLLAMA_CHAT_URL = f"{OLLAMA_BASE_URL}/api/chat"
OLLAMA_TAGS_URL = f"{OLLAMA_BASE_URL}/api/tags"

# Default model — matches what's pulled locally
DEFAULT_OLLAMA_MODEL = "llama3"


# ─── Ollama Backend (LIVE) ────────────────────────────────────────────

def ollama_available():
    """Check if the Ollama server is reachable and responding."""
    # A simple GET to /api/tags returns the list of installed models.
    # If this fails, Ollama isn't running or isn't installed.
    try:
        resp = requests.get(OLLAMA_TAGS_URL, timeout=3)
        return resp.status_code == 200
    except requests.ConnectionError:
        return False


def ollama_list_models():
    """Return a list of model names available in the local Ollama instance."""
    # The /api/tags endpoint returns JSON with a "models" array.
    # Each model dict has a "name" field like "llama3:latest".
    try:
        resp = requests.get(OLLAMA_TAGS_URL, timeout=5)
        resp.raise_for_status()
        data = resp.json()
        return [m["name"] for m in data.get("models", [])]
    except (requests.RequestException, KeyError):
        return []


def ollama_model_info(model=DEFAULT_OLLAMA_MODEL):
    """Get size and parameter details for a specific Ollama model."""
    # Pulls details from the /api/tags listing — no separate endpoint needed.
    try:
        resp = requests.get(OLLAMA_TAGS_URL, timeout=5)
        resp.raise_for_status()
        for m in resp.json().get("models", []):
            # Match by name (with or without the ":latest" tag)
            if m["name"].startswith(model):
                details = m.get("details", {})
                return {
                    "name": m["name"],
                    "size_gb": round(m["size"] / 1e9, 1),
                    "parameters": details.get("parameter_size", "unknown"),
                    "quantization": details.get("quantization_level", "unknown"),
                    "family": details.get("family", "unknown"),
                }
    except requests.RequestException:
        pass
    return None


def chat_ollama(messages, model=DEFAULT_OLLAMA_MODEL, stream=True):
    """Send a conversation to Ollama and yield response tokens as they arrive.

    Args:
        messages: List of dicts with "role" and "content" keys.
                  Roles are "user", "assistant", or "system".
        model:    Which Ollama model to use (must be pulled locally).
        stream:   If True, yield tokens one at a time for live display.
                  If False, yield the complete response as a single string.

    Yields:
        String chunks of the assistant's response.
    """
    # Build the request payload — Ollama's chat API mirrors OpenAI's format.
    payload = {
        "model": model,
        "messages": messages,
        "stream": stream,
    }

    try:
        # stream=True on the requests side means we get the HTTP body
        # incrementally — important because Ollama sends one JSON object
        # per line as it generates each token.
        resp = requests.post(
            OLLAMA_CHAT_URL,
            json=payload,
            stream=stream,
            timeout=120,
        )
        resp.raise_for_status()

        if stream:
            # Each line is a complete JSON object with a "message" field.
            # The "done" field is True on the final line.
            for line in resp.iter_lines(decode_unicode=True):
                if not line:
                    continue
                chunk = json.loads(line)
                token = chunk.get("message", {}).get("content", "")
                if token:
                    yield token
        else:
            # Non-streaming: the entire response comes back at once.
            data = resp.json()
            content = data.get("message", {}).get("content", "")
            if content:
                yield content

    except requests.ConnectionError:
        yield "[Error] Cannot reach Ollama — is it running? Try: ollama serve"
    except requests.Timeout:
        yield "[Error] Ollama request timed out after 120s."
    except requests.RequestException as e:
        yield f"[Error] Ollama request failed: {e}"


# ─── Small Fast Local Model (STUB) ───────────────────────────────────

def chat_local(messages, model="tinyllama"):
    """Stub: Route to a small, fast local model for quick tasks.

    Intended for lightweight queries where speed > quality —
    things like summarizing a short text or simple Q&A.
    Will use Ollama with a smaller model (e.g., tinyllama, phi3-mini).
    """
    # TODO: Pull and configure a small model in Ollama
    # TODO: Add automatic model-pull if not found locally
    yield "[Stub] Small local model not yet connected."


# ─── Claude API (STUB) ───────────────────────────────────────────────

def chat_claude(messages, model="claude-sonnet-4-6"):
    """Stub: Route to Anthropic's Claude API.

    Will require an ANTHROPIC_API_KEY in the environment.
    Uses the Anthropic Python SDK for clean integration.
    """
    # TODO: pip install anthropic
    # TODO: Read ANTHROPIC_API_KEY from env
    # TODO: Convert message format (Ollama-style → Anthropic-style)
    yield "[Stub] Claude API not yet connected."


# ─── GPT-4o API (STUB) ───────────────────────────────────────────────

def chat_gpt4o(messages, model="gpt-4o"):
    """Stub: Route to OpenAI's GPT-4o API.

    Will require an OPENAI_API_KEY in the environment.
    Uses the OpenAI Python SDK.
    """
    # TODO: pip install openai
    # TODO: Read OPENAI_API_KEY from env
    # TODO: Convert message format if needed
    yield "[Stub] GPT-4o API not yet connected."


# ─── Grok API (STUB) ─────────────────────────────────────────────────

def chat_grok(messages, model="grok-2"):
    """Stub: Route to xAI's Grok API.

    Will require an XAI_API_KEY in the environment.
    Grok uses an OpenAI-compatible endpoint format.
    """
    # TODO: Read XAI_API_KEY from env
    # TODO: Point requests at https://api.x.ai/v1/chat/completions
    yield "[Stub] Grok API not yet connected."


# ─── Router Dispatch ─────────────────────────────────────────────────

# Maps backend names to their handler functions.
# This dict is the single source of truth for available backends.
BACKENDS = {
    "ollama": chat_ollama,
    "local": chat_local,
    "claude": chat_claude,
    "gpt4o": chat_gpt4o,
    "grok": chat_grok,
}


def route(messages, backend="ollama", **kwargs):
    """Dispatch a conversation to the specified backend.

    This is the main entry point that main.py calls.
    It abstracts away which LLM is actually generating the response,
    so the UI code never needs to know about API specifics.

    Args:
        messages: Conversation history as a list of role/content dicts.
        backend:  Which backend to use (key from BACKENDS dict).
        **kwargs: Extra args passed through to the backend function
                  (e.g., model name, temperature).

    Yields:
        String chunks from the chosen backend's response.
    """
    handler = BACKENDS.get(backend)
    if handler is None:
        yield f"[Error] Unknown backend '{backend}'. Available: {', '.join(BACKENDS.keys())}"
        return
    # yield from passes through the generator — each token from the
    # backend flows directly to the caller without buffering.
    yield from handler(messages, **kwargs)


def list_backends():
    """Return a list of all registered backend names."""
    return list(BACKENDS.keys())


# ─── Learning Notes ──────────────────────────────────────────────────
#
# 1. Generator pattern (yield / yield from): Each chat function is a
#    generator that yields tokens incrementally. This lets the TUI
#    display text as it's generated, not after the full response.
#
# 2. Ollama's REST API: Runs on localhost:11434. The /api/chat endpoint
#    accepts OpenAI-style messages and streams back JSON lines.
#
# 3. requests.post(stream=True): Tells the requests library to not
#    buffer the full response — iter_lines() gives us one line at a
#    time as the server sends them.
#
# 4. Router pattern: A dict mapping names → functions. Adding a new
#    backend is just: write the function, add it to BACKENDS.
#
# 5. Stubs as generators: Even stub functions use "yield" so they
#    match the same interface as live backends. The caller doesn't
#    need to know if a backend is real or a placeholder.
