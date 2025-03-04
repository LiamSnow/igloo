use std::sync::Arc;

use tokio::sync::oneshot;

use crate::stack::IglooStack;

pub fn spawn(
    _script_name: String,
    id: u32,
    stack: Arc<IglooStack>,
    _uid: usize,
    _args: Vec<String>,
    _cancel_rx: oneshot::Receiver<()>,
    _filename: String,
) {
    tokio::spawn(async move {
        //TODO

        // clean up
        let mut script_states = stack.script_states.lock().await;
        script_states.current.remove(&id);
    });
}
