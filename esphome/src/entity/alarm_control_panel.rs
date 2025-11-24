use igloo_interface::{Component, AlarmState};
use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};

impl EntityRegister for api::ListEntitiesAlarmControlPanelResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
    }
}

impl api::AlarmControlPanelState {
    pub fn as_igloo(&self) -> AlarmState {
        match self {
            api::AlarmControlPanelState::AlarmStateDisarmed => AlarmState::Disarmed,
            api::AlarmControlPanelState::AlarmStateArmedHome => AlarmState::ArmedHome,
            api::AlarmControlPanelState::AlarmStateArmedAway => AlarmState::ArmedAway,
            api::AlarmControlPanelState::AlarmStateArmedNight => AlarmState::ArmedNight,
            api::AlarmControlPanelState::AlarmStateArmedVacation => AlarmState::ArmedVacation,
            api::AlarmControlPanelState::AlarmStateArmedCustomBypass => AlarmState::ArmedUnknown,
            api::AlarmControlPanelState::AlarmStatePending => AlarmState::Pending,
            api::AlarmControlPanelState::AlarmStateArming => AlarmState::Arming,
            api::AlarmControlPanelState::AlarmStateDisarming => AlarmState::Disarming,
            api::AlarmControlPanelState::AlarmStateTriggered => AlarmState::Triggered,
        }
    }
}

impl EntityUpdate for api::AlarmControlPanelStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::AlarmState(self.state().as_igloo())]
    }
}

fn alarm_state_to_command(state: &AlarmState) -> api::AlarmControlPanelStateCommand {
    match state {
        AlarmState::Disarmed => api::AlarmControlPanelStateCommand::AlarmControlPanelDisarm,
        AlarmState::ArmedHome => api::AlarmControlPanelStateCommand::AlarmControlPanelArmHome,
        AlarmState::ArmedAway => api::AlarmControlPanelStateCommand::AlarmControlPanelArmAway,
        AlarmState::ArmedNight => api::AlarmControlPanelStateCommand::AlarmControlPanelArmNight,
        AlarmState::ArmedVacation => {
            api::AlarmControlPanelStateCommand::AlarmControlPanelArmVacation
        }
        AlarmState::ArmedUnknown => {
            api::AlarmControlPanelStateCommand::AlarmControlPanelArmCustomBypass
        }
        AlarmState::Pending => api::AlarmControlPanelStateCommand::AlarmControlPanelDisarm,
        AlarmState::Triggered => api::AlarmControlPanelStateCommand::AlarmControlPanelTrigger,
        AlarmState::Arming => api::AlarmControlPanelStateCommand::AlarmControlPanelArmHome,
        AlarmState::Disarming => api::AlarmControlPanelStateCommand::AlarmControlPanelDisarm,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::AlarmControlPanelCommandRequest {
        key,
        command: api::AlarmControlPanelStateCommand::AlarmControlPanelDisarm.into(),
        code: String::new(),
    };

    for comp in comps {
        use Component::*;
        match comp {
            Text(code) => {
                req.code = code;
            }

            AlarmState(state) => {
                req.command = alarm_state_to_command(&state).into();
            }

            comp => {
                println!(
                    "AlarmControlPanel got unexpected component '{comp:?}' during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::AlarmControlPanelCommandRequest, &req)
        .await
}
