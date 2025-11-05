use async_trait::async_trait;
use igloo_interface::{
    AlarmState, DESELECT_ENTITY, END_TRANSACTION, Text, WRITE_ALARM_STATE, WRITE_TEXT,
    floe::FloeWriterDefault,
};

use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesAlarmControlPanelResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(
                writer,
                &self.name,
                self.key,
                crate::model::EntityType::AlarmControlPanel,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
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

#[async_trait]
impl EntityUpdate for api::AlarmControlPanelStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.alarm_state(&self.state().as_igloo()).await
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
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::AlarmControlPanelCommandRequest {
        key,
        command: api::AlarmControlPanelStateCommand::AlarmControlPanelDisarm.into(),
        code: String::new(),
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_TEXT => {
                let code: Text = borsh::from_slice(&payload)?;
                req.code = code;
            }

            WRITE_ALARM_STATE => {
                let state: AlarmState = borsh::from_slice(&payload)?;
                req.command = alarm_state_to_command(&state).into();
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!(
                    "AlarmControlPanel got unexpected command {cmd_id} during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::AlarmControlPanelCommandRequest, &req)
        .await
}
