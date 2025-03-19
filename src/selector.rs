use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;

use crate::{
    device::DeviceIDLut, state::IglooState, entity::{EntityCommand, TargetedEntityCommand}
};

#[derive(Error, Debug, Serialize)]
pub enum SelectorError {
    #[error("scope selector must be `all`, ZONE, ZONE.DEVICE, or ZONE.DEVICE.SUBDEVICE")]
    BadSelector,
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
    #[error("unknown device `{0}.{1}`")]
    UnknownDevice(String, String),
}

#[derive(Error, Debug, Serialize)]
pub enum DeviceChannelError {
    #[error("full")]
    Full,
    #[error("closed")]
    Closed,
}

impl From<TrySendError<TargetedEntityCommand>> for DeviceChannelError {
    fn from(value: TrySendError<TargetedEntityCommand>) -> Self {
        match value {
            TrySendError::Full(_) => Self::Full,
            TrySendError::Closed(_) => Self::Closed,
        }
    }
}

#[derive(Clone)]
pub enum Selection {
    All,
    /// zid, start_did, end_did
    Zone(usize, usize, usize),
    /// zid, did
    Device(usize, usize),
    /// zid, did, entity_name
    Entity(usize, usize, String),
}

impl Selection {
    pub fn from_str(lut: &DeviceIDLut, s: &str) -> Result<Self, SelectorError> {
        Self::new(lut, &SelectionString::new(s)?)
    }

    pub fn new(lut: &DeviceIDLut, sel_str: &SelectionString) -> Result<Self, SelectorError> {
        match sel_str {
            SelectionString::All => Ok(Selection::All),
            SelectionString::Zone(zone_name) => {
                let zid = lut
                    .zid
                    .get(*zone_name)
                    .ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let (start_did, end_did) = lut.did_range.get(*zid).unwrap();
                Ok(Self::Zone(*zid, *start_did, *end_did))
            }
            SelectionString::Device(zone_name, dev_name) => {
                let zid = lut
                    .zid
                    .get(*zone_name)
                    .ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let dev_lut = lut.did.get(*zid).unwrap();
                let did = dev_lut.get(*dev_name).ok_or(SelectorError::UnknownDevice(
                    zone_name.to_string(),
                    dev_name.to_string(),
                ))?;
                Ok(Self::Device(*zid, *did))
            }
            SelectionString::Entity(zone_name, dev_name, entity_name) => {
                let zid = lut
                    .zid
                    .get(*zone_name)
                    .ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let dev_lut = lut.did.get(*zid).unwrap();
                let did = dev_lut.get(*dev_name).ok_or(SelectorError::UnknownDevice(
                    zone_name.to_string(),
                    dev_name.to_string(),
                ))?;
                Ok(Self::Entity(*zid, *did, entity_name.to_string()))
            }
        }
    }

    pub fn is_all(self) -> bool {
        matches!(self, Self::All)
    }

    pub fn get_zone(self) -> Option<(usize, usize, usize)> {
        match self {
            Self::Zone(zid, start_did, end_did) => Some((zid, start_did, end_did)),
            _ => None,
        }
    }

    pub fn get_device(self) -> Option<(usize, usize)> {
        match self {
            Self::Device(zid, did) => Some((zid, did)),
            _ => None,
        }
    }

    pub fn get_entity(self) -> Option<(usize, usize, String)> {
        match self {
            Self::Entity(zid, did, entity_name) => Some((zid, did, entity_name)),
            _ => None,
        }
    }

    pub fn execute(
        &self,
        state: &Arc<IglooState>,
        cmd: EntityCommand,
    ) -> Result<(), DeviceChannelError> {
        match self {
            Self::All => {
                for dev_chan in &state.devices.channels {
                    dev_chan.try_send(TargetedEntityCommand {
                        cmd: cmd.clone(),
                        entity_name: None,
                    })?;
                }
            }
            Self::Zone(_, start_did, end_did) => {
                for dev_chan in &state.devices.channels[*start_did..=*end_did] {
                    dev_chan.try_send(TargetedEntityCommand {
                        cmd: cmd.clone(),
                        entity_name: None,
                    })?;
                }
            }
            Self::Device(_, did) => {
                let dev_chan = state.devices.channels.get(*did).unwrap();
                dev_chan.try_send(TargetedEntityCommand {
                    cmd: cmd.clone(),
                    entity_name: None,
                })?;
            }
            Self::Entity(_, did, entity_name) => {
                let dev_chan = state.devices.channels.get(*did).unwrap();
                dev_chan.try_send(TargetedEntityCommand {
                    cmd: cmd.clone(),
                    entity_name: Some(entity_name.to_string()),
                })?;
            }
        }
        Ok(())
    }

