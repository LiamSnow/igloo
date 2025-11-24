use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, LockState};

impl EntityRegister for api::ListEntitiesLockResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(3);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps.push(Component::Text(self.code_format));
        comps
    }
}

impl api::LockState {
    fn as_igloo(&self) -> LockState {
        match self {
            api::LockState::None => LockState::Unknown,
            api::LockState::Locked => LockState::Locked,
            api::LockState::Unlocked => LockState::Unlocked,
            api::LockState::Jammed => LockState::Jammed,
            api::LockState::Locking => LockState::Locking,
            api::LockState::Unlocking => LockState::Unlocking,
        }
    }
}

impl EntityUpdate for api::LockStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::LockState(self.state().as_igloo())]
    }
}

fn lock_state_to_command(state: &LockState) -> api::LockCommand {
    match state {
        LockState::Locked => api::LockCommand::LockLock,
        LockState::Unlocked => api::LockCommand::LockUnlock,
        LockState::Jammed => api::LockCommand::LockUnlock,
        LockState::Locking => api::LockCommand::LockLock,
        LockState::Unlocking => api::LockCommand::LockUnlock,
        LockState::Unknown => api::LockCommand::LockUnlock,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::LockCommandRequest {
        key,
        command: api::LockCommand::LockLock.into(),
        has_code: false,
        code: String::new(),
    };

    for comp in comps {
        use Component::*;
        match comp {
            Text(code) => {
                req.has_code = true;
                req.code = code;
            }

            LockState(state) => {
                req.command = lock_state_to_command(&state).into();
            }

            comp => {
                println!("Lock got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::LockCommandRequest, &req).await
}
