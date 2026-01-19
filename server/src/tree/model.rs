use igloo_interface::{
    Component, ComponentType, MSIC,
    id::{DeviceID, EntityID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
    ipc::IWriter,
    query::{DeviceSnapshot, EntitySnapshot, ExtensionSnapshot, GroupSnapshot, TypeFilter},
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use smallvec::SmallVec;
use std::{collections::HashMap, time::Instant};
use tokio::task::JoinHandle;

use crate::tree::{arena::Arena, persist::IniWriter};

pub const COMP_TYPE_ARR_LEN: usize = MSIC as usize + 1;

/// Root
/// WARN: Mutations to the device tree must only occur in `mutation.rs`
pub struct DeviceTree {
    pub(super) groups: Arena<GroupID, Group>,
    pub(super) attached_exts: Vec<Option<Extension>>,
    pub(super) ext_ref_lut: FxHashMap<ExtensionID, ExtensionIndex>,
    pub(super) devices: Arena<DeviceID, Device>,
    pub(super) groups_writer: IniWriter,
    pub(super) devices_writer: IniWriter,
}

/// Connected Extension
/// `ExtensionIndex` is an ephemeral array index
/// `ExtensionID` is the persistent name
#[derive(Debug)]
pub struct Extension {
    pub(super) id: ExtensionID,
    pub(super) index: ExtensionIndex,
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

/// Physical Device, owned (registered & attached) by Extension
#[derive(Debug, Clone)]
pub struct Device {
    pub(super) id: DeviceID,
    pub(super) name: String,
    pub(super) owner: ExtensionID,
    pub(super) owner_ref: Option<ExtensionIndex>,
    pub(super) groups: FxHashSet<GroupID>,
    /// Bitset of which component types exist on any of its entity
    // TODO FIXME now that we have comp_to_entity, I think this is useless
    pub(super) presense: Presense,
    /// ComponentType -> Entity Indexes
    pub(super) comp_to_entity: [SmallVec<[EntityIndex; 4]>; COMP_TYPE_ARR_LEN],
    /// Entity index -> Entity
    pub(super) entities: SmallVec<[Entity; 16]>,
    pub(super) entity_index_lut: FxHashMap<EntityID, EntityIndex>,
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
    pub(super) id: EntityID,
    #[allow(dead_code)]
    pub(super) index: EntityIndex,
    pub(super) components: SmallVec<[Component; 8]>,
    /// Maps `ComponentType` ID -> index in `components`
    /// `0xFF` = not present
    pub(super) indices: [u8; COMP_TYPE_ARR_LEN],
    pub(super) last_updated: Instant,
}

#[derive(thiserror::Error, Debug)]
pub enum TreeIDError {
    #[error("Device {0} does not exist")]
    DeviceDeleted(DeviceID),
    #[error("Group {0} does not exist")]
    GroupDeleted(GroupID),
    /// Really really shouldn't happen
    #[error("Extension Reference {0} is invalid, because the Extension has detached.")]
    ExtensionDetached(ExtensionIndex),
    #[error("Extension Reference {0} is out of bounds. Most likely this is an internal issue.")]
    ExtensionRefOutOfBounds(ExtensionIndex),
    #[error("Extension {0} is invalid. Maybe it isn't attached?")]
    ExtensionIDInvalid(ExtensionID),
}

impl DeviceTree {
    pub fn exts(&self) -> &Vec<Option<Extension>> {
        &self.attached_exts
    }

    pub fn groups(&self) -> &Arena<GroupID, Group> {
        &self.groups
    }

    pub fn devices(&self) -> &Arena<DeviceID, Device> {
        &self.devices
    }

    /// Gets & Validates from DeviceID
    #[inline]
    pub fn device(&self, did: &DeviceID) -> Result<&Device, TreeIDError> {
        self.devices
            .get(did)
            .ok_or(TreeIDError::DeviceDeleted(*did))
    }

    /// Gets & Validates from DeviceID
    #[inline]
    pub fn device_mut(&mut self, did: &DeviceID) -> Result<&mut Device, TreeIDError> {
        self.devices
            .get_mut(did)
            .ok_or(TreeIDError::DeviceDeleted(*did))
    }

    /// Gets & Validates from GroupID
    #[inline]
    pub fn group(&self, gid: &GroupID) -> Result<&Group, TreeIDError> {
        self.groups.get(gid).ok_or(TreeIDError::GroupDeleted(*gid))
    }

    /// Gets & Validates from GroupID
    #[inline]
    pub(super) fn group_mut(&mut self, gid: &GroupID) -> Result<&mut Group, TreeIDError> {
        self.groups
            .get_mut(gid)
            .ok_or(TreeIDError::GroupDeleted(*gid))
    }

    #[inline]
    pub fn ext(&self, index: &ExtensionIndex) -> Result<&Extension, TreeIDError> {
        match self.attached_exts.get(index.0) {
            Some(o) => match o.as_ref() {
                Some(f) => Ok(f),
                None => Err(TreeIDError::ExtensionDetached(*index)),
            },
            None => Err(TreeIDError::ExtensionRefOutOfBounds(*index)),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn ext_mut(&mut self, index: &ExtensionIndex) -> Result<&mut Extension, TreeIDError> {
        match self.attached_exts.get_mut(index.0) {
            Some(o) => match o.as_mut() {
                Some(f) => Ok(f),
                None => Err(TreeIDError::ExtensionDetached(*index)),
            },
            None => Err(TreeIDError::ExtensionRefOutOfBounds(*index)),
        }
    }

    #[inline]
    pub fn ext_index(&self, eid: &ExtensionID) -> Result<&ExtensionIndex, TreeIDError> {
        self.ext_ref_lut
            .get(eid)
            .ok_or(TreeIDError::ExtensionIDInvalid(eid.clone()))
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
            id: EntityID(String::with_capacity(20)),
            index: EntityIndex(usize::MAX),
            last_updated: Instant::now(),
        }
    }
}

impl Entity {
    #[inline]
    pub fn id(&self) -> &EntityID {
        &self.id
    }

    #[inline]
    #[allow(dead_code)]
    pub fn index(&self) -> &EntityIndex {
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
            id: self.id.clone(),
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
    pub fn new(id: DeviceID, name: String, owner: ExtensionID) -> Self {
        Device {
            id,
            name,
            owner,
            owner_ref: None,
            groups: FxHashSet::with_capacity_and_hasher(10, FxBuildHasher),
            presense: Presense::default(),
            entities: SmallVec::default(),
            entity_index_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
            last_updated: Instant::now(),
            comp_to_entity: [const { SmallVec::new_const() }; COMP_TYPE_ARR_LEN],
        }
    }

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
    pub fn owner(&self) -> &ExtensionID {
        &self.owner
    }

    #[inline]
    #[allow(dead_code)]
    pub fn owner_ref(&self) -> Option<ExtensionIndex> {
        self.owner_ref
    }

    #[inline]
    #[allow(dead_code)]
    pub fn entity_index(&self, eid: &EntityID) -> Option<&EntityIndex> {
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

    pub fn comp_to_entity(&self) -> &[SmallVec<[EntityIndex; 4]>; COMP_TYPE_ARR_LEN] {
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

impl Extension {
    #[inline]
    pub fn id(&self) -> &ExtensionID {
        &self.id
    }

    #[inline]
    pub fn index(&self) -> &ExtensionIndex {
        &self.index
    }

    #[inline]
    pub fn devices(&self) -> &SmallVec<[DeviceID; 50]> {
        &self.devices
    }

    pub fn snapshot(&self) -> ExtensionSnapshot {
        ExtensionSnapshot {
            id: self.id.clone(),
            index: self.index,
            max_supported_component: self.msic,
            devices: self.devices.to_vec(),
        }
    }
}
