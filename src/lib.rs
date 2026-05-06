pub mod bootstrap;

mod anthropic;
mod app;
mod command;
mod external;
mod history;
mod keys;
mod llm;
mod model_task;
mod ollama;
mod openai;
mod openai_compatible;
mod router;
mod rules;
mod terminal;
mod ui;
mod xai;

pub use bootstrap::run;
