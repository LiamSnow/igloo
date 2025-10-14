use std::{error::Error, time::Duration};

use igloo_interface::{Component, ComponentAverage, ComponentType, SelectEntity, StartTransaction};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use tokio::sync::{mpsc, oneshot};

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
        QueryKind::WatchAll(prefix, tx, comp_type) => {
            handle_watch_all_query(tree, prefix, tx, comp_type, query.filter, query.target).await
        }
    }
}

async fn handle_get_one_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Option<OneQueryResult>>,
    comp_type: ComponentType,
    filter: QueryFilter,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let filter = match filter {
        QueryFilter::None => QueryFilter::With(comp_type),
        f => QueryFilter::And(Box::new((f, QueryFilter::With(comp_type)))),
    };

    let res = match target {
        QueryTarget::All => {
            let mut res = None;
            'outer: for (did, device) in tree.iter_devices() {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for (eidx, entity) in device.entities.iter().enumerate() {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res = Some((did, eidx, entity.get(comp_type).unwrap().clone()));
                    break 'outer;
                }
            }
            res
        }
        QueryTarget::Group(group) => {
            let mut res = None;
            'outer: for (did, device) in tree.iter_devices_in_group(group) {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for (eidx, entity) in device.entities.iter().enumerate() {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res = Some((did, eidx, entity.get(comp_type).unwrap().clone()));
                    break 'outer;
                }
            }
            res
        }
        QueryTarget::Device(did) => {
            let device = tree.device(did)?;
            let mut res = None;
            if device.presense.matches_filter(&filter) {
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    res = Some((did, eidx, entity.get(comp_type).unwrap().clone()));
                    break;
                }
            }
            res
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            let Some(eidx) = device.get_entity_idx(&eid) else {
                return Err("invalid entity ID".into());
            };
            let entity = &device.entities[*eidx];

            if entity.matches_filter(&filter) {
                Some((did, *eidx, entity.get(comp_type).unwrap().clone()))
            } else {
                None
            }
        }
    };

    tx.send(res)
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_watch_all_query(
    tree: &mut DeviceTree,
    prefix: u32,
    tx: mpsc::Sender<PrefixedOneQueryResult>,
    comp_type: ComponentType,
    filter: QueryFilter,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let filter = match filter {
        QueryFilter::None => QueryFilter::With(comp_type),
        f => QueryFilter::And(Box::new((f, QueryFilter::With(comp_type)))),
    };

    let mut query = WatchQuery {
        filter: filter.clone(),
        tx: tx.clone(),
        gid: None,
        prefix,
    };

    // send all initially + register for persistence
    match target {
        QueryTarget::All => {
            tree.attach_query_to_all(comp_type, query)?;

            for (did, device) in tree.iter_devices() {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        let comp = entity.get(comp_type).unwrap().clone();
                        if let Err(e) = tx
                            .send_timeout((prefix, did, eidx, comp), Duration::from_millis(10))
                            .await
                        {
                            eprintln!("Failed to send init watch result: {e}");
                        }
                    }
                }
            }
        }
        QueryTarget::Group(gid) => {
            query.gid = Some(gid);
            tree.attach_query_to_group(gid, comp_type, query)?;

            for (did, device) in tree.iter_devices_in_group(gid) {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        let comp = entity.get(comp_type).unwrap().clone();
                        if let Err(e) = tx
                            .send_timeout((prefix, did, eidx, comp), Duration::from_millis(10))
                            .await
                        {
                            eprintln!("Failed to send init watch result: {e}");
                        }
                    }
                }
            }
        }
        QueryTarget::Device(did) => {
            tree.attach_query(did, comp_type, query)?;

            let device = tree.device(did)?;
            if device.presense.matches_filter(&filter) {
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        let comp = entity.get(comp_type).unwrap().clone();
                        if let Err(e) = tx
                            .send_timeout((prefix, did, eidx, comp), Duration::from_millis(10))
                            .await
                        {
                            eprintln!("Failed to send init watch result: {e}");
                        }
                    }
                }
            }
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            if let Some(eidx) = device.get_entity_idx(&eid) {
                // we are not going to error on invalid entity IDs
                // here, because maybe its not registered yet
                // BUT will recieve updates later
                let entity = &device.entities[*eidx];

                if entity.matches_filter(&filter) {
                    let comp = entity.get(comp_type).unwrap().clone();
                    if let Err(e) = tx
                        .send_timeout((prefix, did, *eidx, comp), Duration::from_millis(10))
                        .await
                    {
                        eprintln!("Failed to send init watch result: {e}");
                    }
                }

                tree.attach_entity_query(did, *eidx, comp_type, query)?;
            } else {
                tree.attach_pending_entity_query(did, eid, comp_type, query)?;
            }
        }
    }

    Ok(())
}

