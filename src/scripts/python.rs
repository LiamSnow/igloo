use std::sync::Arc;

use tokio::sync::oneshot;

use crate::state::IglooState;

pub fn spawn(
    _script_name: String,
    id: u32,
    state: Arc<IglooState>,
    _uid: Option<usize>,
    _args: Vec<String>,
    _cancel_rx: oneshot::Receiver<()>,
    _filename: String,
) {
    tokio::spawn(async move {
        //TODO

        // clean up
        let mut script_states = state.scripts.states.lock().await;
        script_states.current.remove(&id);
    });
}
