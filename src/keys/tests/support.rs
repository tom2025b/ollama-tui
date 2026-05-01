use super::super::ModelEvent;
use tokio::sync::mpsc;

pub(super) fn model_event_sender() -> mpsc::UnboundedSender<ModelEvent> {
    let (sender, _receiver) = mpsc::unbounded_channel();
    sender
}
