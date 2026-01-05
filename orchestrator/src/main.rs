mod message_bus;
mod router;
mod orchestrator;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

use crate::orchestrator::Orchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("hybrid_llm=debug,info")
        .init();

    info!("🚀 Hybrid LLM Platform starting...");

    // Create and run orchestrator
    let orchestrator = Orchestrator::new().await?;

    info!("✅ Orchestrator initialized");
    info!("🎯 System ready for LLM operations");

    // Run the orchestrator
    orchestrator.run().await?;

    Ok(())
}
