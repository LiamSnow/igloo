mod manager;
pub mod state;

use std::sync::Arc;

pub use state::GlacierState;
use tokio::sync::RwLock;

use manager::{FloeManager, FloeResult};

const EXAMPLE_FLOE: &str = "../example_provider/target/release/example_provider";

pub(crate) async fn spawn(state: GlacierState) -> FloeResult<()> {
    let shared_state = Arc::new(RwLock::new(state));

    let manager = FloeManager::new(EXAMPLE_FLOE, shared_state.clone()).await?;

    match manager.ping().await {
        Ok(response) => println!("Ping successful: {:?}", response),
        Err(e) => println!("Ping failed: {}", e),
    }

    Ok(())
}
