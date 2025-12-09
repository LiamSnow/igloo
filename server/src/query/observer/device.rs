//! Observers either if a Device is attached or if it's renamed.
//! While this does have more complex filters than simply ID
//! and can have IDFilter::Any, it still makes sense NOT to have an
//! expansion/contraction system.
//! The contraction system means less checks for ext_attached, device_renamed
//! BUT means many more checks: device_created, group_device_added, group_device_removed, group_deleted, device_deleted
//! It should have better performance while being massively simpler

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryContext,
        iter::{
            passes_device_last_update, passes_entity_count, passes_group_filter, passes_id_filter,
            passes_owner_filter,
        },
        observer::{dispatch::ObserverHandler, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{DeviceID, EntityIndex, GroupID},
    query::{
        DeviceAction, DeviceFilter, DeviceQuery, IDFilter, ObserverUpdate as U, check::QueryError,
    },
};
use std::time::Instant;

pub struct DeviceObserver {
    pub client_id: usize,
    pub query_id: usize,
    pub query: DeviceQuery,
    observer_id: usize,
}

impl DeviceObserver {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    pub fn register(
        _: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _: &DeviceTree,
        query_id: usize,
        observer_id: usize,
        client_id: usize,
        query: DeviceQuery,
    ) -> Result<Self, QueryError> {
        if query.limit.is_some() {
            return Err(QueryError::LimitOnObserver);
        }

        match &query.filter.id {
            IDFilter::Any => {
                subscribe_to_all_devices(subs, observer_id, &query.action);
            }
            IDFilter::Is(gid) => {
                subscribe_to_device(subs, *gid, observer_id, &query.action);
            }
            IDFilter::OneOf(gids) => {
                for gid in gids {
                    subscribe_to_device(subs, *gid, observer_id, &query.action);
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
        match &self.query.filter.id {
            IDFilter::Any => {
                unsubscribe_from_all_devices(subs, self.observer_id, &self.query.action);
            }
            IDFilter::Is(gid) => {
                unsubscribe_from_device(subs, *gid, self.observer_id, &self.query.action);
            }
            IDFilter::OneOf(gids) => {
                for gid in gids {
                    unsubscribe_from_device(subs, *gid, self.observer_id, &self.query.action);
                }
            }
        }
    }
}

fn subscribe_to_all_devices(subs: &mut TreeSubscribers, observer_id: usize, action: &DeviceAction) {
    match action {
        DeviceAction::ObserveAttached => {
            subs.ext_attached.all.push(observer_id);
            subs.ext_detached.all.push(observer_id);
            subs.device_deleted.all.push(observer_id);
        }
        DeviceAction::ObserveName => {
            subs.device_renamed.all.push(observer_id);
        }
        _ => {}
    }
}

fn unsubscribe_from_all_devices(
    subs: &mut TreeSubscribers,
    observer_id: usize,
    action: &DeviceAction,
) {
    match action {
        DeviceAction::ObserveAttached => {
            subs.ext_attached.all.retain(|o| *o != observer_id);
            subs.ext_detached.all.retain(|o| *o != observer_id);
            subs.device_deleted.all.retain(|o| *o != observer_id);
        }
        DeviceAction::ObserveName => {
            subs.device_renamed.all.retain(|o| *o != observer_id);
        }
        _ => {}
    }
}

fn subscribe_to_device(
    subs: &mut TreeSubscribers,
    did: DeviceID,
    observer_id: usize,
    action: &DeviceAction,
) {
    match action {
        DeviceAction::ObserveAttached => {
            subs.ext_attached
                .by_did
                .entry(did)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(observer_id);
            subs.ext_detached
                .by_did
                .entry(did)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(observer_id);
            subs.device_deleted
                .by_did
                .entry(did)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(observer_id);
        }
        DeviceAction::ObserveName => {
            subs.device_renamed
                .by_did
                .entry(did)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(observer_id);
        }
        _ => {}
    }
}

fn unsubscribe_from_device(
    subs: &mut TreeSubscribers,
    did: DeviceID,
    observer_id: usize,
    action: &DeviceAction,
) {
    match action {
        DeviceAction::ObserveAttached => {
            if let Some(subs) = subs.ext_attached.by_did.get_mut(&did) {
                subs.retain(|o| *o != observer_id);
            }
            if let Some(subs) = subs.ext_detached.by_did.get_mut(&did) {
                subs.retain(|o| *o != observer_id);
            }
            if let Some(subs) = subs.device_deleted.by_did.get_mut(&did) {
                subs.retain(|o| *o != observer_id);
            }
        }
        DeviceAction::ObserveName => {
            if let Some(subs) = subs.device_renamed.by_did.get_mut(&did) {
                subs.retain(|o| *o != observer_id);
            }
        }
        _ => {}
    }
}

fn passes_all_filters(tree: &DeviceTree, device: &Device, filter: &DeviceFilter) -> bool {
    passes_id_filter(device, &filter.id)
        && passes_entity_count(device, &filter.entity_count)
        && passes_device_last_update(&Instant::now(), device, &filter.last_update)
        && passes_group_filter(device, &filter.group, tree)
        && passes_owner_filter(device, &filter.owner)
}

impl ObserverHandler for DeviceObserver {
    fn on_device_deleted(
        &mut self,
        cm: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, DeviceAction::ObserveAttached));

        if passes_all_filters(tree, device, &self.query.filter) {
            cm.send(
                self.client_id,
                IglooResponse::ObserverUpdate {
                    query_id: self.query_id,
                    result: U::DeviceAttached(*device.id(), false),
                },
            )?;
        }

        Ok(())
    }

    fn on_device_renamed(
        &mut self,
        cm: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, DeviceAction::ObserveName));

