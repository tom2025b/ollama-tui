use super::{App, ChatMessage, PendingRequest};
use crate::llm::RouteDecision;
use crate::router::Router;

impl App {
    /// Try to submit the current prompt.
    pub fn submit_prompt(&mut self) -> Option<PendingRequest> {
        let prompt = self.session.input.trim().to_string();

        if prompt.is_empty() {
            self.ui.status = "Write a prompt before pressing Enter.".to_string();
            return None;
        }

        if self.try_execute_command(&prompt) {
            return None;
        }

        if self.session.waiting_for_model {
            self.ui.status = "A model is already answering. Wait for it to finish.".to_string();
            return None;
        }

        let route = self.route_prompt(&prompt);
        let context = self.conversation_context();
        let prompt_for_model = self.rules.prompt_with_rules(&prompt);
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
        });
        self.trim_history();

        Some(PendingRequest {
            prompt: prompt_for_model,
            route,
            context,
        })
    }

    fn try_execute_command(&mut self, prompt: &str) -> bool {
        let crate::command::ParseResult::Command(command) =
            crate::command::parse_slash_command(prompt)
        else {
            return false;
        };

        self.session.input.clear();
        let dispatch = self.commands.command_dispatcher.dispatch(command);
        let available_commands = self.commands.command_dispatcher.available_commands();
        crate::command::execute_dispatch(self, dispatch, &available_commands);
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
}
