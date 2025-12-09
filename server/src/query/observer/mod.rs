use crate::query::observer::{
    comp::ComponentObserver, device::DeviceObserver, entity::EntityObserver,
    ext::ExtensionObserver, group::GroupObserver,
};

pub mod dispatch;
pub mod subscriber;

mod comp;
mod device;
mod entity;
mod ext;
mod group;

pub type ObserverID = usize;
pub type ObserverList = Vec<ObserverID>;

pub enum Observer {
    Devices(DeviceObserver),
    Entities(EntityObserver),
    Components(ComponentObserver),
    Groups(GroupObserver),
    Extensions(ExtensionObserver),
}
