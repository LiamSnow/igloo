use igloo_interface::{Component, ComponentType, FloeWriterDefault, MAX_SUPPORTED_COMPONENT};
use ini::Ini;
use rustc_hash::FxHashMap;
use smallvec::{SmallVec, smallvec};
use std::{error::Error, fmt::Display, time::Duration};
use tokio::fs;

use crate::glacier::{entity::HasComponent, query::WatchQuery};

use super::entity::Entity;

pub const GROUPS_FILE: &str = "groups.ini";
pub const DEVICES_FILE: &str = "devices.ini";

const MAX_EMPTY_DEVICE_SLOTS: usize = 10;
const MAX_EMPTY_GROUP_SLOTS: usize = 10;

/// persistent
#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct FloeID(pub String);

/// ephemeral
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloeRef(usize);

/// persistent
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct DeviceID(u64);

/// persistent
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GroupID(u64);

#[derive(Debug, Default)]
pub struct DeviceTree {
    /// Group idx -> Group
    groups: Vec<Option<Group>>,
    /// Floe Ref -> Floe
    floes: Vec<Option<Floe>>,
    /// Floe ID -> Floe Ref
    floe_ref_lut: FxHashMap<FloeID, FloeRef>,
    /// Device idx -> Device
    devices: Vec<Option<Device>>,

    group_generation: u32,
    device_generation: u32,
}

#[derive(Debug)]
pub struct Floe {
    id: FloeID,
    pub writer: FloeWriterDefault,
    pub max_supported_component: u16,
}

#[derive(Debug, Clone)]
pub struct Group {
    generation: u32,
    name: String,
    devices: SmallVec<[DeviceID; 20]>,
}

#[derive(Debug, Default, Clone)]
pub struct Device {
    idx: u32,
    generation: u32,
    name: String,
    owner: FloeID,
    owner_ref: Option<FloeRef>,

    // TODO FIXME whenever device is added/removed from group, NEED to update these
    /// queries for any entity
    queries: FxHashMap<ComponentType, SmallVec<[WatchQuery; 2]>>,
    /// queries for one entity
    entity_queries: FxHashMap<(usize, ComponentType), SmallVec<[WatchQuery; 2]>>,
    /// queries waiting for entity to be registered
    pending_entity_queries: FxHashMap<String, Vec<(ComponentType, WatchQuery)>>,

    pub presense: Presense,
    /// entity idx -> entity
    pub entities: SmallVec<[Entity; 16]>,
    /// entity ID -> entity idx
    entity_idx_lut: FxHashMap<String, usize>,
}

#[derive(Debug, Default, Clone)]
pub struct Presense([u32; MAX_SUPPORTED_COMPONENT.div_ceil(32) as usize]);

impl Device {
    pub fn attach_query(&mut self, comp_type: ComponentType, query: WatchQuery) {
        match self.queries.get_mut(&comp_type) {
            Some(v) => v.push(query.clone()),
            None => {
                self.queries.insert(comp_type, smallvec![query.clone()]);
            }
        }
    }

    pub fn attach_entity_query(
        &mut self,
        eidx: usize,
        comp_type: ComponentType,
        query: WatchQuery,
    ) {
        match self.entity_queries.get_mut(&(eidx, comp_type)) {
            Some(v) => v.push(query.clone()),
            None => {
                self.entity_queries
                    .insert((eidx, comp_type), smallvec![query.clone()]);
            }
        }
    }

    pub fn attach_pending_entity_query(
        &mut self,
        eid: String,
        comp_type: ComponentType,
        query: WatchQuery,
    ) {
        match self.pending_entity_queries.get_mut(&eid) {
            Some(v) => v.push((comp_type, query)),
            None => {
                self.pending_entity_queries
                    .insert(eid, vec![(comp_type, query)]);
            }
        }
    }

    fn unpend_entity_queries(&mut self, eid: &str, eidx: usize) {
        let Some(pending) = self.pending_entity_queries.remove(eid) else {
            return;
        };
        for (comp_type, query) in pending {
            self.attach_entity_query(eidx, comp_type, query);
        }
    }

    pub fn register_entity(&mut self, eid: String) {
        let eidx = self.entities.len();
        self.unpend_entity_queries(&eid, eidx);
        self.entity_idx_lut.insert(eid, eidx);
        self.entities.push(Entity::default());
    }

