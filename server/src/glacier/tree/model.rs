use igloo_interface::{
    Component, ComponentType, MAX_SUPPORTED_COMPONENT,
    floe::FloeWriterDefault,
    id::{DeviceID, FloeID, FloeRef, GroupID},
};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

/// Root
/// WARN: Mutations to the device tree must only occur in `mutation.rs`
#[derive(Debug, Default)]
pub struct DeviceTree {
    pub(super) groups: Vec<Option<Group>>,
    pub(super) floes: Vec<Option<Floe>>,
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
    pub(super) writer: FloeWriterDefault,
    #[allow(dead_code)]
    pub(super) max_supported_component: u16,
}

/// Collection of devices (ex. "Living Room")
#[derive(Debug, Clone)]
pub struct Group {
    pub(super) id: GroupID,
    pub(super) name: String,
    pub(super) devices: SmallVec<[DeviceID; 20]>,
}

/// Physical Device, owned (registered & attached) by Floe
#[derive(Debug, Default, Clone)]
pub struct Device {
    pub(super) id: DeviceID,
    pub(super) name: String,
    pub(super) owner: FloeID,
    pub(super) owner_ref: Option<FloeRef>,
    /// Bitset of which component types exist on any of its entity
    pub(super) presense: Presense,
    /// Entity index -> Entity
    pub(super) entities: SmallVec<[Entity; 16]>,
    /// Entity name -> index
    pub(super) entity_idx_lut: FxHashMap<String, usize>,
}

/// Bitset tracking component presence on a device
/// Each bit corresponds to a `ComponentType` ID
#[derive(Debug, Default, Clone)]
pub(crate) struct Presense([u32; MAX_SUPPORTED_COMPONENT.div_ceil(32) as usize]);

#[derive(Debug, Clone)]
pub struct Entity {
    #[allow(dead_code)]
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) idx: usize,
    pub(crate) components: SmallVec<[Component; 8]>,
    /// Maps `ComponentType` ID -> index in `components`
    /// `0xFF` = not present
    pub(crate) indices: [u8; MAX_SUPPORTED_COMPONENT as usize],
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

impl Presense {
    #[inline]
    pub fn set(&mut self, typ: ComponentType) {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        self.0[index] |= 1u32 << bit;
    }

    #[inline]
    #[allow(dead_code)]
    pub fn has(&self, typ: ComponentType) -> bool {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        (self.0[index] & (1u32 << bit)) != 0
    }
}

impl HasComponent for Presense {
    #[inline]
    fn has(&self, typ: ComponentType) -> bool {
        self.has(typ)
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            components: SmallVec::new(),
            indices: [0xFF; MAX_SUPPORTED_COMPONENT as usize],
            name: String::with_capacity(20),
            idx: usize::MAX,
        }
    }
}

