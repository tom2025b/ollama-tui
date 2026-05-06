# ai-suite Migration Plan

This plan migrates `ollama-tui` into one clean Rust binary named `ai-suite`.
The target architecture supports multiple subcommands, keeps the public version
professional, allows a private hybrid version through extension points, and
keeps every file focused and small.

## 1. Final Folder Structure

```text
src/
  main.rs
  lib.rs

  bootstrap.rs

  cli/
    mod.rs
    args.rs
    dispatch.rs

  runtime/
    mod.rs
    context.rs
    env.rs
    paths.rs

  config/
    mod.rs
    settings.rs
    secrets.rs
    profiles.rs

  subcommands/
    mod.rs
    registry.rs
    spec.rs

    tui/
      mod.rs
      run.rs
      app/
      ui/
      terminal.rs
      input.rs
      external.rs
      model_task.rs
      slash_commands/

    swarm/
      mod.rs
      run.rs

    food/
      mod.rs
      run.rs

  llm/
    mod.rs
    model.rs
    provider.rs
    route.rs
    turn.rs

  providers/
    mod.rs
    ollama/
    anthropic/
    openai_compatible/
    openai.rs
    xai.rs

  routing/
    mod.rs
    catalog.rs
    profile.rs
    profile/
    selection.rs

  prompt_rules/
    mod.rs
    content.rs
    paths.rs
    prompt.rs
    report.rs
    state.rs
    state/
    storage.rs
    target.rs

  storage/
    mod.rs
    history.rs

  tools/
    mod.rs
    registry.rs
    spec.rs
    execution.rs
    builtins/
      mod.rs

  extensions/
    mod.rs
    api.rs
    public.rs
    registry.rs
```

## 2. Files That Stay Conceptually

These files stay conceptually where they are in the architecture, though some
become thinner or move from a flat file to a `mod.rs`.

```text
src/main.rs              # remains binary entrypoint, but shrinks to ai_suite::run()
src/llm.rs               # becomes src/llm/mod.rs
src/llm/model.rs
src/llm/provider.rs
src/llm/route.rs
src/llm/turn.rs
```

The `llm` module is already provider-neutral enough to remain a shared core
module.

## 3. Files To Move Or Rename

### TUI-Specific Modules

```text
src/app/                 -> src/subcommands/tui/app/
src/app.rs               -> src/subcommands/tui/app/mod.rs

src/ui/                  -> src/subcommands/tui/ui/
src/ui.rs                -> src/subcommands/tui/ui/mod.rs

src/command/             -> src/subcommands/tui/slash_commands/
src/command/mod.rs       -> src/subcommands/tui/slash_commands/mod.rs

src/keys.rs              -> src/subcommands/tui/input.rs
src/keys/                -> src/subcommands/tui/input/tests/

src/terminal.rs          -> src/subcommands/tui/terminal.rs
src/external.rs          -> src/subcommands/tui/external.rs
src/model_task.rs        -> src/subcommands/tui/model_task.rs
```

Top-level CLI subcommands and TUI slash commands are different concepts. The
current `src/command/*` module belongs under the TUI as `slash_commands`.

### Provider Modules

```text
src/ollama.rs            -> src/providers/ollama/mod.rs
src/ollama/              -> src/providers/ollama/

src/anthropic.rs         -> src/providers/anthropic/mod.rs
src/anthropic/           -> src/providers/anthropic/

src/openai_compatible.rs -> src/providers/openai_compatible/mod.rs
src/openai_compatible/   -> src/providers/openai_compatible/

src/openai.rs            -> src/providers/openai.rs
src/xai.rs               -> src/providers/xai.rs
```

### Shared Non-TUI Modules

```text
src/router.rs            -> src/routing/mod.rs
src/router/              -> src/routing/

src/rules.rs             -> src/prompt_rules/mod.rs
src/rules/               -> src/prompt_rules/

src/history.rs           -> src/storage/history.rs
```

### Entrypoint Extraction

```text
current src/main.rs app loop -> src/subcommands/tui/run.rs
new src/main.rs              -> tiny binary entrypoint
new src/lib.rs               -> public crate root
new src/bootstrap.rs         -> top-level ai-suite runner
```

## 4. New Files And Folders To Create

Create these first as empty or thin modules:

```text
src/lib.rs
src/bootstrap.rs

src/cli/mod.rs
src/cli/args.rs
src/cli/dispatch.rs

src/runtime/mod.rs
src/runtime/context.rs
src/runtime/env.rs
src/runtime/paths.rs

src/config/mod.rs
src/config/settings.rs
src/config/secrets.rs
src/config/profiles.rs

src/subcommands/mod.rs
src/subcommands/registry.rs
src/subcommands/spec.rs
src/subcommands/tui/mod.rs
src/subcommands/tui/run.rs
src/subcommands/swarm/mod.rs
src/subcommands/swarm/run.rs
src/subcommands/food/mod.rs
src/subcommands/food/run.rs

src/providers/mod.rs

src/storage/mod.rs

src/tools/mod.rs
src/tools/spec.rs
src/tools/registry.rs
src/tools/execution.rs
src/tools/builtins/mod.rs

src/extensions/mod.rs
src/extensions/api.rs
src/extensions/public.rs
src/extensions/registry.rs
```

## 5. Step-By-Step Migration Order

### Step 1: Add Library Entrypoint

Add `src/lib.rs` and `src/bootstrap.rs`.

Target `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ai_suite::run().await
}
```

Target `src/lib.rs`:

