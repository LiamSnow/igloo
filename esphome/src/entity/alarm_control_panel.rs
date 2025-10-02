use async_trait::async_trait;
use igloo_interface::{AlarmState, FloeWriterDefault};

use crate::{api, entity::EntityUpdate};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesAlarmControlPanelResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::AlarmControlPanel)
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
