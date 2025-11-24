use igloo_interface::{
    Component, ComponentType, MSIC,
    id::{DeviceID, FloeID, FloeRef, GroupID},
    ipc::IWriter,
    query::{DeviceSnapshot, EntitySnapshot, FloeSnapshot, GroupSnapshot, TypeFilter},
};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use std::time::Instant;
use tokio::task::JoinHandle;

pub const COMP_TYPE_ARR_LEN: usize = MSIC as usize + 1;

/// Root
/// WARN: Mutations to the device tree must only occur in `mutation.rs`
#[derive(Debug, Default)]
pub struct DeviceTree {
    pub(super) groups: Vec<Option<Group>>,
    pub(super) attached_floes: Vec<Option<Floe>>,
    pub(super) floe_ref_lut: FxHashMap<FloeID, FloeRef>,
    pub(super) devices: Vec<Option<Device>>,

    pub(super) group_generation: u32,
    pub(super) device_generation: u32,
}

/// Connected Floe
/// `FloeRef` is an ephemeral array index
/// `FloeID` is the persistent name
#[derive(Debug)]
pub struct Floe {
    pub(super) id: FloeID,
    pub(super) fref: FloeRef,
    pub writer: IWriter,
    pub(super) devices: SmallVec<[DeviceID; 50]>,
    pub(super) handle: JoinHandle<()>,
    pub msic: u16,
    pub msim: u8,
}

/// Collection of devices (ex. "Living Room")
#[derive(Debug, Clone)]
pub struct Group {
    pub(super) id: GroupID,
    pub(super) name: String,
    pub(super) devices: FxHashSet<DeviceID>,
}

/// Physical Device, owned (registered & attached) by Floe
#[derive(Debug, Clone)]
pub struct Device {
    pub(super) id: DeviceID,
    pub(super) name: String,
    pub(super) owner: FloeID,
    pub(super) owner_ref: Option<FloeRef>,
    pub(super) groups: FxHashSet<GroupID>,
    /// Bitset of which component types exist on any of its entity
    pub(super) presense: Presense,
    /// ComponentType -> Entity Indexes
    pub(super) comp_to_entity: [SmallVec<[usize; 4]>; COMP_TYPE_ARR_LEN],
    /// Entity index -> Entity
    pub(super) entities: SmallVec<[Entity; 16]>,
    /// Entity name -> index
    pub(super) entity_index_lut: FxHashMap<String, usize>,
    pub(super) last_updated: Instant,
}

/// Tracks presence of components on a device
/// Union of all entity presenses
/// Each bit corresponds to a `ComponentType` ID
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Presense(pub [u32; COMP_TYPE_ARR_LEN.div_ceil(32)]);

#[derive(Debug, Clone)]
pub struct Entity {
    #[allow(dead_code)]
    pub(super) name: String,
    #[allow(dead_code)]
    pub(super) index: usize,
    pub(super) components: SmallVec<[Component; 8]>,
    /// Maps `ComponentType` ID -> index in `components`
    /// `0xFF` = not present
    pub(super) indices: [u8; COMP_TYPE_ARR_LEN],
    pub(super) last_updated: Instant,
}

#[derive(thiserror::Error, Debug)]
pub enum TreeIDError {
    #[error("Device {0} does not exist")]
    DeviceNotExistant(DeviceID),
    #[error("Device {0} is a stale reference. It was deleted and something else replaced it.")]
    DeviceStale(DeviceID),
    #[error("Group {0} does not exist")]
    GroupNotExistant(GroupID),
    #[error("Group {0} is a stale reference. It was deleted and something else replaced it.")]
    GroupStale(GroupID),
    /// Really really shouldn't happen
    #[error("Floe Reference {0} is invalid, because the Floe has detached.")]
    FloeDetached(FloeRef),
    #[error("Floe Reference {0} is out of bounds. Most likely this is an internal issue.")]
    FloeRefOutOfBounds(FloeRef),
    #[error("Floe {0} is invalid. Maybe it isn't attached?")]
    FloeIDInvalid(FloeID),
}

impl DeviceTree {
    pub fn floes(&self) -> &Vec<Option<Floe>> {
        &self.attached_floes
    }

    pub fn groups(&self) -> &Vec<Option<Group>> {
        &self.groups
    }

    pub fn devices(&self) -> &Vec<Option<Device>> {
        &self.devices
    }

