use std::{error::Error, fmt::Display};

use igloo_interface::{ComponentType, FloeWriterDefault, MAX_SUPPORTED_COMPONENT};
use ini::Ini;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use tokio::fs;

use crate::glacier::entity::HasComponent;

use super::entity::Entity;

pub const ZONES_FILE: &str = "zones.ini";
pub const DEVICES_FILE: &str = "devices.ini";

const MAX_EMPTY_DEVICE_SLOTS: usize = 10;
const MAX_EMPTY_ZONE_SLOTS: usize = 10;

/// persistent
#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct FloeID(pub String);

/// ephemeral
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloeRef(usize);

/// persistent
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DeviceID(u64);

/// persistent
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ZoneID(u64);

#[derive(Debug, Default)]
pub struct DeviceTree {
    /// Zone idx -> Zone
    zones: Vec<Option<Zone>>,
    /// Floe Ref -> Floe
    floes: Vec<Option<Floe>>,
    /// Floe ID -> Floe Ref
    floe_ref_lut: FxHashMap<FloeID, FloeRef>,
    /// Device idx -> Device
    devices: Vec<Option<Device>>,
    zone_generation: u32,
    device_generation: u32,
}

#[derive(Debug)]
pub struct Floe {
    id: FloeID,
    pub writer: FloeWriterDefault,
    pub max_supported_component: u16,
}

#[derive(Debug, Clone)]
pub struct Zone {
    generation: u32,
    name: String,
    devices: SmallVec<[DeviceID; 20]>,
}

#[derive(Debug, Default, Clone)]
pub struct Device {
    generation: u32,
    name: String,
    owner: FloeID,
    owner_ref: Option<FloeRef>,

    pub presense: Presense,
    /// entity idx -> entity
    pub entities: SmallVec<[Entity; 16]>,
    /// entity ID -> entity idx
    pub entity_idx_lut: FxHashMap<String, usize>,
}

#[derive(Debug, Default, Clone)]
pub struct Presense([u32; MAX_SUPPORTED_COMPONENT.div_ceil(32) as usize]);

impl DeviceTree {
    pub fn iter_zones_with_ids(&self) -> impl Iterator<Item = (ZoneID, &Zone)> {
        self.zones.iter().enumerate().filter_map(|(idx, z)| {
            z.as_ref().map(|zone| {
                let did = ZoneID::from_parts(idx as u32, zone.generation);
                (did, zone)
            })
        })
    }

    pub fn iter_devices_with_ids(&self) -> impl Iterator<Item = (DeviceID, &Device)> {
        self.devices.iter().enumerate().filter_map(|(idx, d)| {
            d.as_ref().map(|device| {
                let did = DeviceID::from_parts(idx as u32, device.generation);
                (did, device)
            })
        })
    }

    pub fn iter_devices(&self) -> impl Iterator<Item = &Device> {
        self.devices.iter().filter_map(|d| d.as_ref())
    }

