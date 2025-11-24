use crate::query::observer::{
    comp::ComponentObserver, device::DeviceObserver, entity::EntityObserver, floe::FloeObserver,
    group::GroupObserver,
};

pub mod dispatch;
pub mod subscriber;

mod comp;
mod device;
mod entity;
mod floe;
mod group;

pub type ObserverID = usize;
pub type ObserverList = Vec<ObserverID>;

pub enum Observer {
    Devices(DeviceObserver),
    Entities(EntityObserver),
    Components(ComponentObserver),
    Groups(GroupObserver),
    Floes(FloeObserver),
}
