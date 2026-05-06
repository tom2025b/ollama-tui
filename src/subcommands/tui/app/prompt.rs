use std::path::PathBuf;

use super::{App, ChatMessage, PendingRequest};
use crate::llm::{Provider, RouteDecision};
use crate::routing::Router;
use crate::subcommands::tui::slash_commands::{self, ExternalAction, ParseResult, handlers};

impl App {
    /// Try to submit the current prompt.
    pub fn submit_prompt(&mut self) -> Option<PendingRequest> {
        let prompt = self.session.input.trim().to_string();

        if prompt.is_empty() {
            self.ui.status = "Write a prompt before pressing Enter.".to_string();
            return None;
        }

        // Slash commands may stage a follow-up prompt (e.g. /fix, /explain,
        // /review). When they do, fall through and treat that staged prompt as
        // the user's real submission so the model actually sees it.
        let prompt = if self.try_execute_command(&prompt) {
            self.commands.take_staged_prompt()?
        } else {
            prompt
        };

        if self.session.waiting_for_model {
            self.ui.status = "A model is already answering. Wait for it to finish.".to_string();
            return None;
        }

        let route = self.route_prompt(&prompt);

        // Terminal apps (Claude Code, Codex) get the prompt forwarded directly as
        // a CLI argument. We skip context and rules-wrapping here — those are
        // Ollama-specific concerns; terminal apps have their own context systems.
        if let Some(action) = self.terminal_action_for_route(&route, &prompt) {
            self.session.input.clear();
            self.ui.scroll_offset = 0;
            let model_name = route.model.display_label();
            self.ui.status = format!("Forwarding to {model_name}...");
            self.session.history.push(ChatMessage {
                prompt: prompt.clone(),
                model_name,
                route_reason: route.reason,
                answer: "→ Prompt forwarded. Working in terminal app — exit to return here."
                    .to_string(),
                in_progress: false,
                failed: false,
                include_in_context: false,
                is_local_message: false,
            });
            self.trim_history();
            self.commands.queue_external_action(action);
            return None;
        }

        // Ollama streaming path.
        let context = self.conversation_context();
        let prompt_for_model = self.prompt_for_model(&prompt);
        let model_name = route.model.display_label();
        let route_reason = if let Some(rules_summary) = self.rules.application_summary() {
            format!("{} {}", route.reason, rules_summary)
        } else {
            route.reason.clone()
        };

        self.session.input.clear();
        self.session.waiting_for_model = true;
        self.session.active_model_name = Some(model_name.clone());
        self.session.activity_tick = 0;
        self.ui.scroll_offset = 0;
        self.ui.status = format!("Sent to {model_name}. Waiting for first token...");

        self.session.history.push(ChatMessage {
            prompt: prompt.clone(),
            model_name,
            route_reason,
            answer: String::new(),
            in_progress: true,
            failed: false,
            include_in_context: true,
            is_local_message: false,
        });
        self.trim_history();

        Some(PendingRequest {
            prompt: prompt_for_model,
            route,
            context,
        })
    }

    fn try_execute_command(&mut self, prompt: &str) -> bool {
        let ParseResult::Command(command) = slash_commands::parse_slash_command(prompt) else {
            return false;
        };

        self.session.input.clear();
        match self.commands.registry.resolve(&command) {
            Some(execute) => execute(self, &command),
            None => {
                let available_commands = self.commands.registry.available_commands();
                handlers::session::unknown_command(self, &command, &available_commands);
            }
        }
        true
    }

    fn route_prompt(&self, prompt: &str) -> RouteDecision {
        if let Some(pinned) = &self.routing.pinned_model {
            return RouteDecision {
                model: pinned.clone(),
                reason: format!(
                    "Pinned to {} via /model picker. Router skipped.",
                    pinned.display_label()
                ),
            };
        }

        self.routing.router.route(prompt)
    }

    fn terminal_action_for_route(
        &self,
        route: &RouteDecision,
        prompt: &str,
    ) -> Option<ExternalAction> {
        let working_dir = self
            .runtime
            .paths()
            .project_root()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        match route.model.provider {
            Provider::ClaudeCode => Some(ExternalAction::ClaudeCode {
                working_dir,
                prompt: prompt.to_string(),
            }),
            Provider::Codex => Some(ExternalAction::CodexCli {
                working_dir,
                prompt: prompt.to_string(),
            }),
            Provider::Ollama => None,
        }
    }

    fn prompt_for_model(&self, prompt: &str) -> String {
        let notes = self.memory.notes_prompt_prefix();
        let effective_prompt = if notes.is_empty() {
            prompt.to_string()
        } else {
            format!("{notes}\n{prompt}")
        };
        self.rules.prompt_with_rules(&effective_prompt)
    }
}