    /// Gets & Validates from DeviceID
    #[inline]
    pub fn device(&self, did: &DeviceID) -> Result<&Device, TreeIDError> {
        let index = did.index() as usize;
        match self.devices.get(index).and_then(|o| o.as_ref()) {
            Some(d) if d.id.generation() == did.generation() => Ok(d),
            Some(_) => Err(TreeIDError::DeviceStale(*did)),
            None => Err(TreeIDError::DeviceNotExistant(*did)),
        }
    }

    /// Gets & Validates from DeviceID
    #[inline]
    pub fn device_mut(&mut self, did: &DeviceID) -> Result<&mut Device, TreeIDError> {
        let index = did.index() as usize;
        match self.devices.get_mut(index).and_then(|o| o.as_mut()) {
            Some(d) if d.id.generation() == did.generation() => Ok(d),
            Some(_) => Err(TreeIDError::DeviceStale(*did)),
            None => Err(TreeIDError::DeviceNotExistant(*did)),
        }
    }

    /// Gets & Validates from GroupID
    #[inline]
    pub fn group(&self, gid: &GroupID) -> Result<&Group, TreeIDError> {
        match self
            .groups
            .get(gid.index() as usize)
            .and_then(|o| o.as_ref())
        {
            Some(g) if g.id.generation() == gid.generation() => Ok(g),
            Some(_) => Err(TreeIDError::GroupStale(*gid)),
            None => Err(TreeIDError::GroupNotExistant(*gid)),
        }
    }

    /// Gets & Validates from GroupID
    #[inline]
    pub(super) fn group_mut(&mut self, gid: &GroupID) -> Result<&mut Group, TreeIDError> {
        let index = gid.index() as usize;
        match self.groups.get_mut(index).and_then(|o| o.as_mut()) {
            Some(g) if g.id.generation() == gid.generation() => Ok(g),
            Some(_) => Err(TreeIDError::GroupStale(*gid)),
            None => Err(TreeIDError::GroupNotExistant(*gid)),
        }
    }

