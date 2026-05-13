use crate::Result;
use crate::runtime::Runtime;

pub async fn run(_runtime: &Runtime) -> Result<()> {
    let tools = crate::subcommands::capabilities::public_tool_registry()?;

    println!("Food planner");
    println!(
        "A simple local starter plan is available without sending private food data anywhere."
    );
    println!("Registered public tools: {}", tools.len());
    println!("Day 1: rice bowl with beans, greens, and yogurt sauce");
    println!("Day 2: sheet-pan vegetables with eggs or tofu");
    println!("Day 3: lentil soup with toast and a side salad");

    Ok(())
}