    pub async fn exec_queries(&mut self, eidx: usize, comp: Component) {
        let comp_type = comp.get_type();
        let did = DeviceID::from_parts(self.idx, self.generation);
        if let Some(queries) = self.queries.get(&comp_type) {
            for query in queries {
                if self.entities[eidx].matches_filter(&query.filter)
                    && let Err(e) = query
                        .tx
                        .send_timeout(
                            (query.prefix, did, eidx, comp.clone()),
                            Duration::from_millis(10),
                        )
                        .await
                {
                    eprintln!("Error sending watch query update: {e}");
                }
            }
        }
        if let Some(queries) = self.entity_queries.get(&(eidx, comp_type)) {
            for query in queries {
                if self.entities[eidx].matches_filter(&query.filter)
                    && let Err(e) = query
                        .tx
                        .send_timeout(
                            (query.prefix, did, eidx, comp.clone()),
                            Duration::from_millis(10),
                        )
                        .await
                {
                    eprintln!("Error sending watch query update: {e}");
                }
            }
        }
    }

    pub fn entity_idx_lut(&self) -> &FxHashMap<String, usize> {
        &self.entity_idx_lut
    }

    pub fn get_entity_idx(&self, eid: &str) -> Option<&usize> {
        self.entity_idx_lut.get(eid)
    }
}

impl DeviceTree {
    pub fn attach_query_to_all(
        &mut self,
        comp_type: ComponentType,
        query: WatchQuery,
    ) -> Result<(), Box<dyn Error>> {
        for device in &mut self.devices {
            let Some(device) = device else { continue };
            device.attach_query(comp_type, query.clone());
        }
        Ok(())
    }

    pub fn attach_query_to_group(
        &mut self,
        gid: GroupID,
        comp_type: ComponentType,
        query: WatchQuery,
    ) -> Result<(), Box<dyn Error>> {
        let group = self.group(gid)?;
        for did in group.devices.clone() {
            self.attach_query(did, comp_type, query.clone())?;
        }
        Ok(())
    }

    pub fn attach_query(
        &mut self,
        did: DeviceID,
        comp_type: ComponentType,
        query: WatchQuery,
    ) -> Result<(), Box<dyn Error>> {
        let device = self.device_mut(did)?;
        device.attach_query(comp_type, query);
        Ok(())
    }

    pub fn attach_entity_query(
        &mut self,
        did: DeviceID,
        eidx: usize,
        comp_type: ComponentType,
        query: WatchQuery,
    ) -> Result<(), Box<dyn Error>> {
        let device = self.device_mut(did)?;
        device.attach_entity_query(eidx, comp_type, query);
        Ok(())
    }

    pub fn attach_pending_entity_query(
        &mut self,
        did: DeviceID,
        eid: String,
        comp_type: ComponentType,
        query: WatchQuery,
    ) -> Result<(), Box<dyn Error>> {
        let device = self.device_mut(did)?;
        device.attach_pending_entity_query(eid, comp_type, query);
        Ok(())
    }

    pub fn iter_groups(&self) -> impl Iterator<Item = (GroupID, &Group)> {
        self.groups.iter().enumerate().filter_map(|(idx, g)| {
            g.as_ref().map(|group| {
                let did = GroupID::from_parts(idx as u32, group.generation);
                (did, group)
            })
        })
    }

    pub fn iter_devices(&self) -> impl Iterator<Item = (DeviceID, &Device)> {
        self.devices.iter().enumerate().filter_map(|(idx, d)| {
            d.as_ref().map(|device| {
                let did = DeviceID::from_parts(idx as u32, device.generation);
                (did, device)
            })
        })
    }