```rust
pub mod bootstrap;

pub async fn run() -> anyhow::Result<()> {
    bootstrap::run().await
}
```

### Step 2: Move The Current App Loop Into The TUI Subcommand

Create:

```text
src/subcommands/mod.rs
src/subcommands/tui/mod.rs
src/subcommands/tui/run.rs
```

Move the current `main.rs` event loop into:

```text
src/subcommands/tui/run.rs
```

Initially, `bootstrap::run()` should call only:

```rust
crate::subcommands::tui::run::run().await
```

Do not add CLI parsing yet. Preserve current behavior first.

### Step 3: Move TUI Internals Under `subcommands/tui`

Move in this order:

```text
src/app/ and src/app.rs
src/ui/ and src/ui.rs
src/keys.rs
src/terminal.rs
src/external.rs
src/model_task.rs
src/command/
```

Rename:

```text
command -> slash_commands
keys.rs -> input.rs
```

Update imports mechanically:

```text
crate::app      -> crate::subcommands::tui::app
crate::ui       -> crate::subcommands::tui::ui
crate::keys     -> crate::subcommands::tui::input
crate::terminal -> crate::subcommands::tui::terminal
crate::external -> crate::subcommands::tui::external
crate::command  -> crate::subcommands::tui::slash_commands
```

### Step 4: Move Providers Into `src/providers`

Create `src/providers/mod.rs`:

```rust
pub mod anthropic;
pub mod ollama;
pub mod openai;
pub mod openai_compatible;
pub mod xai;
```

Move the provider files and directories:

```text
src/ollama.rs            -> src/providers/ollama/mod.rs
src/ollama/              -> src/providers/ollama/
src/anthropic.rs         -> src/providers/anthropic/mod.rs
src/anthropic/           -> src/providers/anthropic/
src/openai_compatible.rs -> src/providers/openai_compatible/mod.rs
src/openai_compatible/   -> src/providers/openai_compatible/
src/openai.rs            -> src/providers/openai.rs
src/xai.rs               -> src/providers/xai.rs
```

Update imports:

```text
crate::ollama            -> crate::providers::ollama
crate::anthropic         -> crate::providers::anthropic
crate::openai_compatible -> crate::providers::openai_compatible
crate::openai            -> crate::providers::openai
crate::xai               -> crate::providers::xai
```

### Step 5: Move Shared Routing And Prompt Rules

Move:

```text
src/router.rs -> src/routing/mod.rs
src/router/   -> src/routing/
src/rules.rs  -> src/prompt_rules/mod.rs
src/rules/    -> src/prompt_rules/
```

Update imports:

```text
crate::router -> crate::routing
crate::rules  -> crate::prompt_rules
```

### Step 6: Move History Into Storage

Move:

```text
src/history.rs -> src/storage/history.rs
```

Create `src/storage/mod.rs`:

```rust
pub mod history;
```

Update imports:

```text
crate::history -> crate::storage::history
```

### Step 7: Add Real Top-Level CLI Parsing

Add `clap`:

```toml
clap = { version = "4", features = ["derive"] }
```

Add:

```text
src/cli/mod.rs
src/cli/args.rs
src/cli/dispatch.rs
```

Support:

```text
ai-suite
ai-suite tui
ai-suite swarm
ai-suite food
```

No-argument behavior should run `tui`, preserving today’s behavior.

### Step 8: Add Subcommand Contracts

Add:

```text
src/subcommands/spec.rs
src/subcommands/registry.rs
```

Start simple:

```rust
pub trait Subcommand {
    fn name(&self) -> &'static str;
}
```

Do not overbuild async trait objects yet. CLI dispatch can stay explicit until
real duplication appears.

### Step 9: Add Stub `swarm` And `food` Subcommands

Create:

```text
src/subcommands/swarm/mod.rs
src/subcommands/swarm/run.rs
src/subcommands/food/mod.rs
src/subcommands/food/run.rs
```

Initial behavior:

```text
ai-suite swarm -> prints "swarm is not implemented yet"
ai-suite food  -> prints "food is not implemented yet"
```

This proves the binary architecture without dragging unfinished features into
the TUI.

### Step 10: Add Tool Architecture

Create interfaces only:

```text
src/tools/spec.rs
src/tools/registry.rs
src/tools/execution.rs
src/tools/builtins/mod.rs
```

Do not migrate slash commands into tools yet. Top-level tools should stay
provider-neutral and reusable by `tui`, `swarm`, `food`, and future
subcommands.

### Step 11: Add Extension Architecture

Create:

```text
src/extensions/api.rs
src/extensions/public.rs
src/extensions/registry.rs
```

Initial shape:

```rust
pub trait ExtensionPack {
    fn register_tools(&self, tools: &mut ToolRegistry);
}
```

Later this can grow to register subcommands and providers.

The public repo should register only `extensions::public`. A private hybrid
version can add another extension pack in a private branch or private overlay
crate.

### Step 12: Final Cleanup Pass

Run:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
find src -name '*.rs' -exec wc -l {} +
```

Any file approaching 400 lines should be split before adding new features.

## Recommended PR/Commit Order

```text
1. Extract lib/bootstrap/tui runner without behavior changes
2. Move TUI modules under subcommands/tui
3. Move providers under providers
4. Rename router/rules/history into shared architecture modules
5. Add clap and explicit tui/swarm/food top-level CLI
6. Add subcommand contracts and stubs
7. Add tool registry interfaces
8. Add extension registry interfaces
9. Cleanup, docs, and final validation
```

This sequence keeps each change mechanical, reviewable, and testable.
