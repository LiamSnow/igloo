//! Takes events from DeviceTree and dispatches to affected Watchers
//! Most of the logic lives in [subscriber.rs]

use crate::{
    core::{ClientManager, IglooError},
    query::{
        QueryEngine,
        ctx::QueryContext,
        watch::{
            Watcher, comp::ComponentWatcher, device::DeviceWatcher, entity::EntityWatcher,
            ext::ExtensionWatcher, group::GroupWatcher, subscriber::TreeSubscribers,
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
    pub fn register_watcher(
        &mut self,
        tree: &DeviceTree,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
        query: Query,
    ) -> Result<Result<(), QueryError>, IglooError> {
        let watcher_id = if let Some(slot) = self.watchers.iter().position(|w| w.is_none()) {
            slot
        } else {
            self.watchers.push(None);
            self.watchers.len() - 1
        };

        let watcher = match query {
            Query::Device(q) => {
                match DeviceWatcher::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    watcher_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Watcher::Devices(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Entity(q) => {
                match EntityWatcher::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    watcher_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Watcher::Entities(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Component(q) => {
                match ComponentWatcher::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    watcher_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Watcher::Components(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Group(q) => {
                match GroupWatcher::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    watcher_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Watcher::Groups(o),
                    Err(e) => return Ok(Err(e)),
                }
            }

            Query::Extension(q) => {
                match ExtensionWatcher::register(
                    &mut self.ctx,
                    &mut self.subscribers,
                    tree,
                    query_id,
                    watcher_id,
                    client_id,
                    q,
                ) {
                    Ok(o) => Watcher::Extensions(o),
                    Err(e) => return Ok(Err(e)),
                }
            }
        };

        cm.add_watcher(client_id, query_id, watcher_id)?;

        self.watchers[watcher_id] = Some(watcher);

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

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
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
                    Watcher::Entities(w) => {
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
                    Watcher::Components(w) => {
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
                    Watcher::Groups(w) => {
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
                    Watcher::Extensions(w) => {
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

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
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
                    Watcher::Entities(w) => {
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
                    Watcher::Components(w) => {
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
                    Watcher::Groups(w) => {
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
                    Watcher::Extensions(w) => {
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

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Entities(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Components(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Groups(w) => {
                        w.on_device_created(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Extensions(w) => {
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

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Entities(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Components(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Groups(w) => {
                        w.on_device_deleted(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Entities(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Components(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Groups(w) => {
                        w.on_device_renamed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                        )?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Watcher::Entities(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Watcher::Components(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Watcher::Groups(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Entities(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Components(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Groups(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Watcher::Entities(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Watcher::Components(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Watcher::Groups(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.subscribers, tree, gid)?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Entities(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Components(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Groups(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.subscribers, tree, group)?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Entities(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Components(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Groups(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Entities(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Components(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Groups(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.subscribers,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Entities(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Components(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Groups(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Extensions(w) => {
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
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Devices(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Entities(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Components(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Groups(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                    Watcher::Extensions(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.subscribers, tree, ext)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub trait WatchHandler {
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
