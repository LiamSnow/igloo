//! This one is super simple. We are only listening for Extension
//! Attach and Detach events. Since the ID is persistent, we can
//! simply subscribe to those events and have no need for a
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
    query::{ExtensionQuery, IDFilter, ObserverUpdate as U, check::QueryError},
};

pub struct ExtensionObserver {
    pub client_id: usize,
    pub query_id: usize,
    pub query: ExtensionQuery,
    observer_id: usize,
}

impl ExtensionObserver {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    pub fn register(
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        query_id: usize,
        observer_id: usize,
        client_id: usize,
        query: ExtensionQuery,
    ) -> Result<Self, QueryError> {
        if query.limit.is_some() {
            return Err(QueryError::LimitOnObserver);
        }

        match &query.id {
            IDFilter::Any => {
                subs.ext_attached.all.push(observer_id);
                subs.ext_detached.all.push(observer_id);
            }
            IDFilter::Is(id) => {
                subs.ext_attached
                    .by_xid
                    .entry(id.clone())
                    .or_default()
                    .push(observer_id);
                subs.ext_detached
                    .by_xid
                    .entry(id.clone())
                    .or_default()
                    .push(observer_id);
            }
            IDFilter::OneOf(ids) => {
                for id in ids {
                    subs.ext_attached
                        .by_xid
                        .entry(id.clone())
                        .or_default()
                        .push(observer_id);
                    subs.ext_detached
                        .by_xid
                        .entry(id.clone())
                        .or_default()
                        .push(observer_id);
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
                subs.ext_attached.all.retain(|o| *o != self.observer_id);
                subs.ext_detached.all.retain(|o| *o != self.observer_id);
            }
            IDFilter::Is(id) => {
                if let Some(observers) = subs.ext_attached.by_xid.get_mut(id) {
                    observers.retain(|o| *o != self.observer_id);
                }
                if let Some(observers) = subs.ext_detached.by_xid.get_mut(id) {
                    observers.retain(|o| *o != self.observer_id);
                }
            }
            IDFilter::OneOf(ids) => {
                for id in ids {
                    if let Some(observers) = subs.ext_attached.by_xid.get_mut(id) {
                        observers.retain(|o| *o != self.observer_id);
                    }
                    if let Some(observers) = subs.ext_detached.by_xid.get_mut(id) {
                        observers.retain(|o| *o != self.observer_id);
                    }
                }
            }
        }
    }
}

impl ObserverHandler for ExtensionObserver {
    fn on_ext_attached(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        extension: &Extension,
    ) -> Result<(), IglooError> {
        cm.send(
            self.client_id,
            IglooResponse::ObserverUpdate {
                query_id: self.query_id,
                result: U::ExtensionAttached(extension.id().clone(), true),
            },
        )
    }

    fn on_ext_detached(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        extension: &Extension,
    ) -> Result<(), IglooError> {
        cm.send(
            self.client_id,
            IglooResponse::ObserverUpdate {
                query_id: self.query_id,
                result: U::ExtensionAttached(extension.id().clone(), false),
            },
        )
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
            "ExtensionObserver should never receive component_set events"
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
            "ExtensionObserver should never receive component_put events"
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
            "ExtensionObserver should never receive device_created events"
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
            "ExtensionObserver should never receive device_deleted events"
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
            "ExtensionObserver should never receive device_renamed events"
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
            "ExtensionObserver should never receive entity_registered events"
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
            "ExtensionObserver should never receive group_created events"
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
            "ExtensionObserver should never receive group_deleted events"
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
            "ExtensionObserver should never receive group_renamed events"
        );
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
            "ExtensionObserver should never receive group_device_added events"
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
            "ExtensionObserver should never receive group_device_removed events"
        );
        Ok(())
    }
}
