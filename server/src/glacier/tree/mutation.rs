//! Handles ALL mutations to the DeviceTree
//! Nothing else should modify the DeviceTree
//!
//! Has a few important things to watch for:
//!  1. Good error handling
//!  2. ID validation (generational and bounds checking, using .group, .device, .floe)
//!  3. Internal side-effects (ex. updating device presence)
//!  4. External side-effects (persistence, query engine)

use super::{Device, DeviceTree, Entity, Floe, Group};
use crate::glacier::{
    query::{EngineError, QueryEngine},
    tree::{COMP_TYPE_ARR_LEN, Presense, TreeIDError, persist::TreePersistError},
};
use igloo_interface::{
    Component,
    floe::FloeWriterDefault,
    id::{DeviceID, FloeID, FloeRef, GroupID},
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
    #[error("Query engine error: {0}")]
    QueryEngine(#[from] EngineError),
    #[error("Floe {0} already attached. Cannot attach again.")]
    FloeAlreadyAttached(FloeID),
    #[error(
        "Bad entity registration. Floe expected index={2} but is index={3}. Device={0}, Entity={1}."
    )]
    BadEntityRegistration(DeviceID, String, usize, usize),
}

// Floe Mutations
impl DeviceTree {
    pub async fn attach_floe(
        &mut self,
        engine: &mut QueryEngine,
        fid: FloeID,
        writer: FloeWriterDefault,
        max_supported_component: u16,
    ) -> Result<FloeRef, TreeMutationError> {
        if self.floe_ref_lut.contains_key(&fid) {
            return Err(TreeMutationError::FloeAlreadyAttached(fid));
        }

        let devices = self
            .devices
            .iter()
            .filter_map(|d| d.as_ref())
            .filter(|d| d.owner == fid)
            .map(|d| d.id)
            .collect();

        let fref = if let Some(slot) = self.floes.iter().position(|f| f.is_none()) {
            let fref = FloeRef(slot);
            self.floes[slot] = Some(Floe {
                id: fid.clone(),
                fref,
                writer,
                max_supported_component,
                devices,
            });
            fref
        } else {
            let index = self.floes.len();
            let fref = FloeRef(index);
            self.floes.push(Some(Floe {
                id: fid.clone(),
                fref,
                writer,
                max_supported_component,
                devices,
            }));
            fref
        };

        // link devices owned by this Floe
        for device in &mut self.devices {
            if let Some(d) = device.as_mut()
                && d.owner == fid
            {
                d.owner_ref = Some(fref);
            }
        }

        self.floe_ref_lut.insert(fid.clone(), fref);

        engine.on_floe_attached(self, self.floe(&fref)?).await?;

        Ok(fref)
    }

    pub async fn detach_floe(
        &mut self,
        engine: &mut QueryEngine,
        fref: FloeRef,
    ) -> Result<(), TreeMutationError> {
        // make sure valid first
        self.floe(&fref)?;

        let floe = self.floes[fref.0].take().unwrap(); // FIXME unwrap
        let fid = &floe.id;
        self.floe_ref_lut.remove(fid);

        // unlink devices
        for device in &mut self.devices {
            if let Some(d) = device.as_mut()
                && d.owner_ref == Some(fref)
            {
                d.owner_ref = None;
            }
        }

        engine.on_floe_detached(self, &fref).await?;

        Ok(())
    }
}

