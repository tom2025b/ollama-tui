use anyhow::{Context, Result};

use super::OllamaClient;
use crate::llm::{ConversationTurn, append_utf8_chunk, finish_utf8_stream};
use crate::ollama::http::{connection_error, require_success};
use crate::ollama::stream::{process_final_ollama_stream_buffer, process_ollama_stream_buffer};
use crate::ollama::types::ChatRequest;

impl OllamaClient {
    pub(super) async fn stream_without_model_check<F>(
        &self,
        model_name: &str,
        context: &[ConversationTurn],
        prompt: &str,
        mut on_token: F,
    ) -> Result<String>
    where
        F: FnMut(String),
    {
        let url = self.api_url("/api/chat");
        let request = ChatRequest::new(model_name, context, prompt);
        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| connection_error(&url))?;

        let mut response = require_success(response).await?;
        let (mut buffer, mut answer, mut pending_utf8) = (String::new(), String::new(), Vec::new());

        while let Some(chunk) = response
            .chunk()
            .await
            .context("failed to read Ollama stream chunk")?
        {
            append_utf8_chunk("Ollama", &mut pending_utf8, &mut buffer, &chunk)?;
            process_ollama_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
        }

        finish_utf8_stream("Ollama", &mut pending_utf8, &mut buffer)?;
        process_final_ollama_stream_buffer(&mut buffer, &mut answer, &mut on_token)?;
        Ok(answer)
    }
}
