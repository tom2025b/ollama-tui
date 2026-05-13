# AI Suite — Post-Error-Audit Code Review

Reviewer: Claude (Opus 4.7)
Date: 2026-05-13
Scope: Workspace state after Modules 0–17 of the centralized error-handling audit (commits `509d8d1`..`695c0f6`). Focus on the new `ai_suite::Error`/`Result` surface, propagation through async/stream/provider paths, and overall Rust hygiene. Modular file layout is intentional and is not flagged.

---

## Overall verdict

The migration landed cleanly. The codebase has a single typed error surface, no remaining production `anyhow`, no remaining production `unwrap()`/`expect()`, and a regression test that exercises the original streaming bug class with a real TCP server. The error enum is shaped along the right axes, and `friendly_error` is correctly written against `dyn StdError` so it works uniformly on typed errors and chained sources.

The modular layout works in this codebase's favor for error handling: each provider sub-module owns its own `http.rs`/`stream_parser.rs`/`client.rs` slice, and the three SSE-style providers (Anthropic, Ollama, OpenAI-compatible) reach almost the same shape via independent files rather than a shared trait. That keeps each provider easy to read on its own and is consistent with the project's stated design.

A handful of small, real issues are listed below — all minor, none blocking.

---

## Strengths

- **Error taxonomy is well-shaped.** `errors.rs` distinguishes the right axes: `Io` (path-aware) vs `IoOperation` (pathless), `HttpClientBuild` vs `HttpRequest` vs `HttpStatus`, `ProviderResponse` vs `Streaming`. Most callers can match a single variant without string-sniffing, and the `Display` lines read well in raw form.
- **Allocation discipline.** Variants use `&'static str` for `operation`/`context`/`provider`, so the common error-build path is one `Into<String>` allocation at most (often zero). For an error type that flows through hot streaming paths, this is the right call.
- **UTF-8 boundary is correctly factored.** `llm::append_utf8_chunk` / `finish_utf8_stream` handle split codepoints, decoding errors mid-stream, and truncation at stream end as distinct typed variants (`Error::Utf8`, `Error::Streaming`, `Error::Invariant`). All three SSE providers reuse the same helper, so the fix lands once.
- **The Module 0 regression test is the real article.** `providers/openai_compatible/tests.rs::test_stream_error_propagates` spins up an actual TCP server that lies about `content-length`, then asserts the resulting `Error::Streaming { provider, .. }` variant and that the tokens received *before* the failure were still surfaced. This is the test that the original bug-class would have needed, not a synthetic mock.
- **`friendly_error` works on `dyn StdError`.** Because `chain_text_std` walks `.source()` rather than depending on `anyhow::Chain`, both typed `ai_suite::Error` values and arbitrary error chains render uniformly. The unit tests cover both paths (`typed_missing_api_key_is_rendered`, `typed_http_status_is_classified`).
- **Process-global debug flag is correctly scoped.** `init_debug_mode_from_env` + `toggle_debug_mode` + `debug_mode_enabled` give the GUI's `/debug` slash command and the CLI's `AI_SUITE_DEBUG=1` exactly the same effect, and `friendly_error` reads from one place. The errors-module test note also calls out that the unit tests deliberately don't race on the atomic — that's the right design notice to leave in the file.
- **Typed errors survive the GUI channel.** `ai-suite-gui/src/backend.rs::BackendEvent::Error(ai_suite::Error)` keeps the variant intact across the `mpsc` boundary instead of pre-rendering to a string. That preserves all classification information for the GUI's `error_text()` to use later. Good restraint.
- **No production `expect`/`unwrap`/`panic!`.** The only remaining occurrences are in `#[cfg(test)]` blocks or in `#[ignore]`-gated live smoke tests. The migration met the stated goal here.
- **No remaining `anyhow` anywhere in source or `Cargo.toml`.** Verified by repo-wide grep. The audit changelog claim matches reality.
- **Routing's `primary_ollama_model()` invariant has test coverage** in both `route()` and `explain()` (`routing/tests/invariants.rs`), which is exactly the place a typed `Error::Routing` is the most valuable.

---

## Issues found

### 1. Dead branch in `friendly_error::classify` for Ollama 404 (minor, real)

`ai-suite/src/errors.rs:466`

```rust
if chain.contains("ollama returned 404") {
    return Some(
        "Ollama is running but rejected the request (404). The model name may be wrong or the API path changed."
            .into(),
    );
}
```

This branch is unreachable. `Error::HttpStatus` displays as `"{provider} returned HTTP {status}. ..."`, so the lower-cased chain text contains `"ollama returned http 404"` — note the `"http "` between `"returned"` and `"404"`. The literal `"ollama returned 404"` never appears.

In practice the generic provider+404 branch a few lines later still catches the case, so users get *a* helpful 404 message — just not the more specific one this branch intended ("model name may be wrong or the API path changed"), which is the more useful message for the Ollama-specific case (e.g. wrong tag, missing `/api/chat`).

Fix: change the substring to `"ollama returned http 404"`, or match the typed variant directly via a small structured-classify path on `Error::HttpStatus` before falling back to the chain-string sniffer.

### 2. Loose space-delimited HTTP-code fallback (minor, fragile)

`ai-suite/src/errors.rs:540-543`

```rust
let mentions_code = codes.iter().any(|code| {
    chain.contains(&format!("http {code}")) || chain.contains(&format!(" {code} "))
});
```

