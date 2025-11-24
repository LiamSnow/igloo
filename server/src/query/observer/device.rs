use crate::{
    core::IglooError,
    query::{
        QueryContext,
        observer::{dispatch::ObserverHandler, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Floe, Group},
};
use igloo_interface::{
    ComponentType,
    id::{DeviceID, FloeRef, GroupID},
    query::DeviceQuery,
};
use rustc_hash::FxHashSet;

#[allow(dead_code)]
pub struct DeviceObserver {
    pub client_id: usize,
    pub query_id: usize,
    pub query: DeviceQuery,
    pub matched: FxHashSet<DeviceID>,
}

impl DeviceObserver {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        _ctx: &mut QueryContext,
        _subscribers: &mut TreeSubscribers,
        _tree: &DeviceTree,
        _query_id: usize,
        _observer_id: usize,
        _client_id: usize,
        _query: DeviceQuery,
    ) -> Self {
        todo!()
    }
}

impl ObserverHandler for DeviceObserver {
    #[allow(dead_code)]
    fn on_component_set(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _device: &Device,
        _entity_index: usize,
        _comp_type: ComponentType,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_component_put(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _device: &Device,
        _entity_index: usize,
        _comp_type: ComponentType,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_device_created(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _device: &Device,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_device_deleted(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _device: &Device,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_device_renamed(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _device: &Device,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_entity_registered(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _device: &Device,
        _entity_index: usize,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_group_created(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _group: &Group,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_group_deleted(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _gid: &GroupID,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_group_renamed(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _group: &Group,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_group_membership_changed(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _group: &Group,
        _device: &Device,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_floe_attached(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _floe: &Floe,
    ) -> Result<(), IglooError> {
        todo!()
    }

    #[allow(dead_code)]
    fn on_floe_detached(
        &mut self,
        _ctx: &mut QueryContext,
        _tree: &DeviceTree,
        _fref: &FloeRef,
    ) -> Result<(), IglooError> {
        todo!()
    }
}
