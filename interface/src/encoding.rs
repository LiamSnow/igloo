use std::mem;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use smallvec::SmallVec;
use uuid::Uuid;

use crate::{
    Component, ComponentUpdate, DeviceRegisteredPayload, FLOE_CUSTOM_COMMAND_ERROR, FLOE_LOG,
    FLOE_REGISTER_DEVICE, FLOE_UPDATES, FLOE_YEAH_BRO_IM_UP, FloeCommand, IGLOO_DEVICE_REGISTERED,
    IGLOO_EXECUTE_CUSTOM_COMMAND, IGLOO_HEY_BUDDY_YOU_AWAKE, IGLOO_REQUEST_UPDATES,
    IglooCodecError, IglooCommand,
};

pub trait IglooCodable {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError>;
    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError>
    where
        Self: Sized;
}

impl IglooCodable for IglooCommand {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
        match self {
            IglooCommand::HeyBuddyYouAwake(version) => {
                buf.put_u8(IGLOO_HEY_BUDDY_YOU_AWAKE);
                buf.put_u8(*version);
            }
            IglooCommand::DeviceRegistered(payload) => {
                buf.put_u8(IGLOO_DEVICE_REGISTERED);
                payload.encode(buf)?;
            }
            IglooCommand::RequestUpdates(update) => {
                buf.put_u8(IGLOO_REQUEST_UPDATES);
                update.encode(buf)?;
            }
            IglooCommand::ExecuteCustomCommand(cmd_id, payload_bytes) => {
                buf.put_u8(IGLOO_EXECUTE_CUSTOM_COMMAND);
                buf.put_u8(*cmd_id);
                buf.put_slice(payload_bytes);
            }
        }
        Ok(())
    }

    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
        if buf.is_empty() {
            return Err(IglooCodecError::InvalidMessage);
        }

        let cmd_byte = buf.get_u8();
        match cmd_byte {
            IGLOO_HEY_BUDDY_YOU_AWAKE => {
                let version = buf.get_u8();
                Ok(IglooCommand::HeyBuddyYouAwake(version))
            }
            IGLOO_DEVICE_REGISTERED => {
                let payload = DeviceRegisteredPayload::decode(buf)?;
                Ok(IglooCommand::DeviceRegistered(payload))
            }
            IGLOO_REQUEST_UPDATES => {
                let update = ComponentUpdate::decode(buf)?;
                Ok(IglooCommand::RequestUpdates(update))
            }
            IGLOO_EXECUTE_CUSTOM_COMMAND => {
                let cmd_id = buf.get_u8();
                Ok(IglooCommand::ExecuteCustomCommand(cmd_id, mem::take(buf)))
            }
            _ => Err(IglooCodecError::UnknownCommand(cmd_byte)),
        }
    }
}

impl IglooCodable for FloeCommand {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
        match self {
            FloeCommand::YeahBroImUp(version) => {
                buf.put_u8(FLOE_YEAH_BRO_IM_UP);
                buf.put_u8(*version);
            }
            FloeCommand::RegisterDevice(payload) => {
                buf.put_u8(FLOE_REGISTER_DEVICE);
                buf.put_slice(payload.id.as_bytes());
                buf.put_u16_le(payload.initial_name.len() as u16);
                buf.put_slice(payload.initial_name.as_bytes());
                buf.put_u16_le(payload.entity_names.len() as u16);
                for name in &payload.entity_names {
                    buf.put_u16_le(name.len() as u16);
                    buf.put_slice(name.as_bytes());
                }
            }
            FloeCommand::Updates(update) => {
                buf.put_u8(FLOE_UPDATES);
                update.encode(buf)?;
            }
            FloeCommand::CustomCommandError(id, msg) => {
                buf.put_u8(FLOE_CUSTOM_COMMAND_ERROR);
                buf.put_u8(*id);
                buf.put_u16_le(msg.len() as u16);
                buf.put_slice(msg.as_bytes());
            }
            FloeCommand::Log(msg) => {
                buf.put_u8(FLOE_LOG);
                buf.put_u16_le(msg.len() as u16);
                buf.put_slice(msg.as_bytes());
            }
        }
        Ok(())
    }

    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError>
    where
        Self: Sized,
    {
        if buf.is_empty() {
            return Err(IglooCodecError::InvalidMessage);
        }

        let cmd_byte = buf.get_u8();
        match cmd_byte {
            FLOE_YEAH_BRO_IM_UP => {
                let version = buf.get_u8();
                Ok(FloeCommand::YeahBroImUp(version))
            }
            FLOE_REGISTER_DEVICE => {
                if buf.remaining() < 16 {
                    return Err(IglooCodecError::InvalidMessage);
                }
                let uuid_bytes = buf.copy_to_bytes(16);
                let id =
                    Uuid::from_slice(&uuid_bytes).map_err(|_| IglooCodecError::InvalidMessage)?;

                let name_len = buf.get_u16_le() as usize;
                if buf.remaining() < name_len {
                    return Err(IglooCodecError::InvalidMessage);
                }
                let name_bytes = buf.copy_to_bytes(name_len);
                let initial_name = String::from_utf8(name_bytes.to_vec())
                    .map_err(|_| IglooCodecError::InvalidUtf8)?;

                let entity_count = buf.get_u16_le() as usize;
                let mut entity_names = Vec::with_capacity(entity_count);
                for _ in 0..entity_count {
                    let entity_name_len = buf.get_u16_le() as usize;
                    if buf.remaining() < entity_name_len {
                        return Err(IglooCodecError::InvalidMessage);
                    }
                    let entity_name_bytes = buf.copy_to_bytes(entity_name_len);
                    let entity_name = String::from_utf8(entity_name_bytes.to_vec())
                        .map_err(|_| IglooCodecError::InvalidUtf8)?;
                    entity_names.push(entity_name);
                }

                Ok(FloeCommand::RegisterDevice(crate::RegisterDevicePayload {
                    id,
                    initial_name,
                    entity_names,
                }))
            }
            FLOE_UPDATES => {
                let update = ComponentUpdate::decode(buf)?;
                Ok(FloeCommand::Updates(update))
            }
            FLOE_CUSTOM_COMMAND_ERROR => {
                let cmd_id = buf.get_u8();
                let msg_len = buf.get_u16_le() as usize;
                if buf.remaining() < msg_len {
                    return Err(IglooCodecError::InvalidMessage);
                }
                let msg_bytes = buf.copy_to_bytes(msg_len);
                let msg = String::from_utf8(msg_bytes.to_vec())
                    .map_err(|_| IglooCodecError::InvalidUtf8)?;
                Ok(FloeCommand::CustomCommandError(cmd_id, msg))
            }
            FLOE_LOG => {
                let msg_len = buf.get_u16_le() as usize;
                if buf.remaining() < msg_len {
                    return Err(IglooCodecError::InvalidMessage);
                }
                let msg_bytes = buf.copy_to_bytes(msg_len);
                let msg = String::from_utf8(msg_bytes.to_vec())
                    .map_err(|_| IglooCodecError::InvalidUtf8)?;
                Ok(FloeCommand::Log(msg))
            }
            _ => Err(IglooCodecError::UnknownCommand(cmd_byte)),
        }
    }
}

