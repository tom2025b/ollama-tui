use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    ai_suite::run().await
}
