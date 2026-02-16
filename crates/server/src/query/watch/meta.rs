use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryContext,
        watch::{WatcherID, dispatch::TreeEventResponder, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{DeviceID, EntityIndex, ExtensionID, GroupID},
    query::{DeviceMetadata, ExtensionMetadata, GroupMetadata, MetadataUpdate as U, WatchUpdate},
};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::collections::HashMap;

pub struct MetadataWatcher {
    pub id: WatcherID,
    pub subs: Vec<(usize, usize)>,

    devices: FxHashMap<DeviceID, DeviceMetadata>,
    groups: FxHashMap<GroupID, GroupMetadata>,
    exts: FxHashMap<ExtensionID, ExtensionMetadata>,
}

impl MetadataWatcher {
    pub fn register(tree: &DeviceTree, subs: &mut TreeSubscribers, id: WatcherID) -> Self {
        subs.device_created.all.push(id);
        subs.device_deleted.all.push(id);
        subs.device_renamed.all.push(id);

        subs.group_created.all.push(id);
        subs.group_deleted.all.push(id);
        subs.group_renamed.all.push(id);
        subs.group_device_added.all.push(id);
        subs.group_device_removed.all.push(id);

        subs.ext_attached.all.push(id);
        subs.ext_detached.all.push(id);

        let mut devices = HashMap::with_capacity_and_hasher(20, FxBuildHasher);
        let mut groups = HashMap::with_capacity_and_hasher(5, FxBuildHasher);
        let mut exts = HashMap::with_capacity_and_hasher(3, FxBuildHasher);

        for device in tree.devices().iter() {
            devices.insert(
                *device.id(),
                DeviceMetadata {
                    name: device.name().to_string(),
                },
            );
        }

        for group in tree.groups().iter() {
            groups.insert(
                *group.id(),
                GroupMetadata {
                    name: group.name().to_string(),
                    devices: group.devices().iter().copied().collect(),
                },
            );
        }

        for ext in tree.exts().iter() {
            if let Some(ext) = ext {
                exts.insert(
                    ext.id().clone(),
                    ExtensionMetadata {
                        index: *ext.index(),
                        devices: ext.devices().iter().copied().collect(),
                    },
                );
            }
        }

        Self {
            id,
            subs: Vec::with_capacity(5),
            devices,
            groups,
            exts,
        }
    }

    pub fn on_sub(
        &mut self,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
    ) -> Result<(), IglooError> {
        self.subs.push((client_id, query_id));

        let mut batch =
            Vec::with_capacity(self.devices.len() + self.groups.len() + self.exts.len());

        for (id, metadata) in &self.devices {
            batch.push(U::Device(*id, metadata.clone()));
        }

        for (id, metadata) in &self.groups {
            batch.push(U::Group(*id, metadata.clone()));
        }

        for (id, metadata) in &self.exts {
            batch.push(U::Extension(id.clone(), metadata.clone()));
        }

        cm.send(
            client_id,
            IglooResponse::WatchUpdate {
                query_id,
                value: WatchUpdate::Metadata(batch),
            },
        )
    }

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        subs.device_created.all.retain(|&id| id != self.id);
        subs.device_deleted.all.retain(|&id| id != self.id);
        subs.device_renamed.all.retain(|&id| id != self.id);

        subs.group_created.all.retain(|&id| id != self.id);
        subs.group_deleted.all.retain(|&id| id != self.id);
        subs.group_renamed.all.retain(|&id| id != self.id);
        subs.group_device_added.all.retain(|&id| id != self.id);
        subs.group_device_removed.all.retain(|&id| id != self.id);

        subs.ext_attached.all.retain(|&id| id != self.id);
        subs.ext_detached.all.retain(|&id| id != self.id);
    }

    fn broadcast(&self, cm: &mut ClientManager, update: U) -> Result<(), IglooError> {
        for (client_id, query_id) in &self.subs {
            cm.send(
                *client_id,
                IglooResponse::WatchUpdate {
                    query_id: *query_id,
                    value: WatchUpdate::Metadata(vec![update.clone()]),
                },
            )?;
        }
        Ok(())
    }
}

impl TreeEventResponder for MetadataWatcher {
    fn on_device_created(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        let metadata = DeviceMetadata {
            name: device.name().to_string(),
        };

        self.devices.insert(*device.id(), metadata.clone());
        self.broadcast(cm, U::Device(*device.id(), metadata))
    }

    fn on_device_deleted(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.devices.remove(device.id());
        self.broadcast(cm, U::DeviceRemoved(*device.id()))
    }

    fn on_device_renamed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        let metadata = DeviceMetadata {
            name: device.name().to_string(),
        };

        self.devices.insert(*device.id(), metadata.clone());
        self.broadcast(cm, U::Device(*device.id(), metadata))
    }

    fn on_group_created(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError> {
        let metadata = GroupMetadata {
            name: group.name().to_string(),
            devices: group.devices().iter().copied().collect(),
        };

        self.groups.insert(*group.id(), metadata.clone());
        self.broadcast(cm, U::Group(*group.id(), metadata))
    }

    fn on_group_deleted(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        gid: &GroupID,
    ) -> Result<(), IglooError> {
        self.groups.remove(gid);
        self.broadcast(cm, U::GroupRemoved(*gid))
    }

    fn on_group_renamed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError> {
        let metadata = GroupMetadata {
            name: group.name().to_string(),
            devices: group.devices().iter().copied().collect(),
        };

        self.groups.insert(*group.id(), metadata.clone());
        self.broadcast(cm, U::Group(*group.id(), metadata))
    }

    fn on_group_device_added(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
        _device: &Device,
    ) -> Result<(), IglooError> {
        let metadata = GroupMetadata {
            name: group.name().to_string(),
            devices: group.devices().iter().copied().collect(),
        };

        self.groups.insert(*group.id(), metadata.clone());
        self.broadcast(cm, U::Group(*group.id(), metadata))
    }

    fn on_group_device_removed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
        _device: &Device,
    ) -> Result<(), IglooError> {
        let metadata = GroupMetadata {
            name: group.name().to_string(),
            devices: group.devices().iter().copied().collect(),
        };

        self.groups.insert(*group.id(), metadata.clone());
        self.broadcast(cm, U::Group(*group.id(), metadata))
    }

    fn on_ext_attached(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        let metadata = ExtensionMetadata {
            index: *ext.index(),
            devices: ext.devices().iter().copied().collect(),
        };

        self.exts.insert(ext.id().clone(), metadata.clone());
        self.broadcast(cm, U::Extension(ext.id().clone(), metadata))
    }

    fn on_ext_detached(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        self.exts.remove(ext.id());
        self.broadcast(cm, U::ExtensionRemoved(ext.id().clone()))
    }

    fn on_component_set(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
        _: EntityIndex,
        _: ComponentType,
        _: &Component,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "MetadataWatcher should never receive component_set events"
        );
        Ok(())
    }

    fn on_component_put(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
        _: EntityIndex,
        _: ComponentType,
        _: &Component,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "MetadataWatcher should never receive component_put events"
        );
        Ok(())
    }

    fn on_entity_registered(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
        _: EntityIndex,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "MetadataWatcher should never receive entity_registered events"
        );
        Ok(())
    }
}
