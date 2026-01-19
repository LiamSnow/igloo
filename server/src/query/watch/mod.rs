use crate::query::watch::{
    comp::ComponentWatcher, device::DeviceWatcher, entity::EntityWatcher, ext::ExtensionWatcher,
    group::GroupWatcher,
};

pub mod dispatch;
pub mod subscriber;

mod comp;
mod device;
mod entity;
mod ext;
mod group;

pub type WatcherID = usize;
pub type WatcherList = Vec<WatcherID>;

pub enum Watcher {
    Devices(DeviceWatcher),
    Entities(EntityWatcher),
    Components(ComponentWatcher),
    Groups(GroupWatcher),
    Extensions(ExtensionWatcher),
}
