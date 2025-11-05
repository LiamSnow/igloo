use super::{EntityRegister, add_device_class, add_entity_category, add_icon, add_unit};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    DESELECT_ENTITY, END_TRANSACTION, Real, WRITE_REAL, floe::FloeWriterDefault,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesNumberResponse {
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
                crate::model::EntityType::Number,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        // add_number_mode(writer, self.mode()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        // add_f32_bounds(writer, self.min_value, self.max_value, Some(self.step)).await?;
        add_unit(writer, self.unit_of_measurement).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::NumberStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.real(&(self.state as f64)).await
    }
}

// impl api::NumberMode {
//     pub fn as_igloo(&self) -> NumberMode {
//         match self {
//             api::NumberMode::Auto => NumberMode::Auto,
//             api::NumberMode::Box => NumberMode::Box,
//             api::NumberMode::Slider => NumberMode::Slider,
//         }
//     }
// }

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::NumberCommandRequest { key, state: 0.0 };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_REAL => {
                let state: Real = borsh::from_slice(&payload)?;
                req.state = state as f32;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Number got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::NumberCommandRequest, &req)
        .await
}