    pub fn iter_devices_in_group(
        &self,
        gid: GroupID,
    ) -> impl Iterator<Item = (DeviceID, &Device)> + '_ {
        let group = self.groups[gid.idx() as usize].as_ref().unwrap();
        let device_ids = group.devices.clone();
        device_ids.into_iter().map(move |did| {
            let device = self.devices[did.idx() as usize].as_ref().unwrap();
            (did, device)
        })
    }

    pub fn group(&self, gid: GroupID) -> Result<&Group, Box<dyn Error>> {
        if !self.is_group_id_valid(&gid) {
            return Err("Group ID invalid".into());
        }
        Ok(self.groups[gid.idx() as usize].as_ref().unwrap())
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
        self.floes[fref.0].as_mut().unwrap()
    }

    pub fn floe(&self, fref: FloeRef) -> &Floe {
        self.floes[fref.0].as_ref().unwrap()
    }

    pub fn floe_ref_lut(&self) -> &FxHashMap<FloeID, FloeRef> {
        &self.floe_ref_lut
    }

    pub async fn create_group(&mut self, name: String) -> Result<GroupID, Box<dyn Error>> {
        let num_free_slots = self.devices.iter().filter(|d| d.is_none()).count();

        let mut group = Group {
            generation: self.group_generation,
            name,
            devices: SmallVec::default(),
        };

        let idx = if num_free_slots > MAX_EMPTY_GROUP_SLOTS {
            self.group_generation += 1;
            group.generation += 1;
            let id = self.groups.iter().position(|d| d.is_none()).unwrap();
            self.groups[id] = Some(group);
            id
        } else {
            let idx = self.groups.len();
            self.groups.push(Some(group));
            idx
        };

        self.save_groups().await?;

        Ok(GroupID::from_parts(idx as u32, self.group_generation))
    }

    pub async fn delete_group(&mut self, gid: GroupID) -> Result<(), Box<dyn Error>> {
        match &self.groups[gid.idx() as usize] {
            Some(group) => {
                if group.generation != gid.generation() {
                    return Err("Stale reference".into());
                }
            }
            None => return Err("Device already deleted".into()),
        }

        self.groups[gid.idx() as usize] = None;

        self.save_groups().await?;

        Ok(())
    }

    pub async fn rename_group(
        &mut self,
        gid: GroupID,
        new_name: String,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_group_id_valid(&gid) {
            return Err("Group ID invalid".into());
        }

        let group = self.groups[gid.idx() as usize].as_mut().unwrap();
        group.name = new_name;

        self.save_groups().await?;

        Ok(())
    }

    pub fn detach_floe(&mut self, fref: FloeRef) {
        self.floes[fref.0] = None;

        let floe = self.floes[fref.0].as_ref().unwrap();
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

        let floe = self.floes[owner_ref.0].as_ref().unwrap();

        let mut device = Device {
            idx: 0,
            generation: self.device_generation,
            name,
            owner: floe.id.clone(),
            owner_ref: Some(owner_ref),
            queries: FxHashMap::default(),
            entity_queries: FxHashMap::default(),
            pending_entity_queries: FxHashMap::default(),
            presense: Presense::default(),
            entities: SmallVec::default(),
            entity_idx_lut: FxHashMap::default(),
        };

        let idx = if num_free_slots > MAX_EMPTY_DEVICE_SLOTS {
            self.device_generation += 1;
            device.generation += 1;
            let idx = self.devices.iter().position(|d| d.is_none()).unwrap();
            device.idx = idx as u32;
            self.devices[idx] = Some(device);
            idx
        } else {
            let idx = self.devices.len();
            device.idx = idx as u32;
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

    pub fn is_group_id_valid(&self, gid: &GroupID) -> bool {
        match &self.groups[gid.idx() as usize] {
            // make sure its not stale
            Some(group) => group.generation == gid.generation(),
            None => false, // deleted
        }
    }

    async fn save_groups(&self) -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();

        for (id, group) in self.groups.iter().enumerate() {
            let Some(group) = group else {
                continue;
            };

            ini.with_section(Some(id.to_string()))
                .set("name", &group.name)
                .set(
                    "devices",
                    group
                        .devices
                        .iter()
                        .map(|d| format!("{}:{}", d.idx(), d.generation()))
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .set("generation", group.generation.to_string());
        }

        let mut buf = Vec::new();
        ini.write_to(&mut buf)?;
        fs::write(GROUPS_FILE, buf).await?;

        Ok(())
    }

    pub async fn load() -> Result<Self, Box<dyn Error>> {
        if !fs::try_exists(GROUPS_FILE).await? || !fs::try_exists(DEVICES_FILE).await? {
            let me = Self::default();
            me.save_groups().await?;
            me.save_devices().await?;
            return Ok(me);
        }

        let groups_content = fs::read_to_string(GROUPS_FILE).await?;
        let groups_ini = Ini::load_from_str(&groups_content)?;

        let max_group_idx = groups_ini
            .sections()
            .filter_map(|s| s?.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        let mut groups = vec![None; max_group_idx + 1];

        for section in groups_ini.sections() {
            let Some(idx_str) = section else { continue };
            let idx: usize = idx_str.parse()?;

            let section_data = groups_ini.section(Some(idx_str)).unwrap();

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

            groups[idx] = Some(Group {
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
                idx: idx as u32,
                generation,
                name,
                owner,
                owner_ref: None,
                queries: FxHashMap::default(),
                entity_queries: FxHashMap::default(),
                pending_entity_queries: FxHashMap::default(),
                presense: Presense::default(),
                entities: SmallVec::default(),
                entity_idx_lut: FxHashMap::default(),
            });
        }

        let group_generation = groups
            .iter()
            .filter_map(|g| g.as_ref().map(|g| g.generation))
            .max()
            .unwrap_or(0);

        let device_generation = devices
            .iter()
            .filter_map(|d| d.as_ref().map(|d| d.generation))
            .max()
            .unwrap_or(0);

        Ok(DeviceTree {
            groups,
            devices,
            group_generation,
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

impl Group {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn devices(&self) -> &SmallVec<[DeviceID; 20]> {
        &self.devices
    }
}

impl GroupID {
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        GroupID(packed)
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
