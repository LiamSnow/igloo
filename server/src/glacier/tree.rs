use std::error::Error;

use igloo_interface::{ComponentType, FloeWriterDefault, MAX_SUPPORTED_COMPONENT};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use smallvec::SmallVec;
use tokio::fs;

use crate::glacier::{entity::HasComponent, file};

use super::entity::Entity;

pub const ZONES_FILE: &str = "zones.ini";
pub const DEVICES_FILE: &str = "devices.ini";

/*
TODO we need to add rigerous testing to enforce the following rules:

@ floe added
 - must be added to `floes`, `floe_idx_lut`

@ device registered
 - must be added to `devices`, `device_idx_lut`, `device_names` (if new)
 - must update all applicable [Zone].idxs

@ zone added
 - must be added to `zones`, `zone_idx_lut`

@ device added to zone, removed from zone
 - Zone.idxs must updated
*/

/// ID is persistent
/// idx is ephemeral
#[derive(Debug, Default)]
pub struct DeviceTree {
    /// Zone idx -> zone
    zones: Vec<Zone>,
    /// Zone ID -> idx
    zone_idx_lut: FxHashMap<String, u16>,

    /// Floe idx -> floe
    floes: Vec<Floe>,
    /// Floe ID -> floe idx
    floe_idx_lut: FxHashMap<String, u16>,

    /// Floe ID + device ID -> device name
    /// removed once Floe is registered
    name_queue: FxHashMap<String, FxHashMap<String, String>>,
}

#[derive(Debug)]
pub struct Floe {
    id: String,
    pub writer: FloeWriterDefault,
    max_supported_component: u16,
    /// device idx -> device
    devices: Vec<Device>,
    /// device ID -> device idx
    device_idx_lut: FxHashMap<String, u16>,
    /// device ID -> device name
    /// removed once Device is registered
    name_queue: FxHashMap<String, String>,
}

#[derive(Debug)]
pub struct Zone {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) disabled: bool,
    /// floe id + device ids
    pub(super) devices: FxHashMap<String, FxHashSet<String>>,
    /// floe idx + device idx
    pub(super) idxs: SmallVec<[(u16, u16); 20]>,
}

#[derive(Debug, Default)]
pub struct Device {
    id: String,
    name: String,
    presense: Presense,
    /// entity idx -> entity
    entities: Entities,
    /// entity name -> idx
    entity_idx_lut: FxHashMap<String, usize>,
}

pub type Entities = SmallVec<[Entity; 16]>;

#[derive(Debug, Default)]
pub struct Presense([u32; MAX_SUPPORTED_COMPONENT.div_ceil(32) as usize]);