    #[inline]
    pub fn floe(&self, fref: &FloeRef) -> Result<&Floe, TreeIDError> {
        match self.attached_floes.get(fref.0) {
            Some(o) => match o.as_ref() {
                Some(f) => Ok(f),
                None => Err(TreeIDError::FloeDetached(*fref)),
            },
            None => Err(TreeIDError::FloeRefOutOfBounds(*fref)),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn floe_mut(&mut self, fref: &FloeRef) -> Result<&mut Floe, TreeIDError> {
        match self.attached_floes.get_mut(fref.0) {
            Some(o) => match o.as_mut() {
                Some(f) => Ok(f),
                None => Err(TreeIDError::FloeDetached(*fref)),
            },
            None => Err(TreeIDError::FloeRefOutOfBounds(*fref)),
        }
    }

    #[inline]
    pub fn floe_ref(&self, fid: &FloeID) -> Result<&FloeRef, TreeIDError> {
        self.floe_ref_lut
            .get(fid)
            .ok_or(TreeIDError::FloeIDInvalid(fid.clone()))
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

    #[inline(always)]
    pub fn has(&self, typ: ComponentType) -> bool {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        (self.0[index] & (1u32 << bit)) != 0
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            components: SmallVec::new(),
            indices: [0xFF; COMP_TYPE_ARR_LEN],
            name: String::with_capacity(20),
            index: usize::MAX,
            last_updated: Instant::now(),
        }
    }
}

impl Entity {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    #[allow(dead_code)]
    pub fn index(&self) -> &usize {
        &self.index
    }

    #[inline]
    #[allow(dead_code)]
    pub fn components(&self) -> &SmallVec<[Component; 8]> {
        &self.components
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get(&self, typ: ComponentType) -> Option<&Component> {
        let index = self.indices[typ as usize];
        if index != 0xFF {
            Some(&self.components[index as usize])
        } else {
            None
        }
    }

    /// Puts or sets (if already exists) a component onto this entity
    /// Returns `Some(typ)` if it was put/new
    #[inline]
    pub fn put(&mut self, val: Component) -> Option<ComponentType> {
        let typ = val.get_type();
        let type_id = typ as usize;
        let index = self.indices[type_id];

        if index == 0xFF {
            // put
            let new_index = self.components.len() as u8;
            self.components.push(val);
            self.indices[type_id] = new_index;
            Some(typ)
        } else {
            // set
            self.components[index as usize] = val;
            None
        }
    }

    #[inline]
    pub fn last_updated(&self) -> &Instant {
        &self.last_updated
    }

    pub fn snapshot(&self, parent: DeviceID) -> EntitySnapshot {
        EntitySnapshot {
            name: self.name.clone(),
            index: self.index,
            components: self.components.to_vec(),
            parent,
        }
    }

    #[inline(always)]
    pub fn has(&self, r#type: ComponentType) -> bool {
        self.indices[r#type as usize] != 0xFF
    }

    #[inline(always)]
    pub fn matches(&self, filter: &TypeFilter) -> bool {
        match filter {
            TypeFilter::With(t) => self.has(*t),
            TypeFilter::Without(t) => !self.has(*t),
            TypeFilter::And(filters) => filters.iter().all(|f| self.matches(f)),
            TypeFilter::Or(filters) => filters.iter().any(|f| self.matches(f)),
            TypeFilter::Not(f) => !self.matches(f),
        }
    }
}

impl Device {
    #[inline]
    pub fn id(&self) -> &DeviceID {
        &self.id
    }

    #[allow(dead_code)]
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    #[allow(dead_code)]
    pub fn owner(&self) -> &FloeID {
        &self.owner
    }

    #[inline]
    #[allow(dead_code)]
    pub fn owner_ref(&self) -> Option<FloeRef> {
        self.owner_ref
    }

    #[inline]
    #[allow(dead_code)]
    pub fn entity_index(&self, eid: &str) -> Option<&usize> {
        self.entity_index_lut.get(eid)
    }

    #[inline]
    pub fn num_entities(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn entities(&self) -> &SmallVec<[Entity; 16]> {
        &self.entities
    }

    #[inline]
    pub fn last_updated(&self) -> &Instant {
        &self.last_updated
    }

    #[allow(dead_code)]
    #[inline]
    pub fn groups(&self) -> &FxHashSet<GroupID> {
        &self.groups
    }

    pub fn snapshot(&self, include_components: bool) -> DeviceSnapshot {
        DeviceSnapshot {
            id: self.id,
            name: self.name.clone(),
            entities: match include_components {
                true => self.entities.iter().map(|e| e.snapshot(self.id)).collect(),
                false => vec![],
            },
            owner: self.owner.clone(),
            owner_ref: self.owner_ref,
            groups: self.groups.clone(),
        }
    }

    #[inline(always)]
    pub fn has(&self, r#type: ComponentType) -> bool {
        self.presense.has(r#type)
    }

    #[inline(always)]
    pub fn matches(&self, filter: &TypeFilter) -> bool {
        match filter {
            TypeFilter::With(t) => self.has(*t),
            TypeFilter::Without(t) => !self.has(*t),
            TypeFilter::And(filters) => filters.iter().all(|f| self.matches(f)),
            TypeFilter::Or(filters) => filters.iter().any(|f| self.matches(f)),
            TypeFilter::Not(f) => !self.matches(f),
        }
    }

    pub fn comp_to_entity(&self) -> &[SmallVec<[usize; 4]>; COMP_TYPE_ARR_LEN] {
        &self.comp_to_entity
    }
}

impl Group {
    #[inline]
    pub fn id(&self) -> &GroupID {
        &self.id
    }

    #[inline]
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    #[allow(dead_code)]
    pub fn devices(&self) -> &FxHashSet<DeviceID> {
        &self.devices
    }

    pub fn snapshot(&self) -> GroupSnapshot {
        GroupSnapshot {
            id: self.id,
            name: self.name.clone(),
            devices: self.devices.clone(),
        }
    }
}

impl Floe {
    #[inline]
    pub fn id(&self) -> &FloeID {
        &self.id
    }

    #[inline]
    pub fn fref(&self) -> &FloeRef {
        &self.fref
    }

    #[inline]
    pub fn devices(&self) -> &SmallVec<[DeviceID; 50]> {
        &self.devices
    }

    pub fn snapshot(&self) -> FloeSnapshot {
        FloeSnapshot {
            id: self.id.clone(),
            fref: self.fref,
            max_supported_component: self.msic,
            devices: self.devices.to_vec(),
        }
    }
}
