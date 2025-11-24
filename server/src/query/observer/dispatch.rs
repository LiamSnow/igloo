use crate::{
    core::{ClientManager, IglooError},
    query::{
        QueryEngine,
        ctx::QueryContext,
        observer::{
            Observer, comp::ComponentObserver, device::DeviceObserver, entity::EntityObserver,
            floe::FloeObserver, group::GroupObserver,
        },
    },
    tree::{Device, DeviceTree, Floe, Group},
};
use igloo_interface::{
    ComponentType,
    id::{FloeRef, GroupID},
    query::Query,
};

impl QueryEngine {
    pub fn register_observer(
        &mut self,
        tree: &DeviceTree,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
        query: Query,
    ) -> Result<(), IglooError> {
        let observer_id = if let Some(slot) = self.observers.iter().position(|w| w.is_none()) {
            slot
        } else {
            self.observers.push(None);
            self.observers.len() - 1
        };

        let observer = match query {
            Query::Device(q) => Observer::Devices(DeviceObserver::register(
                &mut self.ctx,
                &mut self.subscribers,
                tree,
                query_id,
                observer_id,
                client_id,
                q,
            )),

            Query::Entity(q) => Observer::Entities(EntityObserver::register(
                &mut self.ctx,
                &mut self.subscribers,
                tree,
                query_id,
                observer_id,
                client_id,
                q,
            )),

            Query::Component(q) => Observer::Components(ComponentObserver::register(
                &mut self.ctx,
                &mut self.subscribers,
                tree,
                query_id,
                observer_id,
                client_id,
                q,
            )),

            Query::Group(q) => Observer::Groups(GroupObserver::register(
                &mut self.ctx,
                &mut self.subscribers,
                tree,
                query_id,
                observer_id,
                client_id,
                q,
            )),

            Query::Floe(q) => Observer::Floes(FloeObserver::register(
                &mut self.ctx,
                &mut self.subscribers,
                tree,
                query_id,
                observer_id,
                client_id,
                q,
            )),
        };

        cm.add_observer(client_id, observer_id)?;

        self.observers[observer_id] = Some(observer);

        Ok(())
    }

