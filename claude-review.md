# Brutal Code Review: `ollama-tui` / `ai-suite`

**Scope:** ~6,656 LOC across 108 source files. Rust + ratatui + tokio + reqwest. All files comply with the 120-line cap.

I'm going to call out things that are wrong, fragile, or sloppy. The file-count rule is a constraint I'll respect — every issue below is independent of how many files exist.

---

## CRITICAL

### 1. UTF-8 streaming is silently corrupted in all three providers

`src/anthropic.rs:57`, `src/ollama/client.rs:108`, `src/openai_compatible/client.rs:82`:

```rust
buffer.push_str(&String::from_utf8_lossy(&chunk));
```

`reqwest::Response::chunk()` returns a `Bytes` slice that can split mid-codepoint. `from_utf8_lossy` replaces the trailing partial byte sequence with a `U+FFFD` replacement character (`�`). The next chunk starts with the *continuation* bytes of that codepoint, which are also invalid UTF-8 in isolation, so they get replaced *again*.

Net effect: any non-ASCII content (emoji, accented characters, Chinese, code-block formatting marks, smart quotes, em-dashes, etc.) will randomly turn into `�` symbols depending on chunk boundaries. **The code that's most likely to be tested in English ASCII works fine; production output corrupts silently.**

This is the same bug, copy-pasted three times. The buffer should accumulate `Vec<u8>` and only be decoded at line boundaries — or use `Bytes` chains until a delimiter is found.

### 2. The default xAI model name is fictional

`src/xai.rs:18`:

```rust
pub const DEFAULT_XAI_MODEL: &str = "grok-4.20-reasoning";
```

xAI doesn't ship a model called `grok-4.20-reasoning`. This is either a hallucination or a placeholder that was committed. A user who sets `XAI_API_KEY` without `XAI_MODEL` and gets routed to xAI will receive a 404/400 from the API on every Grok-bound prompt. The README documents this fake name as the intended default, so this isn't a stub — it's wrong.

### 3. `fast_ollama_model_name` will panic

`src/router.rs:57-64`:

```rust
pub(super) fn fast_ollama_model_name(&self) -> String {
    self.models
        .iter()
        .find(|model| model.provider == Provider::Ollama && model.name != PRIMARY_OLLAMA_MODEL)
        .expect("router always contains fast Ollama model")
        ...
}
```

If `OLLAMA_FAST_MODEL=llama3` (matching `PRIMARY_OLLAMA_MODEL`), the predicate excludes both Ollama entries and `expect()` panics. Today it works only because the default constant is `"llama3:latest"` — a string-inequality coincidence, since Ollama itself treats `llama3` and `llama3:latest` as the same model. The code's correctness depends on users not picking a normal name. The function's *name* claims "fast", but the implementation is "first non-primary". Two wrongs that happen to align by string accident.

The same panic-by-design pattern exists at `router.rs:53` with `primary_ollama_model().expect("router always contains primary Ollama model")` — invariants encoded as `expect` are landmines, not invariants.

### 4. Input cursor is misplaced for any non-ASCII character

`src/ui/chrome.rs:48`:

```rust
let cursor_x = area.x + 1 + app.input.len().min(area.width.saturating_sub(2) as usize) as u16;
```

`String::len()` is *bytes*, not visual columns. Type one `é` (2 bytes) and the cursor lands two cells past the character. Type one emoji (4 bytes plus possible width 2) and it lands four cells past, with the wrong width. The terminal width clamp also overflows visually because the same byte length is compared to column count.