    pub fn rank(&self) -> u8 {
        match self {
            Self::All => 3,
            Self::Zone(..) => 2,
            Self::Device(..) => 1,
            Self::Entity(..) => 0,
        }
    }

    pub fn get_zid(&self) -> Option<usize> {
        match self {
            Self::All => None,
            Self::Zone(zid, _, _) => Some(*zid),
            Self::Device(zid, _) => Some(*zid),
            Self::Entity(zid, _, _) => Some(*zid),
        }
    }

    pub fn get_did(&self) -> Option<usize> {
        match self {
            Self::All => None,
            Self::Zone(..) => None,
            Self::Device(_, did) => Some(*did),
            Self::Entity(_, did, _) => Some(*did),
        }
    }

    pub fn get_entity_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(..) => None,
            Self::Device(_, _) => None,
            Self::Entity(_, _, entity_name) => Some(&entity_name),
        }
    }

    pub fn collides(&self, other: &Self) -> bool {
        if other.rank() > self.rank() {
            return other.collides(self);
        }
        match self {
            Self::All => true,
            Self::Zone(zid, _, _) => *zid == other.get_zid().unwrap(),
            Self::Device(zid, did) => {
                *zid == other.get_zid().unwrap() && *did == other.get_did().unwrap()
            }
            Self::Entity(zid, did, entity_name) => {
                *zid == other.get_zid().unwrap()
                    && *did == other.get_did().unwrap()
                    && entity_name == other.get_entity_name().unwrap()
            }
        }
    }

    //TODO make more efficient?
    pub fn collides_with_any(&self, others: &Vec<Self>) -> bool {
        if matches!(self, Self::All) {
            return true;
        }
        for other in others {
            if self.collides(other) {
                return true
            }
        }
        false
    }

    //TODO make more efficient?
    pub fn any_collides_with_any(a: &Vec<Self>, other: &Vec<Self>) -> bool {
        for a in a {
            if a.collides_with_any(other) {
                return true
            }
        }
        false
    }

    /// This is REALLY slow, use sparingly
    pub fn to_str(&self, lut: &DeviceIDLut) -> String {
        if let Some(zid) = self.get_zid() {
            let mut res = match lut.zid.iter().find(|(_, v)| **v == zid) {
                Some((k, _)) => k.to_string(),
                None => "ERROR".to_string(),
            };

            if let Some(did) = self.get_did() {
                let did_lut = lut.did.get(zid).unwrap();
                res.push('.');
                res.push_str(match did_lut.iter().find(|(_, v)| **v == did) {
                    Some((k, _)) => k,
                    None => "ERROR",
                });

                if let Some(entity_name) = self.get_entity_name() {
                    res.push('.');
                    res.push_str(&entity_name);
                }
            }

            res
        } else {
            "all".to_string()
        }
    }
}

pub enum SelectionString<'a> {
    All,
    /// zone_name
    Zone(&'a str),
    /// zone_name, dev_name
    Device(&'a str, &'a str),
    /// zone_name, dev_name, entity_name
    Entity(&'a str, &'a str, &'a str),
}

impl<'a> SelectionString<'a> {
    pub fn new(selection_str: &'a str) -> Result<Self, SelectorError> {
        if selection_str == "all" {
            return Ok(Self::All);
        }

        let parts: Vec<&str> = selection_str.split(".").collect();
        if parts.len() < 1 || parts.len() > 3 {
            return Err(SelectorError::BadSelector);
        }

        let zone_name = parts.get(0).unwrap();

        if let Some(dev_name) = parts.get(1) {
            if let Some(entity_name) = parts.get(2) {
                Ok(Self::Entity(zone_name, dev_name, entity_name))
            } else {
                Ok(Self::Device(zone_name, dev_name))
            }
        } else {
            Ok(Self::Zone(zone_name))
        }
    }

    pub fn get_zone_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(zone_name) => Some(zone_name),
            Self::Device(zone_name, _) => Some(zone_name),
            Self::Entity(zone_name, _, _) => Some(zone_name),
        }
    }

    pub fn get_dev_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(..) => None,
            Self::Device(_, dev_name) => Some(dev_name),
            Self::Entity(_, dev_name, _) => Some(dev_name),
        }
    }

    pub fn get_entity_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(..) => None,
            Self::Device(_, _) => None,
            Self::Entity(_, _, entity_name) => Some(&entity_name),
        }
    }
}