    pub fn on_component_set(
        &mut self,
        tree: &DeviceTree,
        device: &Device,
        entity_index: usize,
        comp_type: ComponentType,
    ) -> Result<(), IglooError> {
        self.subscribers.component_set.for_each(
            *device.id(),
            entity_index,
            comp_type,
            |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_component_set(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Entities(w) => {
                            w.on_component_set(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Components(w) => {
                            w.on_component_set(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Groups(w) => {
                            w.on_component_set(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Floes(w) => {
                            w.on_component_set(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                    }
                }
                Ok(())
            },
        )
    }

    pub fn on_component_put(
        &mut self,
        tree: &DeviceTree,
        device: &Device,
        entity_index: usize,
        comp_type: ComponentType,
    ) -> Result<(), IglooError> {
        self.subscribers.component_put.for_each(
            *device.id(),
            entity_index,
            comp_type,
            |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_component_put(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Entities(w) => {
                            w.on_component_put(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Components(w) => {
                            w.on_component_put(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Groups(w) => {
                            w.on_component_put(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                        Observer::Floes(w) => {
                            w.on_component_put(
                                &mut self.ctx,
                                tree,
                                device,
                                entity_index,
                                comp_type,
                            )?;
                        }
                    }
                }
                Ok(())
            },
        )
    }

    pub fn on_device_created(
        &mut self,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.subscribers
            .device_created
            .for_each(*device.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_device_created(&mut self.ctx, tree, device)?;
                        }
                        Observer::Entities(w) => {
                            w.on_device_created(&mut self.ctx, tree, device)?;
                        }
                        Observer::Components(w) => {
                            w.on_device_created(&mut self.ctx, tree, device)?;
                        }
                        Observer::Groups(w) => {
                            w.on_device_created(&mut self.ctx, tree, device)?;
                        }
                        Observer::Floes(w) => {
                            w.on_device_created(&mut self.ctx, tree, device)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_device_deleted(
        &mut self,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.subscribers
            .device_deleted
            .for_each(*device.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_device_deleted(&mut self.ctx, tree, device)?;
                        }
                        Observer::Entities(w) => {
                            w.on_device_deleted(&mut self.ctx, tree, device)?;
                        }
                        Observer::Components(w) => {
                            w.on_device_deleted(&mut self.ctx, tree, device)?;
                        }
                        Observer::Groups(w) => {
                            w.on_device_deleted(&mut self.ctx, tree, device)?;
                        }
                        Observer::Floes(w) => {
                            w.on_device_deleted(&mut self.ctx, tree, device)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_device_renamed(
        &mut self,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.subscribers
            .device_renamed
            .for_each(*device.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_device_renamed(&mut self.ctx, tree, device)?;
                        }
                        Observer::Entities(w) => {
                            w.on_device_renamed(&mut self.ctx, tree, device)?;
                        }
                        Observer::Components(w) => {
                            w.on_device_renamed(&mut self.ctx, tree, device)?;
                        }
                        Observer::Groups(w) => {
                            w.on_device_renamed(&mut self.ctx, tree, device)?;
                        }
                        Observer::Floes(w) => {
                            w.on_device_renamed(&mut self.ctx, tree, device)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_entity_registered(
        &mut self,
        tree: &DeviceTree,
        device: &Device,
        entity_index: usize,
    ) -> Result<(), IglooError> {
        self.subscribers
            .entity_registered
            .for_each(*device.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_entity_registered(&mut self.ctx, tree, device, entity_index)?;
                        }
                        Observer::Entities(w) => {
                            w.on_entity_registered(&mut self.ctx, tree, device, entity_index)?;
                        }
                        Observer::Components(w) => {
                            w.on_entity_registered(&mut self.ctx, tree, device, entity_index)?;
                        }
                        Observer::Groups(w) => {
                            w.on_entity_registered(&mut self.ctx, tree, device, entity_index)?;
                        }
                        Observer::Floes(w) => {
                            w.on_entity_registered(&mut self.ctx, tree, device, entity_index)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_group_created(&mut self, tree: &DeviceTree, group: &Group) -> Result<(), IglooError> {
        self.subscribers
            .group_created
            .for_each(*group.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_group_created(&mut self.ctx, tree, group)?;
                        }
                        Observer::Entities(w) => {
                            w.on_group_created(&mut self.ctx, tree, group)?;
                        }
                        Observer::Components(w) => {
                            w.on_group_created(&mut self.ctx, tree, group)?;
                        }
                        Observer::Groups(w) => {
                            w.on_group_created(&mut self.ctx, tree, group)?;
                        }
                        Observer::Floes(w) => {
                            w.on_group_created(&mut self.ctx, tree, group)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_group_deleted(&mut self, tree: &DeviceTree, gid: &GroupID) -> Result<(), IglooError> {
        self.subscribers
            .group_deleted
            .for_each(*gid, |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_group_deleted(&mut self.ctx, tree, gid)?;
                        }
                        Observer::Entities(w) => {
                            w.on_group_deleted(&mut self.ctx, tree, gid)?;
                        }
                        Observer::Components(w) => {
                            w.on_group_deleted(&mut self.ctx, tree, gid)?;
                        }
                        Observer::Groups(w) => {
                            w.on_group_deleted(&mut self.ctx, tree, gid)?;
                        }
                        Observer::Floes(w) => {
                            w.on_group_deleted(&mut self.ctx, tree, gid)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_group_renamed(&mut self, tree: &DeviceTree, group: &Group) -> Result<(), IglooError> {
        self.subscribers
            .group_renamed
            .for_each(*group.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_group_renamed(&mut self.ctx, tree, group)?;
                        }
                        Observer::Entities(w) => {
                            w.on_group_renamed(&mut self.ctx, tree, group)?;
                        }
                        Observer::Components(w) => {
                            w.on_group_renamed(&mut self.ctx, tree, group)?;
                        }
                        Observer::Groups(w) => {
                            w.on_group_renamed(&mut self.ctx, tree, group)?;
                        }
                        Observer::Floes(w) => {
                            w.on_group_renamed(&mut self.ctx, tree, group)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_group_membership_changed(
        &mut self,
        tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.subscribers
            .group_membership_changed
            .for_each(*group.id(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_group_membership_changed(&mut self.ctx, tree, group, device)?;
                        }
                        Observer::Entities(w) => {
                            w.on_group_membership_changed(&mut self.ctx, tree, group, device)?;
                        }
                        Observer::Components(w) => {
                            w.on_group_membership_changed(&mut self.ctx, tree, group, device)?;
                        }
                        Observer::Groups(w) => {
                            w.on_group_membership_changed(&mut self.ctx, tree, group, device)?;
                        }
                        Observer::Floes(w) => {
                            w.on_group_membership_changed(&mut self.ctx, tree, group, device)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_floe_attached(&mut self, tree: &DeviceTree, floe: &Floe) -> Result<(), IglooError> {
        self.subscribers
            .floe_attached
            .for_each(*floe.fref(), |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_floe_attached(&mut self.ctx, tree, floe)?;
                        }
                        Observer::Entities(w) => {
                            w.on_floe_attached(&mut self.ctx, tree, floe)?;
                        }
                        Observer::Components(w) => {
                            w.on_floe_attached(&mut self.ctx, tree, floe)?;
                        }
                        Observer::Groups(w) => {
                            w.on_floe_attached(&mut self.ctx, tree, floe)?;
                        }
                        Observer::Floes(w) => {
                            w.on_floe_attached(&mut self.ctx, tree, floe)?;
                        }
                    }
                }
                Ok(())
            })
    }

    pub fn on_floe_detached(
        &mut self,
        tree: &DeviceTree,
        fref: &FloeRef,
    ) -> Result<(), IglooError> {
        self.subscribers
            .floe_detached
            .for_each(*fref, |observer_id| {
                if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                    match observer {
                        Observer::Devices(w) => {
                            w.on_floe_detached(&mut self.ctx, tree, fref)?;
                        }
                        Observer::Entities(w) => {
                            w.on_floe_detached(&mut self.ctx, tree, fref)?;
                        }
                        Observer::Components(w) => {
                            w.on_floe_detached(&mut self.ctx, tree, fref)?;
                        }
                        Observer::Groups(w) => {
                            w.on_floe_detached(&mut self.ctx, tree, fref)?;
                        }
                        Observer::Floes(w) => {
                            w.on_floe_detached(&mut self.ctx, tree, fref)?;
                        }
                    }
                }
                Ok(())
            })
    }
}

pub trait ObserverHandler {
    fn on_component_set(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
        entity_index: usize,
        comp_type: ComponentType,
    ) -> Result<(), IglooError>;

    fn on_component_put(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
        entity_index: usize,
        comp_type: ComponentType,
    ) -> Result<(), IglooError>;

    fn on_device_created(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_device_deleted(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_device_renamed(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_entity_registered(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
        entity_index: usize,
    ) -> Result<(), IglooError>;

    fn on_group_created(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError>;

    fn on_group_deleted(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        gid: &GroupID,
    ) -> Result<(), IglooError>;

    fn on_group_renamed(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError>;

    fn on_group_membership_changed(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_floe_attached(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        floe: &Floe,
    ) -> Result<(), IglooError>;

    fn on_floe_detached(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        fref: &FloeRef,
    ) -> Result<(), IglooError>;
}
