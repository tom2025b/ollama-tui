pub mod bootstrap;

mod anthropic;
mod history;
mod llm;
mod ollama;
mod openai;
mod openai_compatible;
mod router;
mod rules;
mod subcommands;
mod xai;

pub use bootstrap::run;