Same issue in `src/ui/palette.rs:43`/`64` for command suggestion padding (luckily slash-command names are ASCII so the bug is dormant there, but it'll bite the moment anything wider lands in those structures).

---

## HIGH

### 5. `CommandContext` is a 22-method god trait

`src/command/handlers/session/context.rs:32-56`. Every command handler takes `&mut dyn CommandContext` and gets the entire surface area of the App: history, rules, settings, picker, status, voice, quit, model list. The interface has no cohesion — handlers that only need `set_status` are statically permitted to drop the conversation, mutate rules, or quit. This is a textbook God Interface that defeats most of the point of having a trait at all. Nothing about the file-size rule prevents splitting this into focused sub-traits (`StatusContext`, `HistoryContext`, `RulesContext`, ...) that each handler asks for.

### 6. Stringly-typed settings dispatch

`src/app/settings.rs:12-31`:

```rust
match setting {
    "theme" => self.theme_report(),
    "voice" => self.voice_report(),
    ...
}
match setting {
    "voice.on" => Ok(self.set_voice_enabled(true)),
    "voice.off" => Ok(self.set_voice_enabled(false)),
    "voice.speed" => self.set_voice_speed(value.unwrap_or_default()),
    "voice.mode" => self.set_voice_mode(value.unwrap_or_default()),
    ...
}
```

The handlers in `command/handlers/voice.rs:16-23` *construct* these magic strings:

```rust
"on" => apply_voice_setting(context, command.raw(), "voice.on", None),
```

Two cooperating layers, both stringly-typed, both have to agree on the namespace at runtime. A typo on either side fails with `"Unknown setting."` at runtime — never at compile time. The asymmetry is also weird: the *report* side uses flat keys (`"voice"`) but the *set* side uses dotted keys (`"voice.on"`). And `set_voice_speed`/`set_voice_mode` receive `value.unwrap_or_default()` — the absence of a value silently parses an empty string instead of returning a usage hint.

This is in *exactly* the same codebase that defines `CommandId` and `Provider` enums. There's no reason settings can't ride a typed enum the same way.

### 7. `CommandId` is dead production code

`src/command/registry/types.rs:42`:

```rust
pub struct RegisteredCommand {
    #[allow(dead_code)]
    pub id: CommandId,
    executor: CommandExecutor,
}
```

Only the test file (`command/registry/tests.rs:16-23`) reads `id`. In production the dispatcher resolves to a `RegisteredCommand` whose only meaningful field is the function pointer. `CommandId` is described as a "stable identifier for command behavior", but command behavior is identified by the function pointer; the enum is decorative. Either delete it or use it (e.g., for analytics, telemetry, or context-sensitive disabling). Right now it's a 16-variant enum that pretends to be load-bearing.

### 8. Three nearly-identical stream parsers, three nearly-identical HTTP error wrappers

The file-size rule isn't violated by *abstracting* — it's violated by *consolidating into one big file*. There is no reason `process_*_stream_buffer`, `process_final_*_stream_buffer`, and `process_*_stream_line` need to be re-implemented per provider. The loop structure is identical:

```rust
while let Some(newline_index) = buffer.find('\n') {
    let line = buffer.drain(..=newline_index).collect::<String>();
    process_*_stream_line(provider_name, line.trim(), answer, on_token)?;
}
```

Same for `require_success` in `anthropic/http.rs:4`, `ollama/http.rs:10`, `openai_compatible/http.rs:4` — three near-clones, each with provider-specific message text. A line-buffered stream framework + per-provider line decoders would be one extra file (well under the 120-line cap), zero merges, and one place to fix the UTF-8 bug instead of three.

The fact that this duplication exists is *why* the UTF-8 bug exists in three places.

### 9. `display_name` and `names[0].name` are redundant

`src/command/registry/definitions/core.rs` and friends. Every visible command repeats its name twice: as `display_name: "/clear"` and as the first `CommandName { name: "/clear", visible: true }`. The two can drift (you could ship a definition where the display name doesn't appear in the names list, or vice-versa). `display_name` could be derived (e.g., the first visible name in `names`).

### 10. Sensitive-keyword filter is over-aggressive for the target audience

`src/router/profile/keywords.rs:30-61` lists `tax`, `taxes`, `contract`, `token`, `secret`, `password`, `credentials`, `attorney`, `medication`, `therapy`, `salary`, `lawyer`, `lawsuit`...

This tool is a coding/work assistant. Real prompts that get force-routed to local Ollama:

- "How do I decode a JWT **token**?"
- "Explain smart **contract** semantics in Solana"
- "What's a **secret** in Kubernetes?"
- "I need to write **password** reset logic"
- "Mock auth **credentials** in this test"
- "Add **token**ization to this tokenizer"

Substring `prompt.contains(keyword)` for `LOCAL_ONLY_KEYWORDS` is also too coarse — `"private"` matches `"privatekey"`, `"private member"`, `"private static"`, `"deprivate"`. The phrase list is checked with `.contains()` on the lowered string, which means `"medical records"` matches `"my medical records project"` (fine) but also matches inside arbitrary embedded text.

`SENSITIVE_WORDS` is at least split on non-alphanumeric so `tax` won't match `syntax`. But the words still produce massive false positives in any technical prompt. The classifier is a privacy feature; a routing classifier with this many false positives just trains the user to *disable* the privacy feature.

### 11. `is_simple` swallows almost everything

`src/router/profile.rs:27`: `is_simple: word_count <= 20 || contains_any(...)`. Most user questions are under 21 words. `route_with_rules` checks `needs_deep_reasoning_or_code` first (good), so coding prompts go to Anthropic. Everything else short → fast Ollama. The `is_creative_or_general_cloud` (OpenAI) branch only fires for a *long* prompt that doesn't match deep-reasoning keywords. In practice the OpenAI default is rarely used. README claims "General and creative prompts prefer OpenAI when configured" — this is misleading because `is_simple` overrules it for short prompts.

### 12. `stop_terminal` and `suspend_terminal` are byte-identical

`src/terminal.rs:26-41`. Two functions with different names, exact same body. This is dead code waiting to drift apart. (I'm not asking to merge files — just delete one of the two, or have one call the other.)

### 13. The `theme::*` palette dispatches on stringified theme name

`src/ui/theme.rs` has `match app.theme_name() { "light" => ..., "mono" => ..., _ => ... }` — six times over. The `_` arm catches "dark" *and* anything new. The underlying type is `enum UiTheme`. The `theme_name()` getter returns `&'static str` only because of stringly-typed plumbing further up. If a future variant is added to `UiTheme` and someone forgets to add a case here, it silently falls into the dark theme arm. This is precisely the bug pattern Rust enums + match exhaustiveness exists to prevent — and you've turned it off voluntarily.

Bonus: `theme::highlight_fg` is a dead match — both arms return `Color::Black`:

```rust
pub(super) fn highlight_fg(app: &App) -> Color {
    match app.theme_name() {
        "mono" => Color::Black,
        _ => Color::Black,
    }
}
```

### 14. Long-running model task has no cancellation

`src/model_task.rs:19` spawns a tokio task and forgets the handle. If the user hits Esc to quit during a long stream, the `App` exits but the spawned future continues running until the 300-second timeout (anthropic), 300s (openai/xai), or 300s (ollama). On a paid backend this consumes API tokens *after* the user quit. The receiver's `let _ = model_event_tx.send(...)` masks the channel-closed error, which is the right thing for a fast quit but wrong for cost.

It also means there's no way to abort a misbehaving stream — `/clear` while waiting is explicitly blocked, which is fine, but there's no `/cancel` either.

---

## MEDIUM

### 15. `accept_suggestion` returns `bool` that nobody uses

`src/app/input.rs:78` returns a bool for accept-success. Callers in `src/keys.rs:50, 67` ignore it. Either drop the return or make `keys.rs` use it (e.g., to fall through differently when there were no suggestions to accept).

### 16. `take_external_action` is a single-slot mailbox without a single-slot type

`src/app/state.rs:43`: `pending_external_action: Option<ExternalAction>`. `queue_external_action` blindly does `*self = Some(action)`, silently overwriting any previous action. Today only one action variant exists, so it doesn't matter. Tomorrow when someone adds a second variant and a handler queues both in sequence, the first will be dropped without warning.

### 17. `expand_user_path` test is a tautology

`src/history.rs:103-108`:

```rust
assert!(expanded.is_absolute() || expanded.starts_with("."));
```

`PathBuf::starts_with` checks path components, not a leading character. `home_dir()` falls back to `PathBuf::from(".")` when `HOME` is unset, so the assertion is trivially true under both branches. This test catches nothing.

### 18. Input box has no scroll, no wrap

`src/ui/chrome.rs:43-50`. `Paragraph::new(app.input.clone())` with no `.wrap()` and no horizontal scrolling. Type 200 characters and the input flows off the right edge of the box. The cursor positioning logic clamps to the box width (line 48), so for long input the cursor is *also* wrong — pinned to the right edge while the text spills past it.

### 19. `pub use rules::complete_rules_edit` escapes the dispatcher abstraction

`src/command/handlers.rs:10`. `complete_rules_edit` is called from `external.rs:29` *outside* the normal command flow — it's the post-nano completion hook. It piggybacks on `CommandContext` but doesn't go through `execute_dispatch`. This is fine functionally but means there are two ways into a "command handler": via `CommandRegistry`, and via this side-door. The asymmetry isn't documented. If a second long-lived external action ever appears, you'll repeat this pattern instead of having a real "external action completed" dispatcher.

### 20. The route reasons are hardcoded English with hardcoded model names

`src/router/selection.rs:46`: `"...so I preferred GPT-4o and then fell back by availability."` But `OPENAI_MODEL` is configurable. Set `OPENAI_MODEL=gpt-5o` and the user sees a route reason claiming GPT-4o was preferred. Same for "preferred Claude" and "preferred Grok". The reason text and the actual selected model are out of sync the moment the user overrides the env var.

### 21. `ChatStreamChunk.done` is read but never used

`src/ollama/stream.rs:11-13`:

```rust
#[serde(default)]
#[allow(dead_code)]
done: bool,
```

The Ollama stream sends `{"done": true}` as its final frame. The code instead relies on the HTTP body terminating. If Ollama ever ships a final frame *followed by* trailing garbage in the same chunk (e.g., a status footer), the parser will try to deserialize it and fail. Using the protocol's own end-of-stream signal would be both more robust and less fragile.

### 22. App is a 22+-field bag — picker/suggestions state is loose

`src/app/state.rs`. `voice` and `rules` are properly grouped, but:

- `show_models_picker`, `models_picker_index`, `pinned_model` → could be `picker: ModelPickerState`
- `suggestion_index`, `suggestions_dismissed` → could be `suggestions: SuggestionsState`
- `show_help` → could move to a UI overlay struct

Splitting *fields* into named sub-structs doesn't violate the file-size rule and would make the giant impl set easier to reason about. It would also let `app/input.rs` and `app/models.rs` mutate clearly-scoped state instead of poking at a sea of flat fields. Right now ten `impl App` blocks across nine files all reach into the same flat namespace.

### 23. `setting_report("voice")` vs `set_setting("voice.on", ...)` — namespace asymmetry

`src/app/settings.rs:13-30`. Reads use flat keys, writes use dotted keys. There's no reason for this except path-of-least-resistance during refactor. Pick one.

### 24. `unknown_command` writes to history but uses `command.raw()` as the prompt

`src/command/handlers/session.rs:33-37`:

```rust
context.append_local_message(
    command.raw(),
    format!("Unknown command. Available commands: {available_commands}."),
);
```

If the user types `/clr` (typo for `/clear`), `/clr` shows up in the visible history. Fine. But this is also how `/help` reports unknown commands — and the *complete* available command list, including non-visible aliases (`/q`, `/exit`), is included. Compare with `CommandRegistry::available_commands` (`registry.rs:61`) which calls `help_entries()`. `help_entries` doesn't filter by `visible`. So `/q` (marked `visible: false`) leaks into "Available commands". Either honor the visible flag here or remove the visible flag from `CommandName` since nothing actually filters by it for help purposes. (The suggestion list at `registry.rs:50` *does* honor visible.)

### 25. `voice` setter parsing is silently lossy

`src/app/settings/voice.rs:69`:

```rust
self.voice.speed = (speed * 10.0).round() / 10.0;
```

`/voice speed 1.234` is silently rounded to `1.2`, with no message indicating that. The validation passes, the user sees `Voice speed set to 1.2x.` and assumes their input was accepted. Either accept full precision or reject non-tenth values explicitly.

### 26. `keys.rs` Esc behavior is conditional and undocumented in detail

Esc on main → `app.quit()`. Esc with suggestions visible → dismiss suggestions. Esc with picker open → close picker. Esc with help open → close help. The user-facing docs (`README` "Keyboard" section) just say "Esc quits from the main screen". A user who hits Esc once expecting to dismiss a popup and instead exits the app loses their conversation. The actual behavior is mode-dependent and only obvious if you read `keys.rs`.

Slightly worse: `KeyCode::Esc => app.quit()` (line 80) is *unconditional* on the main screen. Many TUI apps ask for confirmation or require a second press. One stray Esc nukes everything.

---

## LOW

### 27. Most doc-comments tell you WHAT, not WHY

CLAUDE.md asks comments to explain why. The codebase is the opposite: dozens of methods have a one-line comment that just restates the function name in English.

```rust
/// Set the input box content to empty without sending it.
pub fn clear_input(&mut self) { ... }

/// Hide the help overlay.
pub fn hide_help(&mut self) { ... }
```

These add zero information and rot when the function is renamed. Unless they explain a non-obvious reason for the design, they're noise.

### 28. `app/input.rs::backspace` resets `suggestion_index` to 0 unconditionally

If the user has navigated to suggestion #3 and hits backspace to delete a typo, the highlight jumps to #0. Most autocomplete UIs preserve the highlight as long as the highlighted item still matches. Minor UX paper cut.

### 29. `compact` layout doesn't compact the status panel

`src/app/settings/layout.rs:46-48`: `status_panel_height` always returns 5 regardless of mode. So "compact" only shrinks the model panel. With four model rows + 2 borders the model panel in compact mode (`Length(4)`) shows... two model rows with a chopped border. The layout label promises more than it delivers.

### 30. `accept_suggestion` always appends a trailing space

Even for parameterless commands. Tab on `/cl` → `/clear ` → Enter sends a string with trailing space. Parser trims, so functionally fine. But for parameterless commands (`/clear`, `/quit`, `/help`) you could either send the command immediately (Tab + Enter in one keystroke) or skip the trailing space.

### 31. `app/models.rs::accept_model_selection` recomputes `pickable_models()` to `clone()` one entry

```rust
let chosen = self.pickable_models()[index - 1].clone();
```

`pickable_models` collects into a `Vec<&LanguageModel>`, then we index, then clone. Cheap, but allocating a temp Vec to read one element is wasteful.

### 32. `command_dispatcher` is `Copy` but `App` doesn't use that

`CommandDispatcher` is `#[derive(Clone, Copy, Debug, Default)]` with a single static-pointer field. The dispatcher's whole reason to exist as a stateful struct is to hold the registry — which is itself `Copy + Default`. So `CommandDispatcher::new(CommandRegistry::default())` is identical to `CommandDispatcher::default()`. The `App::new` body uses the verbose form. No real harm, just dead expressiveness.

### 33. The `/voice` command builds setting strings; `/theme` and `/resize` go through generic helpers

`command/handlers/ui_quality.rs` has a clean shared `handle_setting_command` that abstracts `theme` and `layout`. `command/handlers/voice.rs` is twice as long with hand-rolled dispatch. The asymmetry suggests one was refactored and the other wasn't, or the abstraction wasn't general enough to absorb voice. Either way, two patterns coexist for the same shape of problem.

---

## Architecture observations within the 120-line rule

Respecting the rule:

- **Where the rule pays off**: each command handler in its own file with a clear name (`backends.rs`, `voice.rs`, `rules.rs`) is genuinely easy to navigate. The provider split (`anthropic/`, `ollama/`, `openai_compatible/`, `xai`) cleanly isolates per-provider quirks.
- **Where the rule gets in the way**:
  - `llm/turn.rs` (8 lines), `llm/route.rs` (10 lines), `command/mod.rs` (11 lines) are pure structural files. That's allowed by the rule but produces a lot of `pub use` re-exporting.
  - Tracing one slash command from keypress to side-effect requires opening: `keys.rs` → `app/prompt.rs` → `command/parser.rs` → `command/dispatcher.rs` → `command/registry.rs` → `command/registry/types.rs` → `command/registry/definitions/X.rs` → `command/handlers/X.rs` → `app/context.rs` (CommandContext impl) → `app/X.rs` (the App method that actually does the work). Ten files. That's a real cost. It's a cost you've chosen, but be honest about paying it.
- **What the rule doesn't fix**: file size has no relationship to whether `CommandContext` is too wide, whether `set_setting` is stringly-typed, whether the stream parsers duplicate logic, whether the route reasons hardcode model names, or whether `from_utf8_lossy` corrupts UTF-8. None of those have anything to do with file size — they're design choices that need to be re-decided regardless of the constraint.

The 120-line rule is fine as a structural constraint but it doesn't *substitute* for clean abstractions. Several files in this codebase are short *and* sloppy: short-and-sloppy is a worse failure mode than long-and-clean, because it lets you tell yourself the modularity is doing the work when it isn't.

---

## TL;DR

The repo passes its self-imposed file-size rule and has decent module names, but underneath:

- **One real data-corruption bug** (UTF-8 streaming) duplicated three times.
- **One real crash bug** (`fast_ollama_model_name` panic).
- **One bogus default model name** (xAI).
- **One real visual bug** (cursor placement on non-ASCII input).
- **A 22-method God trait** the file-size rule doesn't excuse.
- **Stringly-typed settings dispatch** in a Rust codebase, with mismatched namespaces between read and write paths.
- **Sensitive-keyword filter** that triggers on common technical terms in a coding tool — the privacy feature trains users to disable it.
- **Routing reasons** hardcoded around model names that the user can override.
- **A handful of `expect`s** that encode "this can't happen" as "this will panic if it does."
- **Lots of low-information doc-comments** that restate function names and rot on rename.
- **A registration system** with a `CommandId` enum that's dead code in production.

The bones are reasonable. The flesh has rot in it.