    pub fn iter_devices_in_zone(&self, zid: ZoneID) -> impl Iterator<Item = &Device> + '_ {
        let zone = self.zones[zid.idx() as usize].as_ref().unwrap();
        let device_ids = zone.devices.clone();
        device_ids
            .into_iter()
            .map(move |did| self.devices[did.idx() as usize].as_ref().unwrap())
    }

    pub fn iter_devices_in_zone_with_ids(
        &self,
        zid: ZoneID,
    ) -> impl Iterator<Item = (DeviceID, &Device)> + '_ {
        let zone = self.zones[zid.idx() as usize].as_ref().unwrap();
        let device_ids = zone.devices.clone();
        device_ids.into_iter().map(move |did| {
            let device = self.devices[did.idx() as usize].as_ref().unwrap();
            (did, device)
        })
    }

    pub fn device(&self, did: DeviceID) -> Result<&Device, Box<dyn Error>> {
        if !self.is_device_id_valid(&did) {
            return Err("Device ID invalid".into());
        }
        Ok(self.devices[did.idx() as usize].as_ref().unwrap())
    }

    pub fn device_mut(&mut self, did: DeviceID) -> Result<&mut Device, Box<dyn Error>> {
        if !self.is_device_id_valid(&did) {
            return Err("Device ID invalid".into());
        }
        Ok(self.devices[did.idx() as usize].as_mut().unwrap())
    }

    pub fn floe_mut(&mut self, fref: FloeRef) -> &mut Floe {
        self.floes[fref.0 as usize].as_mut().unwrap()
    }

    pub fn floe(&self, fref: FloeRef) -> &Floe {
        self.floes[fref.0 as usize].as_ref().unwrap()
    }

    pub fn floe_ref_lut(&self) -> &FxHashMap<FloeID, FloeRef> {
        &self.floe_ref_lut
    }

    pub async fn create_zone(&mut self, name: String) -> Result<ZoneID, Box<dyn Error>> {
        let num_free_slots = self.devices.iter().filter(|d| d.is_none()).count();

        let mut zone = Zone {
            generation: self.zone_generation,
            name,
            devices: SmallVec::default(),
        };

        let idx = if num_free_slots > MAX_EMPTY_ZONE_SLOTS {
            self.zone_generation += 1;
            zone.generation += 1;
            let id = self.zones.iter().position(|d| d.is_none()).unwrap();
            self.zones[id] = Some(zone);
            id
        } else {
            let idx = self.zones.len();
            self.zones.push(Some(zone));
            idx
        };

        self.save_zones().await?;

        Ok(ZoneID::from_parts(idx as u32, self.zone_generation))
    }

    pub async fn delete_zone(&mut self, zid: ZoneID) -> Result<(), Box<dyn Error>> {
        match &self.zones[zid.idx() as usize] {
            Some(zone) => {
                if zone.generation != zid.generation() {
                    return Err("Stale reference".into());
                }
            }
            None => return Err("Device already deleted".into()),
        }

        self.zones[zid.idx() as usize] = None;

        self.save_zones().await?;

        Ok(())
    }

    pub async fn rename_zone(
        &mut self,
        zid: ZoneID,
        new_name: String,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_zone_id_valid(&zid) {
            return Err("Zone ID invalid".into());
        }

        let zone = self.zones[zid.idx() as usize].as_mut().unwrap();
        zone.name = new_name;

        self.save_zones().await?;

        Ok(())
    }

    pub fn detach_floe(&mut self, fref: FloeRef) {
        self.floes[fref.0] = None;

        let floe = self.floes[fref.0 as usize].as_ref().unwrap();
        self.floe_ref_lut.remove(&floe.id);

        // detach all its devices
        for device in &mut self.devices {
            let Some(device) = device else {
                continue;
            };

            if device.owner_ref.as_ref() == Some(&fref) {
                device.owner_ref = None;
            }
        }
    }

    pub fn attach_floe(
        &mut self,
        fid: FloeID,
        writer: FloeWriterDefault,
        max_supported_component: u16,
    ) -> Result<FloeRef, String> {
        if self.floe_ref_lut.contains_key(&fid) {
            return Err("Floe already attached".into());
        }

        let floe = Floe {
            id: fid.clone(),
            writer,
            max_supported_component,
        };

        let fref = if let Some(slot) = self.floes.iter().position(|f| f.is_none()) {
            // take empty slot
            // we can safely do this since [detach_floe] removes its refs
            self.floes[slot] = Some(floe);
            FloeRef(slot)
        } else {
            let idx = self.floes.len();
            self.floes.push(Some(floe));
            FloeRef(idx)
        };

        // attach all devices it owns
        for device in &mut self.devices {
            let Some(device) = device else {
                continue;
            };

            if device.owner == fid {
                device.owner_ref = Some(fref)
            }
        }

        self.floe_ref_lut.insert(fid, fref);

        Ok(fref)
    }

    /// ONLY A FLOE SHOULD DO THIS
    /// Otherwise its always going to try and update + attach deleted devices
    pub async fn delete_device(&mut self, did: DeviceID) -> Result<(), Box<dyn Error>> {
        match &self.devices[did.idx() as usize] {
            Some(device) => {
                if device.generation != did.generation() {
                    return Err("Stale reference".into());
                }
            }
            None => return Err("Device already deleted".into()),
        }

        self.devices[did.idx() as usize] = None;

        self.save_devices().await?;

        Ok(())
    }

    /// Create a brand new device (doesn't attach)
    pub async fn create_device(
        &mut self,
        name: String,
        owner_ref: FloeRef,
    ) -> Result<DeviceID, Box<dyn Error>> {
        let num_free_slots = self.devices.iter().filter(|d| d.is_none()).count();

        let floe = self.floes[owner_ref.0 as usize].as_ref().unwrap();

        let mut device = Device {
            generation: self.device_generation,
            name,
            owner: floe.id.clone(),
            owner_ref: Some(owner_ref),
            presense: Presense::default(),
            entities: SmallVec::default(),
            entity_idx_lut: FxHashMap::default(),
        };

        let idx = if num_free_slots > MAX_EMPTY_DEVICE_SLOTS {
            self.device_generation += 1;
            device.generation += 1;
            let idx = self.devices.iter().position(|d| d.is_none()).unwrap();
            self.devices[idx] = Some(device);
            idx
        } else {
            let idx = self.devices.len();
            self.devices.push(Some(device));
            idx
        };

        self.save_devices().await?;

        Ok(DeviceID::from_parts(idx as u32, self.device_generation))
    }

    pub async fn rename_device(
        &mut self,
        did: DeviceID,
        new_name: String,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_device_id_valid(&did) {
            return Err("Device ID invalid".into());
        }

        let device = self.devices[did.idx() as usize].as_mut().unwrap();
        device.name = new_name;

        self.save_devices().await?;

        Ok(())
    }

    pub fn is_device_attached(&self, did: &DeviceID) -> Result<bool, String> {
        match &self.devices[did.idx() as usize] {
            Some(device) => {
                if device.generation != did.generation() {
                    return Err("Stale reference".into());
                }
                Ok(device.owner_ref.is_some())
            }
            None => Err("Device deleted".into()),
        }
    }

    pub fn is_device_id_valid(&self, did: &DeviceID) -> bool {
        match &self.devices[did.idx() as usize] {
            // make sure its not stale
            Some(device) => device.generation == did.generation(),
            None => false, // deleted
        }
    }

    pub fn is_zone_id_valid(&self, zid: &ZoneID) -> bool {
        match &self.zones[zid.idx() as usize] {
            // make sure its not stale
            Some(zone) => zone.generation == zid.generation(),
            None => false, // deleted
        }
    }

    async fn save_zones(&self) -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();

        for (id, zone) in self.zones.iter().enumerate() {
            let Some(zone) = zone else {
                continue;
            };

            ini.with_section(Some(id.to_string()))
                .set("name", &zone.name)
                .set(
                    "devices",
                    zone.devices
                        .iter()
                        .map(|d| format!("{}:{}", d.idx(), d.generation()))
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .set("generation", zone.generation.to_string());
        }

        let mut buf = Vec::new();
        ini.write_to(&mut buf)?;
        fs::write(ZONES_FILE, buf).await?;

        Ok(())
    }

    pub async fn load() -> Result<Self, Box<dyn Error>> {
        if !fs::try_exists(ZONES_FILE).await? || !fs::try_exists(DEVICES_FILE).await? {
            let me = Self::default();
            me.save_zones().await?;
            me.save_devices().await?;
            return Ok(me);
        }

        let zones_content = fs::read_to_string(ZONES_FILE).await?;
        let zones_ini = Ini::load_from_str(&zones_content)?;

        let max_zone_idx = zones_ini
            .sections()
            .filter_map(|s| s?.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        let mut zones = vec![None; max_zone_idx + 1];

        for section in zones_ini.sections() {
            let Some(idx_str) = section else { continue };
            let idx: usize = idx_str.parse()?;

            let section_data = zones_ini.section(Some(idx_str)).unwrap();

            let name = section_data.get("name").ok_or("Missing name")?.to_string();
            let generation: u32 = section_data
                .get("generation")
                .ok_or("Missing generation")?
                .parse()?;
            let devices_str = section_data.get("devices").unwrap_or("");

            let devices: SmallVec<[DeviceID; 20]> = if devices_str.is_empty() {
                SmallVec::new()
            } else {
                devices_str
                    .split(',')
                    .map(|s| {
                        let mut parts = s.split(':');
                        let idx = parts
                            .next()
                            .ok_or("Missing device ID idx part")?
                            .parse::<u32>()?;
                        let generation = parts
                            .next()
                            .ok_or("Missing device ID generation part")?
                            .parse::<u32>()?;
                        Ok(DeviceID::from_parts(idx, generation))
                    })
                    .collect::<Result<_, Box<dyn Error>>>()?
            };

            zones[idx] = Some(Zone {
                generation,
                name,
                devices,
            });
        }

        let devices_content = fs::read_to_string(DEVICES_FILE).await?;
        let devices_ini = Ini::load_from_str(&devices_content)?;

        let max_device_idx = devices_ini
            .sections()
            .filter_map(|s| s?.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        let mut devices = vec![None; max_device_idx + 1];

        for section in devices_ini.sections() {
            let Some(idx_str) = section else { continue };
            let idx: usize = idx_str.parse()?;

            let section_data = devices_ini.section(Some(idx_str)).unwrap();

            let name = section_data.get("name").ok_or("Missing name")?.to_string();
            let generation: u32 = section_data
                .get("generation")
                .ok_or("Missing generation")?
                .parse()?;
            let owner = FloeID(
                section_data
                    .get("owner")
                    .ok_or("Missing owner")?
                    .to_string(),
            );

            devices[idx] = Some(Device {
                generation,
                name,
                owner,
                owner_ref: None,
                presense: Presense::default(),
                entities: SmallVec::default(),
                entity_idx_lut: FxHashMap::default(),
            });
        }

        let zone_generation = zones
            .iter()
            .filter_map(|z| z.as_ref().map(|z| z.generation))
            .max()
            .unwrap_or(0);

        let device_generation = devices
            .iter()
            .filter_map(|d| d.as_ref().map(|d| d.generation))
            .max()
            .unwrap_or(0);

        Ok(DeviceTree {
            zones,
            devices,
            zone_generation,
            device_generation,
            floes: Vec::new(),
            floe_ref_lut: FxHashMap::default(),
        })
    }

    async fn save_devices(&self) -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();

        for (id, device) in self.devices.iter().enumerate() {
            let Some(device) = device else {
                continue;
            };

            ini.with_section(Some(id.to_string()))
                .set("name", &device.name)
                .set("owner", &device.owner.0)
                .set("generation", device.generation.to_string());
        }

        let mut buf = Vec::new();
        ini.write_to(&mut buf)?;
        fs::write(DEVICES_FILE, buf).await?;

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

impl Device {
    pub fn owner(&self) -> &FloeID {
        &self.owner
    }

    pub fn owner_ref(&self) -> Option<FloeRef> {
        self.owner_ref
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Zone {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn devices(&self) -> &SmallVec<[DeviceID; 20]> {
        &self.devices
    }
}

impl ZoneID {
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        ZoneID(packed)
    }

    pub fn idx(&self) -> u32 {
        self.0 as u32
    }

    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }
}

impl DeviceID {
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        DeviceID(packed)
    }

    pub fn from_comb(c: u64) -> Self {
        DeviceID(c)
    }

    pub fn idx(&self) -> u32 {
        self.0 as u32
    }

    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn take(self) -> u64 {
        self.0
    }
}

impl Display for DeviceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.idx(), self.generation())
    }
}