        if passes_all_filters(tree, device, &self.query.filter) {
            cm.send(
                self.client_id,
                IglooResponse::ObserverUpdate {
                    query_id: self.query_id,
                    result: U::DeviceRenamed(*device.id(), device.name().to_string()),
                },
            )?;
        }

        Ok(())
    }

    fn on_ext_attached(
        &mut self,
        cm: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, DeviceAction::ObserveAttached));

        for did in ext.devices() {
            if let Ok(device) = tree.device(did)
                && passes_all_filters(tree, device, &self.query.filter)
            {
                cm.send(
                    self.client_id,
                    IglooResponse::ObserverUpdate {
                        query_id: self.query_id,
                        result: U::DeviceAttached(*device.id(), true),
                    },
                )?;
            }
        }

        Ok(())
    }

    fn on_ext_detached(
        &mut self,
        cm: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        debug_assert!(matches!(self.query.action, DeviceAction::ObserveAttached));

        for did in ext.devices() {
            if let Ok(device) = tree.device(did)
                && passes_all_filters(tree, device, &self.query.filter)
            {
                cm.send(
                    self.client_id,
                    IglooResponse::ObserverUpdate {
                        query_id: self.query_id,
                        result: U::DeviceAttached(*device.id(), false),
                    },
                )?;
            }
        }

        Ok(())
    }

    fn on_group_device_added(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Group,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "DeviceObserver should never receive group_device_added events"
        );
        Ok(())
    }

    fn on_group_device_removed(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Group,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "DeviceObserver should never receive group_device_removed events"
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
            "DeviceObserver should never receive group_deleted events"
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
            "DeviceObserver should never receive device_created events"
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
            "DeviceObserver should never receive entity_registered events"
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
            "DeviceObserver should never receive component_set events"
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
            "DeviceObserver should never receive component_put events"
        );
        Ok(())
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
            "DeviceObserver should never receive group_created events"
        );
        Ok(())
    }

    fn on_group_renamed(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Group,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "DeviceObserver should never receive group_renamed events"
        );
        Ok(())
    }
}