async fn handle_get_all_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<GetAllQueryResult>,
    comp_type: ComponentType,
    filter: QueryFilter,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let mut res = GetAllQueryResult::default();

    let filter = match filter {
        QueryFilter::None => QueryFilter::With(comp_type),
        f => QueryFilter::And(Box::new((f, QueryFilter::With(comp_type)))),
    };

    match target {
        QueryTarget::All => {
            for (did, device) in tree.iter_devices() {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                let mut emap = FxHashMap::default();
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        emap.insert(eidx, entity.get(comp_type).unwrap().clone());
                    }
                }
                if !emap.is_empty() {
                    res.insert(did, emap);
                }
            }
        }
        QueryTarget::Group(gid) => {
            for (did, device) in tree.iter_devices_in_group(gid) {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                let mut emap = FxHashMap::default();
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        emap.insert(eidx, entity.get(comp_type).unwrap().clone());
                    }
                }
                if !emap.is_empty() {
                    res.insert(did, emap);
                }
            }
        }
        QueryTarget::Device(did) => {
            let device = tree.device(did)?;
            if device.presense.matches_filter(&filter) {
                let mut emap = FxHashMap::default();
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        emap.insert(eidx, entity.get(comp_type).unwrap().clone());
                    }
                }
                if !emap.is_empty() {
                    res.insert(did, emap);
                }
            }
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            let Some(eidx) = device.get_entity_idx(&eid) else {
                return Err("invalid entity ID".into());
            };
            let entity = &device.entities[*eidx];

            if entity.matches_filter(&filter) {
                let mut emap = FxHashMap::default();
                emap.insert(*eidx, entity.get(comp_type).unwrap().clone());
                res.insert(did, emap);
            }
        }
    }

    tx.send(res)
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_set_query(
    tree: &mut DeviceTree,
    comps: SmallVec<[Component; 2]>,
    filter: QueryFilter,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let mut applicable = Vec::with_capacity(10);

    match target {
        QueryTarget::All => {
            for (did, device) in tree.iter_devices() {
                if device.owner_ref().is_none() || !device.presense.matches_filter(&filter) {
                    continue;
                }

                let mut applicable_entities = Vec::with_capacity(5);
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        applicable_entities.push(eidx);
                    }
                }
                if !applicable_entities.is_empty() {
                    applicable.push((did, applicable_entities));
                }
            }
        }
        QueryTarget::Group(gid) => {
            for (did, device) in tree.iter_devices_in_group(gid) {
                if device.owner_ref().is_none() || !device.presense.matches_filter(&filter) {
                    continue;
                }

                let mut applicable_entities = Vec::with_capacity(5);
                for (eidx, entity) in device.entities.iter().enumerate() {
                    if entity.matches_filter(&filter) {
                        applicable_entities.push(eidx);
                    }
                }
                if !applicable_entities.is_empty() {
                    applicable.push((did, applicable_entities));
                }
            }
        }
        QueryTarget::Device(did) => {
            let device = tree.device(did)?;
            if device.owner_ref().is_some() {
                let mut applicable_entities = Vec::with_capacity(5);
                if device.presense.matches_filter(&filter) {
                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            applicable_entities.push(eidx);
                        }
                    }
                }
                if !applicable_entities.is_empty() {
                    applicable.push((did, applicable_entities));
                }
            }
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            if device.owner_ref().is_some() {
                let Some(eidx) = device.get_entity_idx(&eid) else {
                    return Err("invalid entity ID".into());
                };
                let entity = &device.entities[*eidx];

                if entity.matches_filter(&filter) {
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

        floe.writer.flush().await?;
    }

    Ok(())
}

async fn handle_get_avg_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Option<Component>>,
    comp_type: ComponentType,
    filter: QueryFilter,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let Some(mut avg) = ComponentAverage::new(comp_type) else {
        // not avgable
        return tx
            .send(None)
            .map_err(|_| "Failed to send query result. Channel closed".into());
    };

    let filter = match filter {
        QueryFilter::None => QueryFilter::With(comp_type),
        f => QueryFilter::And(Box::new((f, QueryFilter::With(comp_type)))),
    };

    match target {
        QueryTarget::All => {
            for (_, device) in tree.iter_devices() {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    avg.add(entity.get(comp_type).unwrap().clone());
                }
            }
        }
        QueryTarget::Group(gid) => {
            for (_, device) in tree.iter_devices_in_group(gid) {
                if !device.presense.matches_filter(&filter) {
                    continue;
                }

                for entity in &device.entities {
                    if !entity.matches_filter(&filter) {
                        continue;
                    }

                    avg.add(entity.get(comp_type).unwrap().clone());
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

                    avg.add(entity.get(comp_type).unwrap().clone());
                }
            }
        }
        QueryTarget::Entity(did, eid) => {
            let device = tree.device(did)?;
            let Some(entity_idx) = device.get_entity_idx(&eid) else {
                return Err("invalid entity ID".into());
            };
            let entity = &device.entities[*entity_idx];

            if entity.matches_filter(&filter) {
                avg.add(entity.get(comp_type).unwrap().clone());
            }
        }
    }

    tx.send(avg.current_average())
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_snapshot_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Snapshot>,
    _filter: QueryFilter,
    _target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    // TODO should snapshot respect Target and/or Filter?

    let mut snap = Snapshot::default();

    for (id, device) in tree.iter_devices() {
        let mut esnaps = Vec::with_capacity(device.entities.len());

        for (eid, eidx) in device.entity_idx_lut() {
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

    for (gid, group) in tree.iter_groups() {
        snap.groups.push(GroupSnapshot {
            id: gid,
            name: group.name().to_string(),
            devices: group.devices().to_vec(),
        });
    }

    match tx.send(snap).is_err() {
        true => Err("Failed to send query result. Channel closed".into()),
        false => Ok(()),
    }
}
