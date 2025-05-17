use std::collections::HashMap;

use crate::{
    entity::{EntityCommand, TargetedEntityCommand},
    state::IglooState,
};

use super::{
    error::{DeviceChannelError, DeviceSelectorError},
    providers::DeviceConfig,
    selection_str::SelectionString,
};

use std::sync::Arc;

#[derive(Default)]
pub struct DeviceIDLut {
    pub num_devs: usize,
    pub num_zones: usize,
    /// zid (zone ID) -> start_did, end_did
    pub did_range: Vec<(usize, usize)>,
    /// zid, dev_name -> did (device ID)
    pub did: Vec<HashMap<String, usize>>,
    /// zone_name -> zid
    pub zid: HashMap<String, usize>,
}

#[derive(Clone, Debug)]
pub enum DeviceSelection {
    All,
    /// zid, start_did, end_did
    /// All Devices in a Zone
    /// Note that zone did's are contiguous, so iterating from
    /// start_did to end_did will cover all the devices in the zone
    Zone(usize, usize, usize),
    /// zid, did
    /// Targets all entities in a Device
    Device(usize, usize),
    /// zid, did, entity_name
    /// One specific Entity inside a Device
    Entity(usize, usize, String),
}

impl DeviceIDLut {
    pub fn init(
        devices: HashMap<String, HashMap<String, DeviceConfig>>,
    ) -> (Self, Vec<DeviceConfig>, Vec<String>) {
        //make lut
        let (mut next_did, mut next_zid) = (0, 0);
        let mut lut = DeviceIDLut::default();
        let (mut dev_cfgs, mut dev_sels) = (Vec::new(), Vec::new());
        for (zone_name, devs) in devices {
            let start_did = next_did;
            let mut did_lut = HashMap::new();
            for (dev_name, dev_cfg) in devs {
                did_lut.insert(dev_name.clone(), next_did);
                dev_cfgs.push(dev_cfg);
                dev_sels.push(format!("{zone_name}.{dev_name}"));
                next_did += 1;
            }
            lut.did.push(did_lut);
            lut.did_range.push((start_did, next_did - 1));
            lut.zid.insert(zone_name, next_zid);
            next_zid += 1;
        }
        lut.num_devs = next_did;
        lut.num_zones = next_zid;
        (lut, dev_cfgs, dev_sels)
    }
}

impl DeviceSelection {
    pub fn from_str(lut: &DeviceIDLut, s: &str) -> Result<Self, DeviceSelectorError> {
        Self::new(lut, &SelectionString::new(s)?)
    }

    pub fn new(lut: &DeviceIDLut, sel_str: &SelectionString) -> Result<Self, DeviceSelectorError> {
        match sel_str {
            SelectionString::All => Ok(DeviceSelection::All),
            SelectionString::Zone(zone_name) => {
                let zid = lut
                    .zid
                    .get(*zone_name)
                    .ok_or(DeviceSelectorError::UnknownZone(zone_name.to_string()))?;
                let (start_did, end_did) = lut.did_range.get(*zid).unwrap();
                Ok(Self::Zone(*zid, *start_did, *end_did))
            }
            SelectionString::Device(zone_name, dev_name) => {
                let zid = lut
                    .zid
                    .get(*zone_name)
                    .ok_or(DeviceSelectorError::UnknownZone(zone_name.to_string()))?;
                let dev_lut = lut.did.get(*zid).unwrap();
                let did = dev_lut
                    .get(*dev_name)
                    .ok_or(DeviceSelectorError::UnknownDevice(
                        zone_name.to_string(),
                        dev_name.to_string(),
                    ))?;
                Ok(Self::Device(*zid, *did))
            }
            SelectionString::Entity(zone_name, dev_name, entity_name) => {
                let zid = lut
                    .zid
                    .get(*zone_name)
                    .ok_or(DeviceSelectorError::UnknownZone(zone_name.to_string()))?;
                let dev_lut = lut.did.get(*zid).unwrap();
                let did = dev_lut
                    .get(*dev_name)
                    .ok_or(DeviceSelectorError::UnknownDevice(
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
                return true;
            }
        }
        false
    }

    //TODO make more efficient?
    pub fn any_collides_with_any(a: &Vec<Self>, other: &Vec<Self>) -> bool {
        for a in a {
            if a.collides_with_any(other) {
                return true;
            }
        }
        false
    }

    /// Converts back to original selection string
    /// This is REALLY slow, use sparingly (reverse lookup)
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

    pub fn get_did_range(&self, lut: &DeviceIDLut) -> (usize, usize) {
        match self {
            DeviceSelection::All => (0 as usize, lut.did.len() - 1),
            DeviceSelection::Zone(_, start_did, end_did) => (*start_did, *end_did),
            DeviceSelection::Device(_, did) | DeviceSelection::Entity(_, did, _) => (*did, *did),
        }
    }
}
