use std::sync::Arc;

use tokio::sync::oneshot;
use tracing::{info, span, Level};

use crate::{scripts::{send_change_to_ui, ScriptStateChange}, state::IglooState};

pub fn spawn(
    script_name: String,
    id: u32,
    state: Arc<IglooState>,
    uid: Option<usize>,
    args: Vec<String>,
    _cancel_rx: oneshot::Receiver<()>,
    _filename: String,
) {
    tokio::spawn(async move {
        let span = span!(Level::INFO, "Python Script", script_name, id);
        let _enter = span.enter();
        info!("running uid={:#?}, args={:#?}", uid, args);

        //TODO

        // clean up
        let mut script_states = state.scripts.states.lock().await;
        script_states.current.remove(&id);
        send_change_to_ui(&state, ScriptStateChange::Remove(id));
    });
}