impl FloeCommand {
    pub fn est_encoded_len(&self) -> usize {
        match self {
            FloeCommand::YeahBroImUp(_) => 2,
            FloeCommand::Updates(update) => {
                // cmd + device(2) entity(2) + count + [components..]
                6 + update.values.len() * 32
            }
            FloeCommand::RegisterDevice(payload) => {
                // md + uuid(16) + [name] + [entity names]
                17 + payload.initial_name.len()
                    + 2
                    + payload
                        .entity_names
                        .iter()
                        .map(|n| n.len() + 2)
                        .sum::<usize>()
            }
            FloeCommand::CustomCommandError(_, msg) => 3 + msg.len(),
            FloeCommand::Log(msg) => 3 + msg.len(),
        }
    }
}

impl IglooCodable for ComponentUpdate {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
        buf.put_u16_le(self.device);
        buf.put_u16_le(self.entity);
        buf.put_u8(self.values.len() as u8);

        for component in &self.values {
            component.encode(buf)?;
        }
        Ok(())
    }

    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
        let device = buf.get_u16_le();
        let entity = buf.get_u16_le();
        let value_count = buf.get_u8() as usize;

        let mut values = SmallVec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(Component::decode(buf)?);
        }

        Ok(ComponentUpdate {
            device,
            entity,
            values,
        })
    }
}

impl IglooCodable for DeviceRegisteredPayload {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
        buf.put_slice(self.id.as_bytes());

        buf.put_u16_le(self.device_ref);

        buf.put_u16_le(self.entity_refs.len() as u16);

        for (name, entity_ref) in &self.entity_refs {
            buf.put_u16_le(name.len() as u16);
            buf.put_slice(name.as_bytes());
            buf.put_u16_le(*entity_ref);
        }

        Ok(())
    }

    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
        let id = Uuid::from_slice(&buf[..16]).map_err(|_| IglooCodecError::InvalidMessage)?;
        buf.advance(16);
        let device_ref = buf.get_u16_le();

        let entity_count = buf.get_u16_le() as usize;
        let mut entity_refs = Vec::with_capacity(entity_count);

        for _ in 0..entity_count {
            let name_len = buf.get_u16_le() as usize;
            let name_bytes = buf.copy_to_bytes(name_len);
            let name =
                String::from_utf8(name_bytes.to_vec()).map_err(|_| IglooCodecError::InvalidUtf8)?;
            let entity_ref = buf.get_u16_le();
            entity_refs.push((name, entity_ref));
        }
        Ok(DeviceRegisteredPayload {
            id,
            device_ref,
            entity_refs,
        })
    }
}

impl IglooCodable for String {
    #[inline(always)]
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
        buf.put_u32_le(self.len() as u32);
        buf.put_slice(self.as_bytes());
        Ok(())
    }

    #[inline(always)]
    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
        let len = buf.get_u32_le() as usize;
        if buf.remaining() < len {
            return Err(IglooCodecError::InvalidMessage);
        }
        let text_bytes = buf.copy_to_bytes(len);
        let text =
            String::from_utf8(text_bytes.to_vec()).map_err(|_| IglooCodecError::InvalidUtf8)?;
        Ok(text)
    }
}

impl IglooCodable for bool {
    #[inline(always)]
    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
        buf.put_u8(*self as u8);
        Ok(())
    }

    #[inline(always)]
    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
        Ok(buf.get_u8() != 0)
    }
}
