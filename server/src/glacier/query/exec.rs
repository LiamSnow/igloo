use std::error::Error;

use igloo_interface::{Component, ComponentType, SelectEntity, StartTransaction};
use tokio::sync::oneshot;

use crate::glacier::{entity::HasComponent, query::*, tree::DeviceTree};

pub async fn handle_query(tree: &mut DeviceTree, query: Query) -> Result<(), Box<dyn Error>> {
    match query.kind {
        QueryKind::Set(comps) => handle_set_query(tree, comps, query.filter, query.target).await,
        QueryKind::GetOne(tx, comp_type) => {
            handle_get_one_query(tree, tx, comp_type, query.filter, query.target).await
        }
        QueryKind::GetAll(tx, comp_type) => {
            handle_get_all_query(tree, tx, comp_type, query.filter, query.target).await
        }
        QueryKind::GetAvg(tx, comp_type) => {
            handle_get_avg_query(tree, tx, comp_type, query.filter, query.target).await
        }
        QueryKind::Snapshot(tx) => {
            handle_snapshot_query(tree, tx, query.filter, query.target).await
        }
    }
}

async fn handle_get_one_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Option<Component>>,
    comp_type: ComponentType,
    filter: Option<QueryFilter>,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let filter = match filter {
        Some(f) => QueryFilter::And(Box::new((f, QueryFilter::With(comp_type)))),
        None => QueryFilter::With(comp_type),
    };

    let res = match target {
        QueryTarget::All => {
            let mut res = None;
            'outer: for device in tree.iter_devices() {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res = entity.get(comp_type);
                    break 'outer;
                }
            }
            res
        }
        QueryTarget::Zone(zid) => {
            let mut res = None;
            'outer: for device in tree.iter_devices_in_zone(zid) {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res = entity.get(comp_type);
                    break 'outer;
                }
            }
            res
        }
        QueryTarget::Device(did) => {
            let device = tree.device(did)?;
            let mut res = None;
            if device.presense.matches_filter(&filter) {
                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res = entity.get(comp_type);
                    break;
                }
            }
            res
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            let Some(entity_idx) = device.entity_idx_lut.get(&eid) else {
                return Err("invalid entity ID".into());
            };
            let entity = &device.entities[*entity_idx];

            if entity.matches_filter(&filter) {
                entity.get(comp_type)
            } else {
                None
            }
        }
    };

    tx.send(res.cloned())
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_get_all_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Vec<Component>>,
    comp_type: ComponentType,
    filter: Option<QueryFilter>,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let mut res: Vec<Component> = Vec::with_capacity(10);

    let filter = match filter {
        Some(f) => QueryFilter::And(Box::new((f, QueryFilter::With(comp_type)))),
        None => QueryFilter::With(comp_type),
    };

    match target {
        QueryTarget::All => {
            for device in tree.iter_devices() {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res.push(entity.get(comp_type).unwrap().clone());
                }
            }
        }
        QueryTarget::Zone(zid) => {
            for device in tree.iter_devices_in_zone(zid) {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res.push(entity.get(comp_type).unwrap().clone());
                }
            }
        }
        QueryTarget::Device(did) => {
            let device = tree.device(did)?;
            if device.presense.matches_filter(&filter) {
                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res.push(entity.get(comp_type).unwrap().clone());
                }
            }
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            let Some(entity_idx) = device.entity_idx_lut.get(&eid) else {
                return Err("invalid entity ID".into());
            };
            let entity = &device.entities[*entity_idx];

            if entity.matches_filter(&filter) {
                res.push(entity.get(comp_type).unwrap().clone());
            }
        }
    }

    tx.send(res)
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_set_query(
    tree: &mut DeviceTree,
    comps: Vec<Component>,
    filter: Option<QueryFilter>,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let mut applicable = Vec::with_capacity(10);

    match target {
        QueryTarget::All => {
            for (did, device) in tree.iter_devices_with_ids() {
                if device.owner_ref().is_none() || !device.presense.matches_filter_opt(&filter) {
                    continue;
                }

                let mut applicable_entities = Vec::with_capacity(5);
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter_opt(&filter) {
                        applicable_entities.push(eidx);
                    }
                }
                if applicable_entities.len() > 0 {
                    applicable.push((did, applicable_entities));
                }
            }
        }
        QueryTarget::Zone(zid) => {
            for (did, device) in tree.iter_devices_in_zone_with_ids(zid) {
                if device.owner_ref().is_none() || !device.presense.matches_filter_opt(&filter) {
                    continue;
                }

                let mut applicable_entities = Vec::with_capacity(5);
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter_opt(&filter) {
                        applicable_entities.push(eidx);
                    }
                }
                if applicable_entities.len() > 0 {
                    applicable.push((did, applicable_entities));
                }
            }
        }
        QueryTarget::Device(did) => {
            let device = tree.device(did)?;
            if device.owner_ref().is_some() {
                let mut applicable_entities = Vec::with_capacity(5);
                if device.presense.matches_filter_opt(&filter) {
                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter_opt(&filter) {
                            applicable_entities.push(eidx);
                        }
                    }
                }
                if applicable_entities.len() > 0 {
                    applicable.push((did, applicable_entities));
                }
            }
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            if device.owner_ref().is_some() {
                let Some(eidx) = device.entity_idx_lut.get(&eid) else {
                    return Err("invalid entity ID".into());
                };
                let entity = &device.entities[*eidx];

                if entity.matches_filter_opt(&filter) {
                    applicable.push((did, vec![*eidx]));
                }
            }
        }
    }

    for (did, eidxs) in applicable {
        let device = tree.device(did).unwrap();
        let floe = tree.floe_mut(device.owner_ref().unwrap());

        floe.writer
            .start_transaction(&StartTransaction {
                device_id: did.take(),
            })
            .await?;

        for eidx in eidxs {
            floe.writer
                .select_entity(&SelectEntity {
                    entity_idx: eidx as u32,
                })
                .await?;

            for comp in &comps {
                floe.writer.write_component(comp).await?;
            }

            floe.writer.deselect_entity().await?;
        }

        floe.writer.end_transaction().await?;
    }

    Ok(())
}

