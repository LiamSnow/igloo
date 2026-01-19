//! This one is simple. Two types of watchers: Name & Membership
//! Only have an ID filter, which means we have no need for a
//! crazy expansion/contraction system.

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryContext,
        watch::{dispatch::WatchHandler, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{EntityIndex, GroupID},
    query::{GroupAction, GroupQuery, IDFilter, WatchUpdate as U, check::QueryError},
};

pub struct GroupWatcher {
    pub client_id: usize,
    pub query_id: usize,
    pub query: GroupQuery,
    watcher_id: usize,
}

impl GroupWatcher {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    pub fn register(
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        query_id: usize,
        watcher_id: usize,
        client_id: usize,
        query: GroupQuery,
    ) -> Result<Self, QueryError> {
        if query.limit.is_some() {
            return Err(QueryError::LimitOnWatcher);
        }

        match &query.id {
            IDFilter::Any => {
                subscribe_to_all_groups(subs, watcher_id, &query.action);
            }
            IDFilter::Is(gid) => {
                subscribe_to_group(subs, *gid, watcher_id, &query.action);
            }
            IDFilter::OneOf(gids) => {
                for gid in gids {
                    subscribe_to_group(subs, *gid, watcher_id, &query.action);
                }
            }
        }

        Ok(Self {
            client_id,
            query_id,
            watcher_id,
            query,
        })
    }

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        match &self.query.id {
            IDFilter::Any => {
                unsubscribe_from_all_groups(subs, self.watcher_id, &self.query.action);
            }
            IDFilter::Is(gid) => {
                unsubscribe_from_group(subs, *gid, self.watcher_id, &self.query.action);
            }
            IDFilter::OneOf(gids) => {
                for gid in gids {
                    unsubscribe_from_group(subs, *gid, self.watcher_id, &self.query.action);
                }
            }
        }
    }
}

fn subscribe_to_all_groups(subs: &mut TreeSubscribers, watcher_id: usize, action: &GroupAction) {
    match action {
        GroupAction::WatchName => {
            subs.group_renamed.all.push(watcher_id);
        }
        GroupAction::WatchMembership => {
            subs.group_device_added.all.push(watcher_id);
            subs.group_device_removed.all.push(watcher_id);
        }
        _ => {}
    }
}

fn unsubscribe_from_all_groups(
    subs: &mut TreeSubscribers,
    watcher_id: usize,
    action: &GroupAction,
) {
    match action {
        GroupAction::WatchName => {
            subs.group_renamed.all.retain(|o| *o != watcher_id);
        }
        GroupAction::WatchMembership => {
            subs.group_device_added.all.retain(|o| *o != watcher_id);
            subs.group_device_removed.all.retain(|o| *o != watcher_id);
        }
        _ => {}
    }
}

fn subscribe_to_group(
    subs: &mut TreeSubscribers,
    gid: GroupID,
    watcher_id: usize,
    action: &GroupAction,
) {
    match action {
        GroupAction::WatchName => {
            subs.group_renamed
                .by_gid
                .entry(gid)
                .or_default()
                .push(watcher_id);
        }
        GroupAction::WatchMembership => {
            subs.group_device_added
                .by_gid
                .entry(gid)
                .or_default()
                .all
                .push(watcher_id);
            subs.group_device_removed
                .by_gid
                .entry(gid)
                .or_default()
                .all
                .push(watcher_id);
        }
        _ => {}
    }
}

fn unsubscribe_from_group(
    subs: &mut TreeSubscribers,
    gid: GroupID,
    watcher_id: usize,
    action: &GroupAction,
) {
    match action {
        GroupAction::WatchName => {
            if let Some(subs) = subs.group_renamed.by_gid.get_mut(&gid) {
                subs.retain(|o| *o != watcher_id);
            }
        }
        GroupAction::WatchMembership => {
            if let Some(subs) = subs.group_device_added.by_gid.get_mut(&gid) {
                subs.all.retain(|o| *o != watcher_id);
            }
            if let Some(subs) = subs.group_device_removed.by_gid.get_mut(&gid) {
                subs.all.retain(|o| *o != watcher_id);
            }
        }
        _ => {}
    }
}

impl WatchHandler for GroupWatcher {
    fn on_group_renamed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, GroupAction::WatchName));

        cm.send(
            self.client_id,
            IglooResponse::WatchUpdate {
                query_id: self.query_id,
                result: U::GroupRenamed(*group.id(), group.name().to_string()),
            },
        )
    }

    fn on_group_device_added(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, GroupAction::WatchMembership));

        cm.send(
            self.client_id,
            IglooResponse::WatchUpdate {
                query_id: self.query_id,
                result: U::GroupMembershipChanged(*group.id(), *device.id(), true),
            },
        )
    }

    fn on_group_device_removed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, GroupAction::WatchMembership));

        cm.send(
            self.client_id,
            IglooResponse::WatchUpdate {
                query_id: self.query_id,
                result: U::GroupMembershipChanged(*group.id(), *device.id(), false),
            },
        )
    }

    fn on_group_created(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Group,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive group_created events"
        );
        Ok(())
    }

    fn on_group_deleted(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &GroupID,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive group_deleted events"
        );
        Ok(())
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
            "GroupWatcher should never receive component_set events"
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
            "GroupWatcher should never receive component_put events"
        );
        Ok(())
    }

    fn on_device_created(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive device_created events"
        );
        Ok(())
    }

    fn on_device_deleted(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive device_deleted events"
        );
        Ok(())
    }

    fn on_device_renamed(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive device_renamed events"
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
            "GroupWatcher should never receive entity_registered events"
        );
        Ok(())
    }

    fn on_ext_attached(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Extension,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive ext_attached events"
        );
        Ok(())
    }

    fn on_ext_detached(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Extension,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "GroupWatcher should never receive ext_detached events"
        );
        Ok(())
    }
}