impl DeviceTree {
    pub async fn load() -> Result<Self, Box<dyn Error>> {
        let mut tree = DeviceTree::default();

        let content = fs::read_to_string(DEVICES_FILE).await?;
        tree.name_queue = file::parse_devices_file(content)?;

        let content = fs::read_to_string(ZONES_FILE).await?;
        (tree.zones, tree.zone_idx_lut) = file::parse_zones_file(content)?;

        tree.validate()?;

        Ok(tree)
    }

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        // validate zones point to validate devices
        for zone in &self.zones {
            for (floe_id, device_id) in &zone.devices {
                if !self
                    .device_names
                    .contains_key(&(floe_id.to_string(), device_id.to_string()))
                {
                    return Err(format!(
                        "Zone '{}' points to invalid device '{floe_id}.{device_id}'",
                        zone.name
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    pub async fn register_device(
        &mut self,
        floe_idx: u16,
        device_id: String,
        mut device: Device,
    ) -> Result<(), Box<dyn Error>> {
        let floe = &mut self.floes[floe_idx as usize];

        if floe.device_idx_lut.contains_key(&device_id) {
            // TODO maybe we should allow this instead of erroring? idk
            return Err(format!("Device {device_id} is already registered!").into());
        }

        let device_idx = floe.devices.len() as u16;

        match floe.name_queue.remove(&device_id) {
            Some(existing_name) => {
                // replace with persistent name
                device.name = existing_name;

                // update zone cache (list of floe+device idxs)
                // only update here since we know zones can only
                // contain existing devices
                for zone in &mut self.zones {
                    let Some(floe_devices) = zone.devices.get_mut(&floe.id) else {
                        continue;
                    };

                    if floe_devices.contains(&device_id) {
                        zone.idxs.push((floe_idx, device_idx));
                    }
                }
            }
            None => {
                // new device -> save Floe provided name to file
                file::add_device(DEVICES_FILE, &floe.id, &device_id, &device.name).await?;
            }
        }

        floe.device_idx_lut.insert(device_id, device_idx);
        floe.devices.push(device);

        Ok(())
    }

    pub async fn rename_device(
        &mut self,
        floe_id: String,
        device_id: String,
        new_name: String,
    ) -> Result<(), Box<dyn Error>> {
        // rename device in file, error if device doesn't exist
        file::rename_device(DEVICES_FILE, &floe_id, &device_id, &new_name).await?;

        match self.floe_idx_lut.get(&floe_id) {
            Some(floe_idx) => {
                let floe = &mut self.floes[*floe_idx as usize];

                match floe.device_idx_lut.get(&device_id) {
                    Some(device_idx) => {
                        // floe + device registered -> rename here
                        floe.devices[*device_idx as usize].name = new_name;
                    }
                    None => {
                        // floe is registered, device isn't, rename in floe's queue
                        floe.name_queue.insert(device_id, new_name);
                    }
                }
            }
            None => {
                // floe isn't registered, rename in global queue
                let name_queue = self.name_queue.get_mut(&floe_id)
                    .ok_or("Unexpected error. Floe isn't registered, exists in file, but not in global name queue")?;

                name_queue.insert(device_id, new_name);
            }
        }

        Ok(())
    }

    pub fn add_floe(
        &mut self,
        id: String,
        writer: FloeWriterDefault,
        max_supported_component: u16,
    ) -> Result<(), Box<dyn Error>> {
        if self.floe_idx_lut.contains_key(&id) {
            return Err(format!("Floe {} already exists!", id).into());
        }

        self.floe_idx_lut
            .insert(id.clone(), self.floes.len() as u16);

        let name_queue = self.name_queue.remove(&id).unwrap_or_default();

        self.floes.push(Floe {
            id,
            writer,
            max_supported_component,
            devices: Vec::with_capacity(20),
            device_idx_lut: FxHashMap::with_capacity_and_hasher(10, FxBuildHasher::default()),
            name_queue,
        });

        Ok(())
    }

    pub async fn add_zone(&mut self, zone_id: String, name: String) -> Result<(), Box<dyn Error>> {
        if self.zone_idx_lut.contains_key(&zone_id) {
            return Err(format!("Zone {zone_id} already exists!").into());
        }

        let zone = Zone {
            name,
            disabled: false,
            devices: FxHashMap::default(),
            idxs: SmallVec::new(),
        };

        file::add_zone(ZONES_FILE, &zone_id, &zone).await?;

        self.zone_idx_lut.insert(zone_id, self.zones.len() as u16);
        self.zones.push(zone);

        Ok(())
    }

    pub async fn set_zone_disabled(
        &mut self,
        zone_id: String,
        disabled: bool,
    ) -> Result<(), Box<dyn Error>> {
        let Some(zone_idx) = self.zone_idx_lut.get(&zone_id) else {
            return Err(format!("Zone {zone_id} doesnt't exists!").into());
        };

        let zone = &mut self.zones[*zone_idx as usize];
        zone.disabled = disabled;
        file::modify_zone(ZONES_FILE, &zone_id, zone).await?;

        Ok(())
    }

    pub async fn rename_zone(
        &mut self,
        zone_id: String,
        new_name: String,
    ) -> Result<(), Box<dyn Error>> {
        let Some(zone_idx) = self.zone_idx_lut.get(&zone_id) else {
            return Err(format!("Zone {zone_id} doesnt't exists!").into());
        };

        let zone = &mut self.zones[*zone_idx as usize];
        zone.name = new_name;
        file::modify_zone(ZONES_FILE, &zone_id, zone).await?;

        Ok(())
    }

    pub async fn add_device_to_zone(
        &mut self,
        zone_idx: u16,
        floe_idx: u16,
        device_idx: u16,
    ) -> Result<(), Box<dyn Error>> {
        let zone = &mut self.zones[zone_idx as usize];

        let floe = &mut self.floes[floe_idx as usize];
        let device = &mut floe.devices[device_idx as usize];

        match zone.devices.get_mut(&floe.id) {
            // zone already has devs with this Floe
            Some(floe_devs) => {
                floe_devs.insert(device.id.clone());
            }
            // first zone has had a dev with this Floe
            None => {
                let mut floe_devs = FxHashSet::default();
                floe_devs.insert(device.id.clone());
                zone.devices.insert(floe.id.clone(), floe_devs);
            }
        }

        zone.idxs.push((floe_idx, device_idx));

        file::modify_zone(ZONES_FILE, &zone.id, zone).await?;

        Ok(())
    }

    pub async fn remove_device_from_zone(
        &mut self,
        zone_idx: u16,
        floe_idx: u16,
        device_idx: u16,
    ) -> Result<(), Box<dyn Error>> {
        let zone = &mut self.zones[zone_idx as usize];
        let floe = &mut self.floes[floe_idx as usize];
        let device = &mut floes.devices[device_idx as usize];

        if let Some(floe_devs) = zone.devices.get_mut(&floe.id) {
            floe_devs.remove(device);
        }

        zone.idxs.retain(|i| *i != (floe_idx, device_idx));

        file::modify_zone(ZONES_FILE, &zone.id, zone).await?;

        Ok(())
    }
}

impl Presense {
    #[inline]
    pub fn set(&mut self, typ: ComponentType) {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        self.0[index] |= 1u32 << bit;
    }
}

impl HasComponent for Presense {
    #[inline]
    fn has(&self, typ: ComponentType) -> bool {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        (self.0[index] & (1u32 << bit)) != 0
    }
}
