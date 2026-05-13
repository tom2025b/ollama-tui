use ai_suite::{ConversationTurn, Error, stream_prompt, stream_prompt_with_model};
use egui::Context;
use tokio::{runtime::Handle, sync::mpsc};

/// Events streamed from the backend task to the GUI render loop.
pub enum BackendEvent {
    /// One streamed token from the model.
    Token(String),
    /// Response complete. full_text is the assembled response; model_name is
    /// the router's choice (e.g. "claude-sonnet-4-6").
    Done {
        full_text: String,
        model_name: String,
    },
    /// A network or API error.
    Error(Error),
}

/// Spawn a background tokio task that calls stream_prompt and sends events to
/// the GUI via `tx`. Calls `ctx.request_repaint()` after each token so the
/// egui frame loop wakes up promptly instead of waiting for vsync.
pub fn spawn_request(
    prompt: String,
    context: Vec<ConversationTurn>,
    model_id: Option<String>,
    tx: mpsc::UnboundedSender<BackendEvent>,
    ctx: Context,
    handle: Handle,
) {
    handle.spawn(async move {
        // Clone tx and ctx so the on_token closure can capture them while the
        // outer async block retains the originals for the Done/Error sends.
        let tx_tok = tx.clone();
        let ctx_tok = ctx.clone();

        let on_token = move |token| {
            let _ = tx_tok.send(BackendEvent::Token(token));
            ctx_tok.request_repaint();
        };

        let result = match model_id {
            Some(model_id) => stream_prompt_with_model(model_id, prompt, context, on_token).await,
            None => stream_prompt(prompt, context, on_token).await,
        };

        match result {
            Ok((full_text, model_name)) => {
                let _ = tx.send(BackendEvent::Done {
                    full_text,
                    model_name,
                });
            }
            Err(e) => {
                let _ = tx.send(BackendEvent::Error(e));
            }
        }
        ctx.request_repaint();
    });
}
