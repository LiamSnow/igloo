use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, span, Level};

use crate::{cli::model::Cli, elements, entity::EntityState, state::IglooState};

/// Spawn state and back_cmd tasks once IglooState is ready
pub(crate) fn init(
    istate_rx: oneshot::Receiver<Arc<IglooState>>,
) -> (
    mpsc::Sender<(usize, String, EntityState)>,
    mpsc::Sender<Cli>,
) {
    let (on_change_tx, on_change_rx) = mpsc::channel(10); //FIXME size?
    let (back_cmd_tx, back_cmd_rx) = mpsc::channel::<Cli>(5);
    tokio::spawn(async move {
        let istate = istate_rx.await.unwrap();
        tokio::spawn(state_task(on_change_rx, istate.clone()));
        tokio::spawn(back_cmd_task(back_cmd_rx, istate));
    });

    (on_change_tx, back_cmd_tx)
}

/// Receive state changes from devices (did, entity_name, value)
///  -> Save them to dev states in IglooState
///  -> Notify elements
async fn state_task(
    mut on_change_rx: mpsc::Receiver<(usize, String, EntityState)>,
    istate: Arc<IglooState>,
) {
    let span = span!(Level::INFO, "Devices State Update Task");
    let _enter = span.enter();
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

/// Execute arbitrary commands given from devices (IE periodic task)
/// This is probably a security issue......
async fn back_cmd_task(mut back_cmd_rx: mpsc::Receiver<Cli>, istate: Arc<IglooState>) {
    let span = span!(Level::INFO, "Devices Back Command Task");
    let _enter = span.enter();
    info!("running");

    while let Some(cmd) = back_cmd_rx.recv().await {
        if let Err(e) = cmd.dispatch(&istate, None, true).await {
            error!("{}", serde_json::to_string(&e).unwrap());
        }
    }
}
