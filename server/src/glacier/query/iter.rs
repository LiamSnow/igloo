use std::{error::Error, slice::Iter};

use crate::glacier::{
    entity::{Entity, HasComponent},
    query::*,
    tree::{Device, DeviceTree},
};

pub struct EntityIterator<'a> {
    tree: &'a DeviceTree,
    filter: Option<QueryFilter>,
    state: EntityIteratorState<'a>,
}

enum EntityIteratorState<'a> {
    All {
        floe_iter: Iter<'a, Vec<Device>>,
        device_iter: Option<Iter<'a, Device>>,
        entity_iter: Option<(usize, &'a Device, Iter<'a, Entity>)>,
    },
    Zone {
        zone_devices: Iter<'a, (u16, u16)>,
        entity_iter: Option<(usize, &'a Device, Iter<'a, Entity>)>,
    },
    Device {
        device: &'a Device,
        entity_iter: Iter<'a, Entity>,
    },
    Entity {
        device: &'a Device,
        entity: Option<&'a Entity>,
    },
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = (&'a Device, &'a Entity);

    fn next(&mut self) -> Option<Self::Item> {
        let filter = self.filter.as_ref();
        loop {
            match &mut self.state {
                EntityIteratorState::All {
                    floe_iter,
                    device_iter,
                    entity_iter,
                } => {
                    if let Some((_, device, entities)) = entity_iter
                        && let Some(entity) = entities.next()
                    {
                        return Some((device, entity));
                    }

                    if let Some(devices) = device_iter
                        && let Some(device) = devices.next()
                    {
                        if filter.is_none_or(|f| device.presense.matches_filter(f)) {
                            *entity_iter = Some((0, device, device.entities.iter()));
                            continue;
                        }
                        continue;
                    }

                    if let Some(devices) = floe_iter.next() {
                        *device_iter = Some(devices.iter());
                        continue;
                    }

                    return None;
                }
                EntityIteratorState::Zone {
                    zone_devices,
                    entity_iter,
                } => {
                    if let Some((_, device, entities)) = entity_iter
                        && let Some(entity) = entities.next()
                    {
                        return Some((device, entity));
                    }

                    for (floe_idx, device_idx) in zone_devices.by_ref() {
                        let device = &self.tree.devices[*floe_idx as usize][*device_idx as usize];
                        if filter.is_none_or(|f| device.presense.matches_filter(f)) {
                            *entity_iter = Some((0, device, device.entities.iter()));
                            break;
                        }
                    }

                    if entity_iter.is_some() {
                        continue;
                    }
                    return None;
                }
                EntityIteratorState::Device {
                    device,
                    entity_iter,
                } => {
                    return entity_iter.next().map(|entity| (*device, entity));
                }
                EntityIteratorState::Entity { device, entity } => {
                    return entity.take().map(|e| (*device, e));
                }
            }
        }
    }
}

pub fn iter_entities<'a>(
    tree: &'a DeviceTree,
    target: QueryTarget,
    filter: Option<QueryFilter>,
) -> Result<EntityIterator<'a>, Box<dyn Error>> {
    let state = match target {
        QueryTarget::All => EntityIteratorState::All {
            floe_iter: tree.devices.iter(),
            device_iter: None,
            entity_iter: None,
        },
        QueryTarget::Zone(zone_id) => {
            let Some(zone_idx) = tree.zone_idx_lut.get(&zone_id) else {
                return Err(format!("Invalid zone ID '{zone_id}'").into());
            };
            let zone = &tree.zones[*zone_idx as usize];
            EntityIteratorState::Zone {
                zone_devices: zone.idxs.iter(),
                entity_iter: None,
            }
        }
        QueryTarget::Device(floe_id, device_id) => {
            let Some(floe_idx) = tree.floe_idx_lut.get(&floe_id) else {
                return Err(format!("Invalid Floe ID '{floe_id}'").into());
            };
            let Some(device_idx) = tree.device_idx_lut[*floe_idx as usize].get(&device_id) else {
                return Err(format!("Invalid device ID '{device_id}'").into());
            };
            let device = &tree.devices[*floe_idx as usize][*device_idx as usize];

            if let Some(filter) = &filter {
                if !device.presense.matches_filter(filter) {
                    EntityIteratorState::Entity {
                        device,
                        entity: None,
                    }
                } else {
                    EntityIteratorState::Device {
                        device,
                        entity_iter: device.entities.iter(),
                    }
                }
            } else {
                EntityIteratorState::Device {
                    device,
                    entity_iter: device.entities.iter(),
                }
            }
        }
        QueryTarget::Entity(floe_id, device_id, entity_id) => {
            let Some(floe_idx) = tree.floe_idx_lut.get(&floe_id) else {
                return Err(format!("Invalid Floe ID '{floe_id}'").into());
            };
            let Some(device_idx) = tree.device_idx_lut[*floe_idx as usize].get(&device_id) else {
                return Err(format!("Invalid device ID '{device_id}'").into());
            };
            let device = &tree.devices[*floe_idx as usize][*device_idx as usize];
            let Some(entity_idx) = device.entity_idx_lut.get(&entity_id) else {
                return Err(format!("Invalid entity ID '{entity_id}'").into());
            };

            let entity = if let Some(filter) = &filter {
                if device.presense.matches_filter(filter) {
                    Some(&device.entities[*entity_idx])
                } else {
                    None
                }
            } else {
                Some(&device.entities[*entity_idx])
            };

            EntityIteratorState::Entity { device, entity }
        }
    };

    Ok(EntityIterator {
        tree,
        filter,
        state,
    })
}
