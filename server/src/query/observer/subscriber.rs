use crate::{
    core::IglooError,
    query::observer::{ObserverID, ObserverList},
};
use igloo_interface::{
    ComponentType,
    id::{DeviceID, FloeRef, GroupID},
};
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct TreeSubscribers {
    pub component_set: ComponentEventSubscribers,
    pub component_put: ComponentEventSubscribers,
    pub entity_registered: EntityEventSubscribers,
    pub device_created: DeviceEventSubscribers,
    pub device_renamed: DeviceEventSubscribers,
    pub device_deleted: DeviceEventSubscribers,
    pub group_created: GroupEventSubscribers,
    pub group_renamed: GroupEventSubscribers,
    pub group_deleted: GroupEventSubscribers,
    pub group_membership_changed: GroupEventSubscribers,
    pub floe_attached: FloeEventSubscribers,
    pub floe_detached: FloeEventSubscribers,
}

impl TreeSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.component_set.unsubscribe(observer_id);
        self.component_put.unsubscribe(observer_id);
        self.device_created.unsubscribe(observer_id);
        self.device_renamed.unsubscribe(observer_id);
        self.device_deleted.unsubscribe(observer_id);
        self.entity_registered.unsubscribe(observer_id);
        self.group_created.unsubscribe(observer_id);
        self.group_renamed.unsubscribe(observer_id);
        self.group_deleted.unsubscribe(observer_id);
        self.group_membership_changed.unsubscribe(observer_id);
        self.floe_attached.unsubscribe(observer_id);
        self.floe_detached.unsubscribe(observer_id);
    }
}

#[derive(Default)]
pub struct GroupEventSubscribers {
    /// sub to group
    pub by_id: FxHashMap<GroupID, ObserverList>,
    /// sub to all groups
    pub all: ObserverList,
}

#[derive(Default)]
pub struct FloeEventSubscribers {
    /// sub to floe
    pub by_ref: FxHashMap<FloeRef, ObserverList>,
    /// sub to all floes
    pub all: ObserverList,
}

#[derive(Default)]
pub struct DeviceEventSubscribers {
    /// sub to device
    pub by_id: FxHashMap<DeviceID, ObserverList>,
    /// sub to all devices
    pub all: ObserverList,
}

#[derive(Default)]
pub struct EntityEventSubscribers {
    /// sub to entity events on specific device
    pub device: FxHashMap<DeviceID, ObserverList>,
    /// sub to entity events on all devices
    pub all: ObserverList,
}

#[derive(Default)]
pub struct ComponentEventSubscribers {
    /// sub to component events on specific device
    pub device: FxHashMap<DeviceID, ComponentEventDeviceSubscribers>,
    /// sub to this component event on any device/entity
    pub comp: FxHashMap<ComponentType, ObserverList>,
    /// sub to all component events on all devices/entities
    pub all: ObserverList,
}

#[derive(Clone, Default)]
pub struct ComponentEventDeviceSubscribers {
    /// sub to component events to specific entity
    pub entity: FxHashMap<usize, ComponentEventEntitySubscribers>,
    /// sub to this component event on any entity in this device
    pub comp: FxHashMap<ComponentType, ObserverList>,
    /// sub to all components events on this device
    pub all: ObserverList,
}

#[derive(Clone, Default)]
pub struct ComponentEventEntitySubscribers {
    /// sub to this component event in this entity
    pub by_type: FxHashMap<ComponentType, ObserverList>,
    /// sub to all component events on this entity
    pub all: ObserverList,
}

impl GroupEventSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.by_id.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    #[inline]
    pub fn for_each(
        &self,
        group: GroupID,
        mut f: impl FnMut(ObserverID) -> Result<(), IglooError>,
    ) -> Result<(), IglooError> {
        for &id in &self.all {
            f(id)?;
        }
        if let Some(list) = self.by_id.get(&group) {
            for &id in list {
                f(id)?;
            }
        }
        Ok(())
    }
}

impl FloeEventSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.by_ref.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    #[inline]
    pub fn for_each(
        &self,
        floe: FloeRef,
        mut f: impl FnMut(ObserverID) -> Result<(), IglooError>,
    ) -> Result<(), IglooError> {
        for &id in &self.all {
            f(id)?;
        }
        if let Some(list) = self.by_ref.get(&floe) {
            for &id in list {
                f(id)?;
            }
        }
        Ok(())
    }
}

impl DeviceEventSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.by_id.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    #[inline]
    pub fn for_each(
        &self,
        did: DeviceID,
        mut f: impl FnMut(ObserverID) -> Result<(), IglooError>,
    ) -> Result<(), IglooError> {
        for &id in &self.all {
            f(id)?;
        }
        if let Some(list) = self.by_id.get(&did) {
            for &id in list {
                f(id)?;
            }
        }
        Ok(())
    }
}

impl EntityEventSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.device.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    #[inline]
    pub fn for_each(
        &self,
        did: DeviceID,
        mut f: impl FnMut(ObserverID) -> Result<(), IglooError>,
    ) -> Result<(), IglooError> {
        for &id in &self.all {
            f(id)?;
        }
        if let Some(list) = self.device.get(&did) {
            for &id in list {
                f(id)?;
            }
        }
        Ok(())
    }
}

impl ComponentEventSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.device.retain(|_, device_sub| {
            device_sub.unsubscribe(observer_id);
            !device_sub.is_empty()
        });
        self.comp.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    #[inline]
    pub fn for_each(
        &self,
        did: DeviceID,
        eindex: usize,
        comp_type: ComponentType,
        mut f: impl FnMut(ObserverID) -> Result<(), IglooError>,
    ) -> Result<(), IglooError> {
        for &id in &self.all {
            f(id)?;
        }
        if let Some(list) = self.comp.get(&comp_type) {
            for &id in list {
                f(id)?;
            }
        }

        if let Some(dsubs) = self.device.get(&did) {
            for &id in &dsubs.all {
                f(id)?;
            }
            if let Some(list) = dsubs.comp.get(&comp_type) {
                for &id in list {
                    f(id)?;
                }
            }

            if let Some(esubs) = dsubs.entity.get(&eindex) {
                for &id in &esubs.all {
                    f(id)?;
                }
                if let Some(list) = esubs.by_type.get(&comp_type) {
                    for &id in list {
                        f(id)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl ComponentEventDeviceSubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.entity.retain(|_, entity_sub| {
            entity_sub.unsubscribe(observer_id);
            !entity_sub.is_empty()
        });
        self.comp.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    pub fn is_empty(&self) -> bool {
        self.entity.is_empty() && self.comp.is_empty() && self.all.is_empty()
    }
}

impl ComponentEventEntitySubscribers {
    pub fn unsubscribe(&mut self, observer_id: ObserverID) {
        self.by_type.retain(|_, list| {
            list.retain(|&id| id != observer_id);
            !list.is_empty()
        });
        self.all.retain(|&id| id != observer_id);
    }

    pub fn is_empty(&self) -> bool {
        self.by_type.is_empty() && self.all.is_empty()
    }
}
