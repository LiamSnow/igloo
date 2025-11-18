//! Handles ALL mutations to the DeviceTree
//! Nothing else should modify the DeviceTree
//!
//! Has a few important things to watch for:
//!  1. Good error handling
//!  2. ID validation (generational and bounds checking, using .group, .device, .floe)
//!  3. Internal side-effects (ex. updating device presence)
//!  4. External side-effects (persistence, query engine)

use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use super::{Device, DeviceTree, Entity, Floe, Group};
use crate::glacier::{
    query::{EngineError, QueryEngine},
    tree::{Presense, TreeIDError, persist::TreePersistError},
};
use igloo_interface::{
    Component,
    floe::FloeWriterDefault,
    id::{DeviceID, FloeID, FloeRef, GroupID},
};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

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

/// Result of a tree mutation
/// Used by side-effects (currently only query engine)
#[allow(dead_code)]
pub enum TreeMutation {
    FloeAttached {
        fid: FloeID,
        fref: FloeRef,
        max_supported_component: u16,
    },
    FloeDetached {
        fid: FloeID,
        fref: FloeRef,
    },

    DeviceCreated {
        did: DeviceID,
        name: String,
        owner: FloeID,
    },
    DeviceDeleted {
        did: DeviceID,
    },
    DeviceRenamed {
        did: DeviceID,
        old_name: String,
        new_name: String,
    },

    EntityRegistered {
        did: DeviceID,
        name: String,
        index: usize,
    },
    ComponentWritten {
        did: DeviceID,
        eindex: usize,
        comp: Component,
        was_new: bool,
    },

    GroupCreated {
        gid: GroupID,
        name: String,
    },
    GroupDeleted {
        gid: GroupID,
    },
    GroupRenamed {
        gid: GroupID,
        old_name: String,
        new_name: String,
    },
    DeviceAddedToGroup {
        gid: GroupID,
        did: DeviceID,
    },
    DeviceRemovedFromGroup {
        gid: GroupID,
        did: DeviceID,
    },
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

        let mutation = TreeMutation::FloeAttached {
            fid,
            fref,
            max_supported_component,
        };
        engine.on_tree_mutation(mutation).await?;

        Ok(fref)
    }

    #[allow(dead_code)]
    pub async fn detach_floe(
        &mut self,
        engine: &mut QueryEngine,
        fref: FloeRef,
    ) -> Result<(), TreeMutationError> {
        // make sure valid first
        self.floe(&fref)?;

        let floe = self.floes[fref.0].take().unwrap();

        let fid = floe.id.clone();
        self.floe_ref_lut.remove(&fid);

        // unlink devices
        for device in &mut self.devices {
            if let Some(d) = device.as_mut()
                && d.owner_ref == Some(fref)
            {
                d.owner_ref = None;
            }
        }

        let mutation = TreeMutation::FloeDetached { fid, fref };
        engine.on_tree_mutation(mutation).await?;

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
        let owner_id = floe.id.clone();

        let mut device = Device {
            id: DeviceID::default(),
            name: name.clone(),
            owner: owner_id.clone(),
            owner_ref: Some(owner),
            groups: HashSet::with_capacity(10),
            presense: Presense::default(),
            entities: SmallVec::default(),
            entity_index_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
            last_updated: Instant::now(),
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

        let mutation = TreeMutation::DeviceCreated {
            did,
            name,
            owner: owner_id,
        };
        self.save_devices().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(did)
    }

    #[allow(dead_code)]
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
        let group_ids: Vec<GroupID> = device.groups.iter().copied().collect();
        for gid in group_ids {
            if let Ok(group) = self.group_mut(&gid) {
                group.devices.remove(&did);
            }
        }

        let mutation = TreeMutation::DeviceDeleted { did };
        self.save_devices().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn rename_device(
        &mut self,
        engine: &mut QueryEngine,
        did: DeviceID,
        new_name: String,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(&did)?;
        let old_name = device.name.clone();
        device.name = new_name.clone();
        device.last_updated = Instant::now();

        let mutation = TreeMutation::DeviceRenamed {
            did,
            old_name,
            new_name,
        };
        self.save_devices().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }
}

// Entity Mutations
impl DeviceTree {
    pub async fn register_entity(
        engine: &mut QueryEngine,
        device: &mut Device,
        did: DeviceID,
        name: String,
        expected_index: usize,
    ) -> Result<(), TreeMutationError> {
        let index = device.entities.len();

        if index != expected_index {
            return Err(TreeMutationError::BadEntityRegistration(
                did,
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
        device.entity_index_lut.insert(name.clone(), index);
        device.last_updated = Instant::now();

        let mutation = TreeMutation::EntityRegistered { did, name, index };
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }

    pub async fn write_component(
        engine: &mut QueryEngine,
        device: &mut Device,
        did: DeviceID,
        eindex: usize,
        comp: Component,
    ) -> Result<(), TreeMutationError> {
        let was_new = match device.entities[eindex].put(comp.clone()) {
            Some(comp_type) => {
                device.presense.set(comp_type);
                true
            }
            None => false,
        };

        device.entities[eindex].last_updated = Instant::now();
        device.last_updated = Instant::now();

        let mutation = TreeMutation::ComponentWritten {
            did,
            eindex,
            comp,
            was_new,
        };
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }
}

// Group Mutations
impl DeviceTree {
    #[allow(dead_code)]
    pub async fn create_group(
        &mut self,
        engine: &mut QueryEngine,
        name: String,
    ) -> Result<GroupID, TreeMutationError> {
        let mut group = Group {
            id: GroupID::default(),
            name: name.clone(),
            devices: HashSet::with_capacity(10),
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

        let mutation = TreeMutation::GroupCreated { gid, name };
        self.save_groups().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(gid)
    }

    #[allow(dead_code)]
    pub async fn delete_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
    ) -> Result<(), TreeMutationError> {
        // make sure its valid first
        self.group(&gid)?;

        let group = self.groups[gid.index() as usize].take().unwrap();

        // remove from all devices
        let device_ids: Vec<DeviceID> = group.devices.iter().copied().collect();
        for did in device_ids {
            if let Ok(device) = self.device_mut(&did) {
                device.groups.remove(&gid);
            }
        }

        let mutation = TreeMutation::GroupDeleted { gid };
        self.save_groups().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn rename_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        new_name: String,
    ) -> Result<(), TreeMutationError> {
        let group = self.group_mut(&gid)?;
        let old_name = group.name.clone();
        group.name = new_name.clone();

        let mutation = TreeMutation::GroupRenamed {
            gid,
            old_name,
            new_name,
        };
        self.save_groups().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn add_device_to_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), TreeMutationError> {
        let device = self.device_mut(&did)?;

        // add group to device
        device.groups.insert(gid);

        // add device to group
        let group = self.group_mut(&gid)?;
        group.devices.remove(&did);

        let mutation = TreeMutation::DeviceAddedToGroup { gid, did };
        self.save_groups().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_device_from_group(
        &mut self,
        engine: &mut QueryEngine,
        gid: GroupID,
        did: DeviceID,
    ) -> Result<(), TreeMutationError> {
        // remove group from device
        let device = self.device_mut(&did)?;
        device.groups.remove(&gid);

        // remove device from group
        let group = self.group_mut(&gid)?;
        group.devices.remove(&did);

        let mutation = TreeMutation::DeviceRemovedFromGroup { gid, did };
        self.save_groups().await?;
        engine.on_tree_mutation(mutation).await?;

        Ok(())
    }
}
