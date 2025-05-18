use std::sync::Arc;

use crate::{cli::error::DispatchError, device::ids::DeviceIDSelection, state::IglooState};

use super::EntityCommand;







pub fn dispatch(
    sel_str: String,
    sel: DeviceIDSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, EntityCommand::Trigger)
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}
