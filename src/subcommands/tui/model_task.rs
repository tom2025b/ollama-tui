use tokio::sync::mpsc;

use crate::{
    anthropic,
    llm::Provider,
    ollama, openai,
    subcommands::tui::app::{ModelEvent, PendingRequest},
    xai,
};

/// Run the selected model without blocking the TUI.
///
/// The UI loop should keep drawing while the model is thinking. Spawning a task
/// gives the selected backend time to answer while the terminal stays
/// responsive.
pub fn spawn_model_request(
    request: PendingRequest,
    model_event_tx: mpsc::UnboundedSender<ModelEvent>,
) {
    tokio::spawn(async move {
        let selected_model = request.route.model.clone();
        let provider_label = selected_model.provider.label();

        let stream_result = match &selected_model.provider {
            Provider::Ollama => {
                ollama::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
            Provider::Anthropic => {
                anthropic::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
            Provider::OpenAi => {
                openai::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
            Provider::Xai => {
                xai::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
        };

        let event = match stream_result {
            Ok(_) => ModelEvent::Finished,
            Err(error) => ModelEvent::Failed(format!("{provider_label} request failed: {error:#}")),
        };

        let _ = model_event_tx.send(event);
    });
}
