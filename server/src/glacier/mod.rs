mod manager;
pub mod state;

use std::error::Error;

pub use state::GlacierState;

use manager::Glacier;

const EXAMPLE_FLOE: &str = "../example_provider/target/release/example_provider";
const EXAMPLE_FLOE2: &str = "../example_provider/target/release/example_provider_1";

pub(crate) async fn spawn(state: GlacierState) -> Result<(), Box<dyn Error>> {
    let _manager = Glacier::new(vec![EXAMPLE_FLOE, EXAMPLE_FLOE2], state).await?;
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
