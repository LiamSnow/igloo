//! Takes events from DeviceTree and dispatches to affected Watchers
//! Most of the logic lives in [subscriber.rs]

use crate::{
    core::{ClientManager, IglooError},
    query::{
        QueryEngine,
        ctx::QueryContext,
        watch::{Watcher, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{EntityIndex, GroupID},
};

impl QueryEngine {
    pub fn on_component_set(
        &mut self,
        cm: &mut ClientManager,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        comp_type: ComponentType,
        comp: Component,
    ) -> Result<(), IglooError> {
        let affected = self
            .tree_subs
            .component_set
            .affected(*device.id(), entity_index, comp_type);

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_component_set(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
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
            self.tree_subs
                .component_put
                .affected(device.id(), &entity_index, &comp_type);

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
                            tree,
                            device,
                            entity_index,
                            comp_type,
                            &comp,
                        )?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_component_put(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
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
        let affected = self.tree_subs.device_created.affected(device.id());

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_device_created(cm, &mut self.ctx, &mut self.tree_subs, tree, device)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_device_created(cm, &mut self.ctx, &mut self.tree_subs, tree, device)?;
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
        let affected = self.tree_subs.device_deleted.affected(device.id());

        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_device_deleted(cm, &mut self.ctx, &mut self.tree_subs, tree, device)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_device_deleted(cm, &mut self.ctx, &mut self.tree_subs, tree, device)?;
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
        let affected = self.tree_subs.device_renamed.affected(device.id());
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_device_renamed(cm, &mut self.ctx, &mut self.tree_subs, tree, device)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_device_renamed(cm, &mut self.ctx, &mut self.tree_subs, tree, device)?;
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
        let affected = self.tree_subs.entity_registered.affected(device.id());
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
                            tree,
                            device,
                            entity_index,
                        )?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_entity_registered(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
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
        let affected = self.tree_subs.group_created.affected(group.id());
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.tree_subs, tree, group)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_group_created(cm, &mut self.ctx, &mut self.tree_subs, tree, group)?;
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
        let affected = self.tree_subs.group_deleted.affected(gid);
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.tree_subs, tree, gid)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_group_deleted(cm, &mut self.ctx, &mut self.tree_subs, tree, gid)?;
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
        let affected = self.tree_subs.group_renamed.affected(group.id());
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.tree_subs, tree, group)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_group_renamed(cm, &mut self.ctx, &mut self.tree_subs, tree, group)?;
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
            .tree_subs
            .group_device_added
            .affected(group.id(), device.id());
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_group_device_added(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
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
            .tree_subs
            .group_device_removed
            .affected(group.id(), device.id());
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
                            tree,
                            group,
                            device,
                        )?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_group_device_removed(
                            cm,
                            &mut self.ctx,
                            &mut self.tree_subs,
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
        let affected = self.tree_subs.ext_attached.affected(ext);
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.tree_subs, tree, ext)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_ext_attached(cm, &mut self.ctx, &mut self.tree_subs, tree, ext)?;
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
        let affected = self.tree_subs.ext_detached.affected(ext);
        for watcher_id in affected {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                match watcher {
                    Watcher::Component(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.tree_subs, tree, ext)?;
                    }
                    Watcher::Metadata(w) => {
                        w.on_ext_detached(cm, &mut self.ctx, &mut self.tree_subs, tree, ext)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub trait TreeEventResponder {
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