async fn handle_get_avg_query(
    _tree: &mut DeviceTree,
    _tx: oneshot::Sender<Option<Component>>,
    _comp_type: ComponentType,
    _filter: Option<QueryFilter>,
    _target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    todo!()
}

async fn handle_snapshot_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Snapshot>,
    _filter: Option<QueryFilter>,
    _target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    // TODO should snapshot respect Target or Filter?

    let mut snap = Snapshot::default();

    for (id, device) in tree.iter_devices_with_ids() {
        let mut esnaps = Vec::with_capacity(device.entities.len());

        for (eid, eidx) in &device.entity_idx_lut {
            let entity = &device.entities[*eidx];
            esnaps.push(EntitySnapshot {
                name: eid.to_string(),
                components: entity.get_comps().to_vec(),
            });
        }

        snap.devices.push(DeviceSnapshot {
            id,
            name: device.name().to_string(),
            owner: device.owner().clone(),
            entities: esnaps,
        });
    }

    for (fid, fref) in tree.floe_ref_lut() {
        let floe = tree.floe(*fref);
        snap.floes.push(FloeSnapshot {
            id: fid.clone(),
            fref: *fref,
            max_supported_component: floe.max_supported_component,
        });
    }

    for (zid, zone) in tree.iter_zones_with_ids() {
        snap.zones.push(ZoneSnapshot {
            id: zid,
            name: zone.name().to_string(),
            devices: zone.devices().to_vec(),
        });
    }

    match tx.send(snap).is_err() {
        true => Err("Failed to send query result. Channel closed".into()),
        false => Ok(()),
    }
}
