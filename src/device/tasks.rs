use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use tracing::{info, span, Level};

use crate::{elements, entity::EntityState, state::IglooState};

/// Spawn state task
pub(crate) fn init() -> (
    mpsc::Sender<(usize, String, EntityState)>,
    oneshot::Sender<Arc<IglooState>>,
) {
    let (on_change_tx, on_change_rx) = mpsc::channel(10); //FIXME size?
    let (istate_tx, istate_rx) = oneshot::channel();
    tokio::spawn(state_task(on_change_rx, istate_rx));
    (on_change_tx, istate_tx)
}

/// Receive state changes from devices (did, entity_name, value)
///  -> Save them to dev states in IglooState
///  -> Notify elements
async fn state_task(
    mut on_change_rx: mpsc::Receiver<(usize, String, EntityState)>,
    istate_rx: oneshot::Receiver<Arc<IglooState>>,
) {
    let span = span!(Level::INFO, "Devices State Update Task");
    let _enter = span.enter();

    //wait until istate is ready
    let istate = istate_rx.await.unwrap();
    info!("running");

    //TODO group changes?
    while let Some((did, entity_name, value)) = on_change_rx.recv().await {
        //push to states
        {
            let mut states = istate.devices.states.lock().await;
            states[did].insert(entity_name.clone(), value.clone());
        }

        //update elements
        elements::state::on_device_update(&istate, did, &entity_name, &value).await;
    }
}