impl Entity {
    #[allow(dead_code)]
    pub fn get_comps(&self) -> &SmallVec<[Component; 8]> {
        &self.components
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn get(&self, typ: ComponentType) -> Option<&Component> {
        let idx = self.indices[typ as usize];
        if idx != 0xFF {
            Some(&self.components[idx as usize])
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
        let idx = self.indices[type_id];

        if idx == 0xFF {
            // put
            let new_idx = self.components.len() as u8;
            self.components.push(val);
            self.indices[type_id] = new_idx;
            Some(typ)
        } else {
            // set
            self.components[idx as usize] = val;
            None
        }
    }
}

impl HasComponent for Entity {
    #[inline(always)]
    fn has(&self, typ: ComponentType) -> bool {
        self.indices[typ as usize] != 0xFF
    }
}

#[allow(dead_code)]
pub trait HasComponent {
    fn has(&self, typ: ComponentType) -> bool;
}

impl DeviceTree {
    #[allow(dead_code)]
    pub fn iter_groups(&self) -> impl Iterator<Item = (GroupID, &Group)> {
        self.groups.iter().enumerate().filter_map(|(idx, g)| {
            g.as_ref().map(|group| {
                (
                    GroupID::from_parts(idx as u32, group.id.generation()),
                    group,
                )
            })
        })
    }

    #[allow(dead_code)]
    pub fn iter_devices(&self) -> impl Iterator<Item = (DeviceID, &Device)> {
        self.devices.iter().enumerate().filter_map(|(idx, d)| {
            d.as_ref().map(|device| {
                (
                    DeviceID::from_parts(idx as u32, device.id.generation()),
                    device,
                )
            })
        })
    }

    #[allow(dead_code)]
    pub fn iter_devices_in_group(
        &self,
        gid: &GroupID,
    ) -> Result<impl Iterator<Item = Result<(DeviceID, &Device), TreeIDError>> + '_, TreeIDError>
    {
        let group = self.group(gid)?;
        Ok(group.devices.iter().map(move |did| {
            let device = self.device(did)?;
            Ok((*did, device))
        }))
    }

    /// Gets & Validates from DeviceID
    #[inline]
    pub fn device(&self, did: &DeviceID) -> Result<&Device, TreeIDError> {
        let idx = did.idx() as usize;
        match self.devices.get(idx).and_then(|o| o.as_ref()) {
            Some(d) if d.id.generation() == did.generation() => Ok(d),
            Some(_) => Err(TreeIDError::DeviceStale(*did)),
            None => Err(TreeIDError::DeviceNotExistant(*did)),
        }
    }

    /// Gets & Validates from DeviceID
    #[inline]
    pub fn device_mut(&mut self, did: &DeviceID) -> Result<&mut Device, TreeIDError> {
        let idx = did.idx() as usize;
        match self.devices.get_mut(idx).and_then(|o| o.as_mut()) {
            Some(d) if d.id.generation() == did.generation() => Ok(d),
            Some(_) => Err(TreeIDError::DeviceStale(*did)),
            None => Err(TreeIDError::DeviceNotExistant(*did)),
        }
    }

    /// Gets & Validates from GroupID
    #[inline]
    pub fn group(&self, gid: &GroupID) -> Result<&Group, TreeIDError> {
        match self.groups.get(gid.idx() as usize).and_then(|o| o.as_ref()) {
            Some(g) if g.id.generation() == gid.generation() => Ok(g),
            Some(_) => Err(TreeIDError::GroupStale(*gid)),
            None => Err(TreeIDError::GroupNotExistant(*gid)),
        }
    }

    /// Gets & Validates from GroupID
    #[inline]
    pub(super) fn group_mut(&mut self, gid: &GroupID) -> Result<&mut Group, TreeIDError> {
        let idx = gid.idx() as usize;
        match self.groups.get_mut(idx).and_then(|o| o.as_mut()) {
            Some(g) if g.id.generation() == gid.generation() => Ok(g),
            Some(_) => Err(TreeIDError::GroupStale(*gid)),
            None => Err(TreeIDError::GroupNotExistant(*gid)),
        }
    }

    #[inline]
    pub fn floe(&self, fref: &FloeRef) -> Result<&Floe, TreeIDError> {
        match self.floes.get(fref.0) {
            Some(o) => match o.as_ref() {
                Some(f) => Ok(f),
                None => Err(TreeIDError::FloeDetached(*fref)),
            },
            None => Err(TreeIDError::FloeRefOutOfBounds(*fref)),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub(super) fn floe_mut(&mut self, fref: &FloeRef) -> Result<&mut Floe, TreeIDError> {
        match self.floes.get_mut(fref.0) {
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

    #[inline]
    pub fn floe_writer_mut(&mut self, fref: FloeRef) -> &mut FloeWriterDefault {
        &mut self.floes[fref.0].as_mut().unwrap().writer
    }
}

impl Device {
    #[inline]
    #[allow(dead_code)]
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
    pub fn entity_idx(&self, eid: &str) -> Option<&usize> {
        self.entity_idx_lut.get(eid)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn num_entities(&self) -> usize {
        self.entities.len()
    }
}

impl Group {
    #[inline]
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    #[allow(dead_code)]
    pub fn devices(&self) -> &SmallVec<[DeviceID; 20]> {
        &self.devices
    }
}
