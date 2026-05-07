use tokio::sync::mpsc;

use crate::{
    errors::friendly_error,
    providers::execution::{self, ModelRequest},
    subcommands::tui::app::{ModelEvent, PendingRequest},
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
        let model_request =
            ModelRequest::new(request.route.model.clone(), request.context, request.prompt);
        let provider_label = model_request.provider_label();

        let stream_result = execution::stream_model_request(&model_request, |token| {
            let _ = model_event_tx.send(ModelEvent::Token(token));
        })
        .await;

        let event = match stream_result {
            Ok(_) => ModelEvent::Finished,
            Err(error) => {
                ModelEvent::Failed(format!("{provider_label}: {}", friendly_error(&error)))
            }
        };

        let _ = model_event_tx.send(event);
    });
}
