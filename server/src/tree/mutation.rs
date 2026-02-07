//! Handles ALL mutations to the DeviceTree
//! Nothing else should modify the DeviceTree
//!
//! Has a few important things to watch for:
//!  1. Good error handling
//!  2. ID validation (generational and bounds checking, using .group, .device, .ext)
//!  3. Internal side-effects (ex. updating device presence)
//!  4. External side-effects (persistence, query engine)

use super::{Device, DeviceTree, Entity, Extension, Group};
use crate::{
    core::{ClientManager, IglooError},
    ext::{ExtensionHandle, ExtensionRequest},
    query::QueryEngine,
    tree::{COMP_TYPE_ARR_LEN, Presense, TreeIDError, persist::TreePersistError},
};
use igloo_interface::{
    Component,
    id::{DeviceID, EntityID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

#[derive(thiserror::Error, Debug)]
pub enum TreeMutationError {
    #[error("Tree persist error: {0}")]
    Persist(#[from] TreePersistError),
    #[error("Tree ID error: {0}")]
    ID(#[from] TreeIDError),
    #[error("Extension {0} already attached. Cannot attach again.")]
    ExtensionAlreadyAttached(ExtensionID),
    #[error(
        "Bad entity registration. Extension expected index={2} but is index={3}. Device={0}, Entity={1}."
    )]
    BadEntityRegistration(DeviceID, EntityID, EntityIndex, EntityIndex),
}

// Extension Mutations
impl DeviceTree {
    pub fn attach_ext(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        mut handle: ExtensionHandle,
        channel: kanal::Sender<ExtensionRequest>,
    ) -> Result<ExtensionIndex, IglooError> {
        let xid = handle.id.clone();

        if self.ext_ref_lut.contains_key(&xid) {
            handle.kill();
            return Err(IglooError::DeviceTreeMutation(
                TreeMutationError::ExtensionAlreadyAttached(xid),
            ))?;
        }

        let devices = self
            .devices
            .iter()
            .filter(|d| d.owner == xid)
            .map(|d| d.id)
            .collect();

        let xindex = match self.attached_exts.iter().position(|f| f.is_none()) {
            Some(index) => index,
            None => {
                self.attached_exts.push(None);
                self.attached_exts.len() - 1
            }
        };

        let xindex = ExtensionIndex(xindex);
        handle.index = xindex;
        let process = handle.spawn();

        self.attached_exts[xindex.0] = Some(Extension {
            id: xid.clone(),
            index: xindex,
            channel,
            devices,
            process,
        });

        // link devices owned by this Extension
        for device in self.devices.iter_mut() {
            if device.owner == xid {
                device.owner_ref = Some(xindex);
            }
        }

        self.ext_ref_lut.insert(xid, xindex);

        engine.on_ext_attached(cm, self, self.ext(&xindex)?)?;

        Ok(xindex)
    }

    #[allow(dead_code)]
    pub fn detach_ext(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        index: ExtensionIndex,
        from_err: bool,
    ) -> Result<(), IglooError> {
        // make sure valid first
        self.ext(&index)?;

        if from_err {
            println!("{index}'s channel is full. This is likely a broken program.");
        }

        let ext = self.attached_exts[index.0].take().unwrap(); // FIXME unwrap
        let xid = &ext.id;
        self.ext_ref_lut.remove(xid);

        // notify the QueryEngine early, so it can still check device filters
        engine.on_ext_detached(cm, self, &ext)?;

        // unlink devices
        let now = Instant::now();
        for device in self.devices.iter_mut() {
            if device.owner_ref == Some(index) {
                device.reset();
                device.last_updated = now;
            }
        }

        // kill it
        ext.process.start_kill();

        Ok(())
    }
}

// Device Mutations
impl DeviceTree {
    pub fn create_device(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        name: String,
        owner: ExtensionIndex,
    ) -> Result<DeviceID, IglooError> {
        let ext = self.ext(&owner)?;

        // FIXME add device new function plz
        let device = Device {
            id: DeviceID::default(),
            name,
            owner: ext.id.clone(),
            owner_ref: Some(owner),
            groups: HashSet::with_capacity_and_hasher(10, FxBuildHasher),
            presense: Presense::default(),
            entities: SmallVec::default(),
            entity_index_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
            last_updated: Instant::now(),
            comp_to_entity: [const { SmallVec::new_const() }; COMP_TYPE_ARR_LEN],
        };

        let did = self.devices.insert(device);
        self.devices.get_mut(&did).unwrap().id = did;

        if let Some(ext) = self.attached_exts[owner.0].as_mut() {
            ext.devices.push(did);
        }

        self.save_devices()?;

        engine.on_device_created(cm, self, self.device(&did)?)?;

        Ok(did)
    }

    #[allow(dead_code)]
    pub fn delete_device(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        let Some(device) = self.devices.remove(did) else {
            return Err(IglooError::DeviceTreeID(TreeIDError::DeviceDeleted(did)));
        };

        // remove from ext
        if let Some(owner_ref) = device.owner_ref
            && let Some(ext) = self.attached_exts[owner_ref.0].as_mut()
            && let Some(pos) = ext.devices.iter().position(|d| d == &did)
        {
            ext.devices.remove(pos);
        }

        // remove from Groups
        for gid in &device.groups {
            if let Ok(group) = self.group_mut(gid) {
                group.devices.remove(&did);
            }
        }

        self.save_devices()?;

        engine.on_device_deleted(cm, self, &device)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn rename_device(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        did: DeviceID,
        new_name: String,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.name = new_name;
        device.last_updated = Instant::now();

        self.save_devices()?;

        engine.on_device_renamed(cm, self, self.device(&did)?)?;

        Ok(())
    }
}

// Entity Mutations
impl DeviceTree {
    pub fn register_entity(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        did: DeviceID,
        id: EntityID,
        expected_index: EntityIndex,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        let index = EntityIndex(device.entities.len());

        if index != expected_index {
            return Err(IglooError::DeviceTreeMutation(
                TreeMutationError::BadEntityRegistration(did, id, expected_index, index),
            ));
        }

        device.entities.push(Entity {
            id: id.clone(),
            index,
            ..Default::default()
        });
        device.entity_index_lut.insert(id, index);
        device.last_updated = Instant::now();

        engine.on_entity_registered(cm, self, self.device(&did)?, index)?;

        Ok(())
    }

    pub fn write_components(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        did: DeviceID,
        eindex: EntityIndex,
        comps: Vec<Component>,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.last_updated = Instant::now();

        for comp in comps {
            // FIXME super slow
            let device = self.device_mut(&did)?;
            let entity = &mut device.entities[eindex.0];

            let comp_type = comp.get_type();
            entity.last_updated = Instant::now();

            match entity.put(comp.clone()) {
                Some(comp_type) => {
                    device.presense.set(comp_type);
                    device.comp_to_entity[comp_type as usize].push(eindex);
                    engine.on_component_put(
                        cm,
                        self,
                        self.device(&did)?,
                        eindex,
                        comp_type,
                        comp,
                    )?;
                }
                None => {
                    engine.on_component_set(
                        cm,
                        self,
                        self.device(&did)?,
                        eindex,
                        comp_type,
                        comp,
                    )?;
                }
            }
        }

        Ok(())
    }
}

// Group Mutations
impl DeviceTree {
    #[allow(dead_code)]
    pub fn create_group(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        name: String,
    ) -> Result<GroupID, IglooError> {
        let group = Group {
            id: GroupID::default(),
            name,
            devices: HashSet::with_capacity_and_hasher(10, FxBuildHasher),
        };

        let gid = self.groups.insert(group);
        self.groups.get_mut(&gid).unwrap().id = gid;

        self.save_groups()?;

        engine.on_group_created(cm, self, self.group(&gid)?)?;

        Ok(gid)
    }

    #[allow(dead_code)]
    pub fn delete_group(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        gid: GroupID,
    ) -> Result<(), IglooError> {
        let Some(group) = self.groups.remove(gid) else {
            return Err(IglooError::DeviceTreeID(TreeIDError::GroupDeleted(gid)));
        };

        // remove from all devices
        for did in group.devices {
            if let Ok(device) = self.device_mut(&did) {
                device.groups.remove(&gid);
            }
        }

        self.save_groups()?;

        engine.on_group_deleted(cm, self, &gid)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn rename_group(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        gid: &GroupID,
        new_name: String,
    ) -> Result<(), IglooError> {
        let group = self.group_mut(gid)?;
        group.name = new_name;

        self.save_groups()?;

        engine.on_group_renamed(cm, self, self.group(gid)?)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn add_device_to_group(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.groups.insert(gid);

        let group = self.group_mut(&gid)?;
        group.devices.insert(did);

        self.save_groups()?;

        engine.on_group_device_added(cm, self, self.group(&gid)?, self.device(&did)?)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_device_from_group(
        &mut self,
        cm: &mut ClientManager,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.groups.remove(&gid);

        let group = self.group_mut(&gid)?;
        group.devices.remove(&did);

        self.save_groups()?;

        engine.on_group_device_removed(cm, self, self.group(&gid)?, self.device(&did)?)?;

        Ok(())
    }
}