The `format!(" {code} ")` fallback is intended to catch wording variations, but it will also match the digits `404`/`429`/etc. anywhere they appear surrounded by spaces — including inside a response body that gets concatenated into the chain text. The `"http {code}"` form is what `Error::HttpStatus` actually emits, so the second branch isn't currently needed and creates a small risk of mis-classification (e.g. a 200 response whose body mentions `" 429 "`).

Recommend dropping the bare-space fallback, or qualifying it with a leading separator like `". 404"`/`"status 404"` if a real call site needs it.

### 3. `Error::routing` used for what is really an invariant violation

`ai-suite/src/routing/mod.rs:55-65`

```rust
pub(super) fn primary_ollama_model(&self) -> Result<LanguageModel> {
    self.models
        .iter()
        .find(|model| model.provider == Provider::Ollama && model.name == PRIMARY_OLLAMA_MODEL)
        .cloned()
        .ok_or_else(|| {
            Error::routing(format!(
                "router is missing required primary Ollama model `{PRIMARY_OLLAMA_MODEL}`"
            ))
        })
}
```

This isn't a routing decision; it's a "the model catalog was built without a required entry" condition that the rest of the catalog construction is supposed to make impossible. `Error::Invariant` exists in the enum for exactly this kind of "should never happen, but if it does we want the chain to read as such" case, and it's already used in `llm::append_utf8_chunk` for an analogous slot. Recommend `Error::invariant(...)` here; it'll read better in the friendly-error chain too. The existing invariant tests in `routing/tests/invariants.rs` would just need their `match` arm updated.

### 4. Duplicate provider name in streaming-chunk error message (cosmetic)

`ai-suite/src/providers/openai_compatible/client.rs:84-92`

```rust
while let Some(chunk) = response.chunk().await.map_err(|source| {
    Error::streaming(
        self.provider_name,
        format!(
            "failed to read {} stream chunk: {source}",
            self.provider_name
        ),
    )
})? {
```

`Error::Streaming` already formats as `"{provider} streaming error: {message}"`, so the full rendered chain reads `"OpenAI streaming error: failed to read OpenAI stream chunk: ..."`. Same shape exists in `providers/ollama/client.rs:112-117` and `providers/anthropic/mod.rs:47-52`. The Module 0 regression test asserts on `"failed to read OpenAI stream chunk"` literally, so any change would touch the assertion — but `format!("failed to read stream chunk: {source}")` would be cleaner and reads better through `friendly_error`.

### 5. Three near-identical `require_success` helpers

`ai-suite/src/providers/ollama/http.rs`, `ai-suite/src/providers/anthropic/http.rs`, `ai-suite/src/providers/openai_compatible/http.rs`

All three differ only in the hardcoded provider string. The openai_compatible variant already takes `provider_name: &'static str`. Given the project's stated preference for "three similar lines is better than a premature abstraction," this is defensible as-is — but if a fourth provider is added it's worth promoting the parametric version to a small shared `providers/http.rs` helper. Not a blocker.

### 6. (Observation, not an issue) Each public `stream_prompt` rebuilds the runtime

`ai-suite/src/stream.rs:30-31, 56, 65, 68-71`

```rust
fn router_from_runtime() -> ModelRouter {
    let runtime = crate::runtime::Runtime::load();
    ModelRouter::new(runtime.config().models())
}
```

Each call to `stream_prompt` / `stream_prompt_with_model` / `available_models` / `route_prompt` does a fresh `Runtime::load()`. That re-reads the config file and env vars every call. It's harmless functionally and arguably the right call for a GUI that may want to pick up config changes between prompts. Worth flagging only so it's a conscious choice rather than an accidental one.

---

## Architecture notes

- The migration's hardest part — preserving non-fatal startup behavior for malformed configs while moving the internal plumbing to typed errors — is handled cleanly in `runtime/config.rs` and `runtime/paths.rs`. The `LoadedRuntimePaths { paths, warnings }` shape lets `bootstrap::run` print warnings on stderr without ever bubbling a startup failure for a bad config file. The two test cases (`missing_home_warns_and_falls_back_to_current_dir`, `current_dir_failure_warns_and_falls_back_to_home`) lock that contract in.
- Provider error variants are well-chosen: `MissingApiKey` and `HttpClientBuild` are pre-flight; `HttpRequest` and `HttpStatus` are request-time; `ProviderResponse`, `Streaming`, `Json`, `Utf8` are response/stream-time. That's the right granularity for `friendly_error` to act on without round-tripping through strings.
- The `Send + Sync` boundary on `Error` is not currently enforced. Since `Error` is `Send` (all sources are `Send`) and used inside `tokio::spawn` payloads (`ai-suite-gui/src/backend.rs:30`), it's worth either adding a `static_assertions::assert_impl_all!(Error: Send, Sync)` or relying on the compile-time check via the `BackendEvent::Error(Error)` enum already crossing the channel. Not a current bug; just a small belt-and-braces step.

---

## Suggested follow-up (optional, in priority order)

1. Fix the dead `"ollama returned 404"` branch (Issue 1). Smallest, real, and the test gap is easy to fill.
2. Switch `primary_ollama_model` to `Error::invariant` (Issue 3) for semantic accuracy.
3. Drop the bare-space HTTP-code fallback in `detect_provider_with_http` (Issue 2).
4. Tidy the duplicated provider name in the streaming-chunk message (Issue 4) — minor cosmetic.

No follow-up required for the migration itself. The audit is closed cleanly.