// Device Mutations
impl DeviceTree {
    pub async fn create_device(
        &mut self,
        engine: &mut QueryEngine,
        name: String,
        owner: FloeRef,
    ) -> Result<DeviceID, TreeMutationError> {
        let floe = self.floe(&owner)?;

        // FIXME add device new function plz
        let mut device = Device {
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

        let did = match self.devices.iter().position(|o| o.is_none()) {
            Some(free_slot) => {
                self.device_generation += 1;
                let did = DeviceID::from_parts(free_slot as u32, self.device_generation);
                device.id = did;
                self.devices[free_slot] = Some(device);
                did
            }
            None => {
                let did = DeviceID::from_parts(self.devices.len() as u32, self.device_generation);
                device.id = did;
                self.devices.push(Some(device));
                did
            }
        };

        if let Some(floe) = self.floes[owner.0].as_mut() {
            floe.devices.push(did);
        }

        self.save_devices().await?;

        engine.on_device_created(self, self.device(&did)?).await?;

        Ok(did)
    }

    pub async fn delete_device(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
    ) -> Result<(), TreeMutationError> {
        // make sure its valid first
        self.device(&did)?;

        let device = self.devices[did.index() as usize].take().unwrap();

        // remove from Floe
        if let Some(owner_ref) = device.owner_ref
            && let Some(floe) = self.floes[owner_ref.0].as_mut()
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

        self.save_devices().await?;

        engine.on_device_deleted(self, &device).await?;

        Ok(())
    }

    pub async fn rename_device(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
        new_name: String,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(&did)?;
        device.name = new_name;
        device.last_updated = Instant::now();

        self.save_devices().await?;

        engine.on_device_renamed(self, self.device(&did)?).await?;

        Ok(())
    }
}

// Entity Mutations
impl DeviceTree {
    pub async fn register_entity(
        &mut self,
        engine: &mut QueryEngine,
        did: &DeviceID,
        name: String,
        expected_index: usize,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(did)?;
        let index = device.entities.len();

        if index != expected_index {
            return Err(TreeMutationError::BadEntityRegistration(
                *did,
                name,
                expected_index,
                index,
            ));
        }

        device.entities.push(Entity {
            name: name.clone(),
            index,
            ..Default::default()
        });
        device.entity_index_lut.insert(name, index);
        device.last_updated = Instant::now();

        engine
            .on_entity_registered(self, self.device(did)?, index)
            .await?;

        Ok(())
    }

    pub async fn write_component(
        &mut self,
        engine: &mut QueryEngine,
        did: &DeviceID,
        eindex: usize,
        comp: Component,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(did)?;
        device.last_updated = Instant::now();
        let entity = &mut device.entities[eindex];
        let comp_type = comp.get_type();

        let was_put = match entity.put(comp) {
            Some(comp_type) => {
                device.presense.set(comp_type);
                device.comp_to_entity[comp_type as usize].push(eindex);
                true
            }
            None => false,
        };

        entity.last_updated = Instant::now();

        if was_put {
            engine
                .on_component_put(self, self.device(&did)?, eindex, comp_type)
                .await?;
        } else {
            engine
                .on_component_set(self, self.device(&did)?, eindex, comp_type)
                .await?;
        }

        Ok(())
    }
}

// Group Mutations
impl DeviceTree {
    pub async fn create_group(
        &mut self,
        engine: &mut QueryEngine,
        name: String,
    ) -> Result<GroupID, TreeMutationError> {
        let mut group = Group {
            id: GroupID::default(),
            name,
            devices: HashSet::with_capacity_and_hasher(10, FxBuildHasher),
        };

        let gid = match self.groups.iter().position(|g| g.is_none()) {
            Some(free_slot) => {
                self.group_generation += 1;
                let gid = GroupID::from_parts(free_slot as u32, self.group_generation);
                group.id = gid;
                self.groups[free_slot] = Some(group);
                gid
            }
            None => {
                let gid = GroupID::from_parts(self.groups.len() as u32, self.group_generation);
                group.id = gid;
                self.groups.push(Some(group));
                gid
            }
        };

        self.save_groups().await?;

        engine.on_group_created(self, self.group(&gid)?).await?;

        Ok(gid)
    }

    pub async fn delete_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: &GroupID,
    ) -> Result<(), TreeMutationError> {
        // make sure its valid first
        self.group(&gid)?;

        let group = self.groups[gid.index() as usize].take().unwrap();

        // remove from all devices
        for did in group.devices {
            if let Ok(device) = self.device_mut(&did) {
                device.groups.remove(&gid);
            }
        }

        self.save_groups().await?;

        engine.on_group_deleted(self, gid).await?;

        Ok(())
    }

    pub async fn rename_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: &GroupID,
        new_name: String,
    ) -> Result<(), TreeMutationError> {
        let group = self.group_mut(gid)?;
        group.name = new_name;

        self.save_groups().await?;

        engine.on_group_renamed(self, self.group(gid)?).await?;

        Ok(())
    }

    pub async fn add_device_to_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(&did)?;
        device.groups.insert(gid);

        let group = self.group_mut(&gid)?;
        group.devices.insert(did);

        self.save_groups().await?;

        engine
            .on_group_membership_changed(self, self.group(&gid)?, self.device(&did)?)
            .await?;

        Ok(())
    }

    pub async fn remove_device_from_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(&did)?;
        device.groups.remove(&gid);

        let group = self.group_mut(&gid)?;
        group.devices.remove(&did);

        self.save_groups().await?;

        engine
            .on_group_membership_changed(self, self.group(&gid)?, self.device(&did)?)
            .await?;

        Ok(())
    }
}
