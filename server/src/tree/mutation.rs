//! Handles ALL mutations to the DeviceTree
//! Nothing else should modify the DeviceTree
//!
//! Has a few important things to watch for:
//!  1. Good error handling
//!  2. ID validation (generational and bounds checking, using .group, .device, .floe)
//!  3. Internal side-effects (ex. updating device presence)
//!  4. External side-effects (persistence, query engine)

use super::{Device, DeviceTree, Entity, Floe, Group};
use crate::{
    core::IglooError,
    floe::FloeHandle,
    query::QueryEngine,
    tree::{COMP_TYPE_ARR_LEN, Presense, TreeIDError, persist::TreePersistError},
};
use igloo_interface::{
    Component,
    id::{DeviceID, FloeID, FloeRef, GroupID},
    ipc::IWriter,
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
    #[error("Floe {0} already attached. Cannot attach again.")]
    FloeAlreadyAttached(FloeID),
    #[error(
        "Bad entity registration. Floe expected index={2} but is index={3}. Device={0}, Entity={1}."
    )]
    BadEntityRegistration(DeviceID, String, usize, usize),
}

// Floe Mutations
impl DeviceTree {
    pub fn attach_floe(
        &mut self,
        engine: &mut QueryEngine,
        mut handle: FloeHandle,
        writer: IWriter,
    ) -> Result<FloeRef, IglooError> {
        let id = handle.id.clone();
        let msic = handle.msic;
        let msim = handle.msim;

        if self.floe_ref_lut.contains_key(&id) {
            return Err(IglooError::DeviceTreeMutation(
                TreeMutationError::FloeAlreadyAttached(id),
            ))?;
        }

        let devices = self
            .devices
            .iter()
            .filter(|d| d.owner == id)
            .map(|d| d.id)
            .collect();

        let index = match self.attached_floes.iter().position(|f| f.is_none()) {
            Some(index) => index,
            None => {
                self.attached_floes.push(None);
                self.attached_floes.len() - 1
            }
        };

        let fref = FloeRef(index);
        handle.fref = fref;
        let handle = handle.spawn();

        self.attached_floes[index] = Some(Floe {
            id: id.clone(),
            fref,
            writer,
            devices,
            handle,
            msic,
            msim,
        });

        // link devices owned by this Floe
        for device in self.devices.iter_mut() {
            if device.owner == id {
                device.owner_ref = Some(fref);
            }
        }

        self.floe_ref_lut.insert(id, fref);

        engine.on_floe_attached(self, self.floe(&fref)?)?;

        Ok(fref)
    }

    #[allow(dead_code)]
    pub fn detach_floe(
        &mut self,
        engine: &mut QueryEngine,
        fref: FloeRef,
    ) -> Result<(), IglooError> {
        // make sure valid first
        self.floe(&fref)?;

        let floe = self.attached_floes[fref.0].take().unwrap(); // FIXME unwrap
        let fid = &floe.id;
        self.floe_ref_lut.remove(fid);

        // unlink devices
        for device in self.devices.iter_mut() {
            if device.owner_ref == Some(fref) {
                device.owner_ref = None;
            }
        }

        // kill it
        floe.handle.abort();

        engine.on_floe_detached(self, &fref)?;

        Ok(())
    }
}

// Device Mutations
impl DeviceTree {
    pub fn create_device(
        &mut self,
        engine: &mut QueryEngine,
        name: String,
        owner: FloeRef,
    ) -> Result<DeviceID, IglooError> {
        let floe = self.floe(&owner)?;

        // FIXME add device new function plz
        let device = Device {
            id: DeviceID::default(),
            name,
            owner: floe.id.clone(),
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

        if let Some(floe) = self.attached_floes[owner.0].as_mut() {
            floe.devices.push(did);
        }

        self.save_devices()?;

        engine.on_device_created(self, self.device(&did)?)?;

        Ok(did)
    }

    #[allow(dead_code)]
    pub fn delete_device(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        let Some(device) = self.devices.remove(did) else {
            return Err(IglooError::DeviceTreeID(TreeIDError::DeviceDeleted(did)));
        };

        // remove from Floe
        if let Some(owner_ref) = device.owner_ref
            && let Some(floe) = self.attached_floes[owner_ref.0].as_mut()
            && let Some(pos) = floe.devices.iter().position(|d| d == &did)
        {
            floe.devices.remove(pos);
        }

        // remove from Groups
        for gid in &device.groups {
            if let Ok(group) = self.group_mut(gid) {
                group.devices.remove(&did);
            }
        }

        self.save_devices()?;

        engine.on_device_deleted(self, &device)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn rename_device(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
        new_name: String,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.name = new_name;
        device.last_updated = Instant::now();

        self.save_devices()?;

        engine.on_device_renamed(self, self.device(&did)?)?;

        Ok(())
    }
}

// Entity Mutations
impl DeviceTree {
    pub fn register_entity(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
        name: String,
        expected_index: usize,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        let index = device.entities.len();

        if index != expected_index {
            Err(TreeMutationError::BadEntityRegistration(
                did,
                name.clone(),
                expected_index,
                index,
            ))?
        }

        device.entities.push(Entity {
            name: name.clone(),
            index,
            ..Default::default()
        });
        device.entity_index_lut.insert(name, index);
        device.last_updated = Instant::now();

        engine.on_entity_registered(self, self.device(&did)?, index)?;

        Ok(())
    }

    pub fn write_components(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
        eindex: usize,
        comps: Vec<Component>,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.last_updated = Instant::now();

        for comp in comps {
            // FIXME super slow
            let device = self.device_mut(&did)?;
            let entity = &mut device.entities[eindex];

            let comp_type = comp.get_type();
            entity.last_updated = Instant::now();

            match entity.put(comp) {
                Some(comp_type) => {
                    device.presense.set(comp_type);
                    device.comp_to_entity[comp_type as usize].push(eindex);
                    engine.on_component_put(self, self.device(&did)?, eindex, comp_type)?;
                }
                None => {
                    engine.on_component_set(self, self.device(&did)?, eindex, comp_type)?;
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

        engine.on_group_created(self, self.group(&gid)?)?;

        Ok(gid)
    }

    #[allow(dead_code)]
    pub fn delete_group(
        &mut self,
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

        engine.on_group_deleted(self, &gid)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn rename_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: &GroupID,
        new_name: String,
    ) -> Result<(), IglooError> {
        let group = self.group_mut(gid)?;
        group.name = new_name;

        self.save_groups()?;

        engine.on_group_renamed(self, self.group(gid)?)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn add_device_to_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.groups.insert(gid);

        let group = self.group_mut(&gid)?;
        group.devices.insert(did);

        self.save_groups()?;

        engine.on_group_membership_changed(self, self.group(&gid)?, self.device(&did)?)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_device_from_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        let device = self.device_mut(&did)?;
        device.groups.remove(&gid);

        let group = self.group_mut(&gid)?;
        group.devices.remove(&did);

        self.save_groups()?;

        engine.on_group_membership_changed(self, self.group(&gid)?, self.device(&did)?)?;

        Ok(())
    }
}
