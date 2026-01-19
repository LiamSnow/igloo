//! Each different Watchers subscribes to a series of
//! events depending on the exact query.
//!
//! [TreeSubscribers] keeps track of these subscriptions and is used by
//! [dispatcher.rs] to dispatch events accordingly.

use crate::{
    query::watch::{WatcherID, WatcherList},
    tree::Extension,
};
use igloo_interface::{
    ComponentType,
    id::{DeviceID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::collections::HashMap;

/// Event Name -> Subscribers by parameters
#[derive(Default)]
pub struct TreeSubscribers {
    pub component_set: ComponentSetEventSubscribers,
    pub component_put: ComponentPutEventSubscribers,
    pub entity_registered: EntityEventSubscribers,
    pub device_created: DeviceEventSubscribers,
    pub device_renamed: DeviceEventSubscribers,
    pub device_deleted: DeviceEventSubscribers,
    pub group_created: GroupEventSubscribers,
    pub group_renamed: GroupEventSubscribers,
    pub group_deleted: GroupEventSubscribers,
    pub group_device_removed: GroupDeviceEventSubscribers,
    pub group_device_added: GroupDeviceEventSubscribers,
    pub ext_attached: ExtensionEventSubscribers,
    pub ext_detached: ExtensionEventSubscribers,
}

impl TreeSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.component_set.unsubscribe(watcher_id);
        self.component_put.unsubscribe(watcher_id);
        self.device_created.unsubscribe(watcher_id);
        self.device_renamed.unsubscribe(watcher_id);
        self.device_deleted.unsubscribe(watcher_id);
        self.entity_registered.unsubscribe(watcher_id);
        self.group_created.unsubscribe(watcher_id);
        self.group_renamed.unsubscribe(watcher_id);
        self.group_deleted.unsubscribe(watcher_id);
        self.group_device_removed.unsubscribe(watcher_id);
        self.group_device_added.unsubscribe(watcher_id);
        self.ext_attached.unsubscribe(watcher_id);
        self.ext_detached.unsubscribe(watcher_id);
    }
}

pub struct GroupEventSubscribers {
    /// sub to group
    pub by_gid: FxHashMap<GroupID, WatcherList>,
    /// sub to all groups
    pub all: WatcherList,
}

pub struct GroupDeviceEventSubscribers {
    /// sub to events from this group
    pub by_gid: FxHashMap<GroupID, GroupDeviceEventGroupSubscribers>,
    /// sub to any device added/removed from any group
    pub all: WatcherList,
}

pub struct GroupDeviceEventGroupSubscribers {
    /// sub to this device's added/removed from this group
    pub by_did: FxHashMap<DeviceID, WatcherList>,
    /// sub to any device added/removed from this group
    pub all: WatcherList,
}

pub struct ExtensionEventSubscribers {
    /// sub to ext
    pub by_xid: FxHashMap<ExtensionID, WatcherList>,
    /// sub to ext
    pub by_xindex: FxHashMap<ExtensionIndex, WatcherList>,
    /// sub to ext containing this device
    pub by_did: FxHashMap<DeviceID, WatcherList>,
    /// sub to all exts
    pub all: WatcherList,
}

pub struct DeviceEventSubscribers {
    /// sub to device
    pub by_did: FxHashMap<DeviceID, WatcherList>,
    /// sub to all devices
    pub all: WatcherList,
}

pub struct EntityEventSubscribers {
    /// sub to entity events on specific device
    pub by_did: FxHashMap<DeviceID, WatcherList>,
    /// sub to entity events on all devices
    pub all: WatcherList,
}

pub struct ComponentSetEventSubscribers(
    pub FxHashMap<(DeviceID, EntityIndex, ComponentType), WatcherList>,
);

pub struct ComponentPutEventSubscribers {
    /// sub to component events on specific device
    pub by_did: FxHashMap<DeviceID, ComponentPutEventDeviceSubscribers>,
    /// sub to this component event on any device/entity
    pub by_comp_type: FxHashMap<ComponentType, WatcherList>,
    /// sub to all component events on all devices/entities
    pub all: WatcherList,
}

