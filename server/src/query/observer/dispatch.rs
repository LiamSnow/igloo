//! Takes events from DeviceTree and dispatches to affected Observers
//! Most of the logic lives in [subscriber.rs]

use crate::{
    core::{ClientManager, IglooError},
    query::{
        QueryEngine,
        ctx::QueryContext,
        observer::{
            Observer, comp::ComponentObserver, device::DeviceObserver, entity::EntityObserver,
            ext::ExtensionObserver, group::GroupObserver, subscriber::TreeSubscribers,
        },
    },
    tree::{Device, DeviceTree, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{EntityIndex, GroupID},
    query::{Query, check::QueryError},
};

impl QueryEngine {
    pub fn register_observer(
        &mut self,
        tree: &DeviceTree,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
        query: Query,
    ) -> Result<Result<(), QueryError>, IglooError> {
        let observer_id = if let Some(slot) = self.observers.iter().position(|w| w.is_none()) {
            slot
        } else {
            self.observers.push(None);
            self.observers.len() - 1
        };

        let observer = match query {
            Query::Device(q) => {
                match DeviceObserver::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    observer_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Observer::Devices(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Entity(q) => {
                match EntityObserver::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    observer_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Observer::Entities(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Component(q) => {
                match ComponentObserver::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    observer_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Observer::Components(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Group(q) => {
                match GroupObserver::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    observer_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Observer::Groups(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Extension(q) => {
                match ExtensionObserver::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    observer_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Observer::Extensions(o),
                    Err(e) => return Ok(Err(e)),
                }
            }
        };

        cm.add_observer(client_id, query_id, observer_id)?;

        self.observers[observer_id] = Some(observer);

        Ok(Ok(()))
    }

    pub fn on_component_set(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        comp_type: ComponentType,
        comp: Component,
    ) -> Result<(), IglooError> {
        let affected =
            self.subscribers
                .component_set
                .affected(*device.id(), entity_index, comp_type);

        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn on_component_put(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        comp_type: ComponentType,
        comp: Component,
    ) -> Result<(), IglooError> {
        let affected =
            self.subscribers
                .component_put
                .affected(device.id(), &entity_index, &comp_type);

        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_device_created(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.device_created.affected(device.id());

        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_device_deleted(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.device_deleted.affected(device.id());

        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn on_device_renamed(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.device_renamed.affected(device.id());
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_entity_registered(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.entity_registered.affected(device.id());
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_group_created(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.group_created.affected(group.id());
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Entities(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Components(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Groups(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Extensions(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_group_deleted(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        gid: &GroupID,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.group_deleted.affected(gid);
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Observer::Entities(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Observer::Components(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Observer::Groups(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Observer::Extensions(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_group_renamed(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.group_renamed.affected(group.id());
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Entities(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Components(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Groups(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Observer::Extensions(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_group_device_added(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        let affected = self
            .subscribers
            .group_device_added
            .affected(group.id(), device.id());
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_group_device_removed(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        let affected = self
            .subscribers
            .group_device_removed
            .affected(group.id(), device.id());
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Entities(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Components(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Groups(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Observer::Extensions(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_ext_attached(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.ext_attached.affected(ext);
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Entities(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Components(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Groups(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Extensions(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn on_ext_detached(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        let affected = self.subscribers.ext_detached.affected(ext);
        for observer_id in affected {
            if let Some(Some(observer)) = self.observers.get_mut(observer_id) {
                match observer {
                    Observer::Devices(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Entities(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Components(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Groups(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Observer::Extensions(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub trait ObserverHandler {
    fn on_component_set(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        comp_type: ComponentType,
        comp: &Component,
    ) -> Result<(), IglooError>;

    fn on_component_put(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        comp_type: ComponentType,
        comp: &Component,
    ) -> Result<(), IglooError>;

    fn on_device_created(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_device_deleted(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_device_renamed(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_entity_registered(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
    ) -> Result<(), IglooError>;

    fn on_group_created(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError>;

    fn on_group_deleted(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        gid: &GroupID,
    ) -> Result<(), IglooError>;

    fn on_group_renamed(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError>;

    fn on_group_device_added(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_group_device_removed(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError>;

    fn on_ext_attached(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError>;

    fn on_ext_detached(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError>;
}
