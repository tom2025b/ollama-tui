use anyhow::Result;

use crate::runtime::Runtime;

pub async fn run(_runtime: &Runtime) -> Result<()> {
    println!("swarm is not implemented yet");
    Ok(())
}
