//! This one is simple. Two types of observers: Name & Membership
//! Only have an ID filter, which means we have no need for a
//! crazy expansion/contraction system.

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryContext,
        observer::{dispatch::ObserverHandler, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{EntityIndex, GroupID},
    query::{GroupAction, GroupQuery, IDFilter, ObserverUpdate as U, check::QueryError},
};

pub struct GroupObserver {
    pub client_id: usize,
    pub query_id: usize,
    pub query: GroupQuery,
    observer_id: usize,
}

impl GroupObserver {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    pub fn register(
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        query_id: usize,
        observer_id: usize,
        client_id: usize,
        query: GroupQuery,
    ) -> Result<Self, QueryError> {
        if query.limit.is_some() {
            return Err(QueryError::LimitOnObserver);
        }

        match &query.id {
            IDFilter::Any => {
                subscribe_to_all_groups(subs, observer_id, &query.action);
            }
            IDFilter::Is(gid) => {
                subscribe_to_group(subs, *gid, observer_id, &query.action);
            }
            IDFilter::OneOf(gids) => {
                for gid in gids {
                    subscribe_to_group(subs, *gid, observer_id, &query.action);
                }
            }
        }

        Ok(Self {
            client_id,
            query_id,
            observer_id,
            query,
        })
    }

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        match &self.query.id {
            IDFilter::Any => {
                unsubscribe_from_all_groups(subs, self.observer_id, &self.query.action);
            }
            IDFilter::Is(gid) => {
                unsubscribe_from_group(subs, *gid, self.observer_id, &self.query.action);
            }
            IDFilter::OneOf(gids) => {
                for gid in gids {
                    unsubscribe_from_group(subs, *gid, self.observer_id, &self.query.action);
                }
            }
        }
    }
}

fn subscribe_to_all_groups(subs: &mut TreeSubscribers, observer_id: usize, action: &GroupAction) {
    match action {
        GroupAction::ObserveName => {
            subs.group_renamed.all.push(observer_id);
        }
        GroupAction::ObserveMembership => {
            subs.group_device_added.all.push(observer_id);
            subs.group_device_removed.all.push(observer_id);
        }
        _ => {}
    }
}

fn unsubscribe_from_all_groups(
    subs: &mut TreeSubscribers,
    observer_id: usize,
    action: &GroupAction,
) {
    match action {
        GroupAction::ObserveName => {
            subs.group_renamed.all.retain(|o| *o != observer_id);
        }
        GroupAction::ObserveMembership => {
            subs.group_device_added.all.retain(|o| *o != observer_id);
            subs.group_device_removed.all.retain(|o| *o != observer_id);
        }
        _ => {}
    }
}

fn subscribe_to_group(
    subs: &mut TreeSubscribers,
    gid: GroupID,
    observer_id: usize,
    action: &GroupAction,
) {
    match action {
        GroupAction::ObserveName => {
            subs.group_renamed
                .by_gid
                .entry(gid)
                .or_default()
                .push(observer_id);
        }
        GroupAction::ObserveMembership => {
            subs.group_device_added
                .by_gid
                .entry(gid)
                .or_default()
                .all
                .push(observer_id);
            subs.group_device_removed
                .by_gid
                .entry(gid)
                .or_default()
                .all
                .push(observer_id);
        }
        _ => {}
    }
}

fn unsubscribe_from_group(
    subs: &mut TreeSubscribers,
    gid: GroupID,
    observer_id: usize,
    action: &GroupAction,
) {
    match action {
        GroupAction::ObserveName => {
            if let Some(subs) = subs.group_renamed.by_gid.get_mut(&gid) {
                subs.retain(|o| *o != observer_id);
            }
        }
        GroupAction::ObserveMembership => {
            if let Some(subs) = subs.group_device_added.by_gid.get_mut(&gid) {
                subs.all.retain(|o| *o != observer_id);
            }
            if let Some(subs) = subs.group_device_removed.by_gid.get_mut(&gid) {
                subs.all.retain(|o| *o != observer_id);
            }
        }
        _ => {}
    }
}

impl ObserverHandler for GroupObserver {
    fn on_group_renamed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        group: &Group,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, GroupAction::ObserveName));

        cm.send(
            self.client_id,
            IglooResponse::ObserverUpdate {
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
        debug_assert!(matches!(self.query.action, GroupAction::ObserveMembership));

        cm.send(
            self.client_id,
            IglooResponse::ObserverUpdate {
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
        debug_assert!(matches!(self.query.action, GroupAction::ObserveMembership));

        cm.send(
            self.client_id,
            IglooResponse::ObserverUpdate {
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
            "GroupObserver should never receive group_created events"
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
            "GroupObserver should never receive group_deleted events"
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
            "GroupObserver should never receive component_set events"
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
            "GroupObserver should never receive component_put events"
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
            "GroupObserver should never receive device_created events"
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
            "GroupObserver should never receive device_deleted events"
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
            "GroupObserver should never receive device_renamed events"
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
            "GroupObserver should never receive entity_registered events"
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
            "GroupObserver should never receive ext_attached events"
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
            "GroupObserver should never receive ext_detached events"
        );
        Ok(())
    }
}