#[derive(Clone)]
pub struct ComponentPutEventDeviceSubscribers {
    /// sub to component events to specific entity
    pub by_eindex: FxHashMap<EntityIndex, ComponentPutEventEntitySubscribers>,
    /// sub to this component event on any entity in this device
    pub by_comp_type: FxHashMap<ComponentType, WatcherList>,
    /// sub to all components events on this device
    pub all: WatcherList,
}

#[derive(Clone)]
pub struct ComponentPutEventEntitySubscribers {
    /// sub to this component event in this entity
    pub by_comp_type: FxHashMap<ComponentType, WatcherList>,
    /// sub to all component events on this entity
    pub all: WatcherList,
}

impl Default for GroupEventSubscribers {
    fn default() -> Self {
        Self {
            by_gid: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for GroupDeviceEventSubscribers {
    fn default() -> Self {
        Self {
            by_gid: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for GroupDeviceEventGroupSubscribers {
    fn default() -> Self {
        Self {
            by_did: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for ExtensionEventSubscribers {
    fn default() -> Self {
        Self {
            by_xid: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            by_xindex: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            by_did: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for DeviceEventSubscribers {
    fn default() -> Self {
        Self {
            by_did: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for EntityEventSubscribers {
    fn default() -> Self {
        Self {
            by_did: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for ComponentSetEventSubscribers {
    fn default() -> Self {
        Self(HashMap::with_capacity_and_hasher(2, FxBuildHasher))
    }
}

impl Default for ComponentPutEventSubscribers {
    fn default() -> Self {
        Self {
            by_did: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            by_comp_type: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for ComponentPutEventDeviceSubscribers {
    fn default() -> Self {
        Self {
            by_eindex: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            by_comp_type: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl Default for ComponentPutEventEntitySubscribers {
    fn default() -> Self {
        Self {
            by_comp_type: HashMap::with_capacity_and_hasher(2, FxBuildHasher),
            all: Vec::with_capacity(2),
        }
    }
}

impl GroupEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_gid.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    #[inline]
    pub fn affected(&self, group: &GroupID) -> Vec<WatcherID> {
        let capacity = self.all.len() + self.by_gid.get(group).map_or(0, |list| list.len());

        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(&self.all);

        if let Some(list) = self.by_gid.get(group) {
            result.extend_from_slice(list);
        }

        result
    }
}

impl GroupDeviceEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_gid.retain(|_, group_sub| {
            group_sub.unsubscribe(watcher_id);
            !group_sub.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    #[inline]
    pub fn affected(&self, group: &GroupID, device: &DeviceID) -> Vec<WatcherID> {
        let group_sub = self.by_gid.get(group);

        let capacity = self.all.len()
            + group_sub.map_or(0, |gs| {
                gs.all.len() + gs.by_did.get(device).map_or(0, |list| list.len())
            });

        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(&self.all);

        if let Some(group_sub) = group_sub {
            result.extend_from_slice(&group_sub.all);
            if let Some(list) = group_sub.by_did.get(device) {
                result.extend_from_slice(list);
            }
        }

        result
    }
}

impl GroupDeviceEventGroupSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_did.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    pub fn is_empty(&self) -> bool {
        self.by_did.is_empty() && self.all.is_empty()
    }
}

impl ExtensionEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_xid.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.by_xindex.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    #[inline]
    pub fn affected(&self, ext: &Extension) -> Vec<WatcherID> {
        let capacity = self.all.len()
            + self.by_xindex.get(ext.index()).map_or(0, |list| list.len())
            + self.by_xid.get(ext.id()).map_or(0, |list| list.len())
            // maybe not the best guess but realistically some devices
            // have multiple subs, some have none so idk
            + ext.devices().len();

        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(&self.all);

        if let Some(list) = self.by_xid.get(ext.id()) {
            result.extend_from_slice(list);
        }

        if let Some(list) = self.by_xindex.get(ext.index()) {
            result.extend_from_slice(list);
        }

        for did in ext.devices() {
            if let Some(list) = self.by_did.get(did) {
                result.extend_from_slice(list);
            }
        }

        result
    }
}

impl DeviceEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_did.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    #[inline]
    pub fn affected(&self, did: &DeviceID) -> Vec<WatcherID> {
        let capacity = self.all.len() + self.by_did.get(did).map_or(0, |list| list.len());

        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(&self.all);

        if let Some(list) = self.by_did.get(did) {
            result.extend_from_slice(list);
        }

        result
    }
}

impl EntityEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_did.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    #[inline]
    pub fn affected(&self, did: &DeviceID) -> Vec<WatcherID> {
        let capacity = self.all.len() + self.by_did.get(did).map_or(0, |list| list.len());

        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(&self.all);

        if let Some(list) = self.by_did.get(did) {
            result.extend_from_slice(list);
        }

        result
    }
}

impl ComponentSetEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.0.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
    }

    #[inline]
    pub fn affected(
        &self,
        did: DeviceID,
        eindex: EntityIndex,
        comp_type: ComponentType,
    ) -> Vec<WatcherID> {
        match self.0.get(&(did, eindex, comp_type)) {
            Some(list) => list.to_vec(),
            None => vec![],
        }
    }
}

impl ComponentPutEventSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_did.retain(|_, device_sub| {
            device_sub.unsubscribe(watcher_id);
            !device_sub.is_empty()
        });
        self.by_comp_type.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    #[inline]
    pub fn affected(
        &self,
        did: &DeviceID,
        eindex: &EntityIndex,
        comp_type: &ComponentType,
    ) -> Vec<WatcherID> {
        let device_sub = self.by_did.get(did);

        let capacity = self.all.len()
            + self
                .by_comp_type
                .get(comp_type)
                .map_or(0, |list| list.len())
            + device_sub.map_or(0, |dsubs| {
                dsubs.all.len()
                    + dsubs
                        .by_comp_type
                        .get(comp_type)
                        .map_or(0, |list| list.len())
                    + dsubs.by_eindex.get(eindex).map_or(0, |esubs| {
                        esubs.all.len()
                            + esubs
                                .by_comp_type
                                .get(comp_type)
                                .map_or(0, |list| list.len())
                    })
            });

        let mut result = Vec::with_capacity(capacity);

        result.extend_from_slice(&self.all);

        if let Some(list) = self.by_comp_type.get(comp_type) {
            result.extend_from_slice(list);
        }

        if let Some(dsubs) = device_sub {
            result.extend_from_slice(&dsubs.all);

            if let Some(list) = dsubs.by_comp_type.get(comp_type) {
                result.extend_from_slice(list);
            }

            if let Some(esubs) = dsubs.by_eindex.get(eindex) {
                result.extend_from_slice(&esubs.all);

                if let Some(list) = esubs.by_comp_type.get(comp_type) {
                    result.extend_from_slice(list);
                }
            }
        }

        result
    }
}

impl ComponentPutEventDeviceSubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_eindex.retain(|_, entity_sub| {
            entity_sub.unsubscribe(watcher_id);
            !entity_sub.is_empty()
        });
        self.by_comp_type.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    pub fn is_empty(&self) -> bool {
        self.by_eindex.is_empty() && self.by_comp_type.is_empty() && self.all.is_empty()
    }
}

impl ComponentPutEventEntitySubscribers {
    pub fn unsubscribe(&mut self, watcher_id: WatcherID) {
        self.by_comp_type.retain(|_, list| {
            list.retain(|&id| id != watcher_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != watcher_id);
    }

    pub fn is_empty(&self) -> bool {
        self.by_comp_type.is_empty() && self.all.is_empty()
    }
}
