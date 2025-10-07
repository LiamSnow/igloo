use std::error::Error;

use igloo_interface::{Component, ComponentType};
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

async fn handle_set_query(
    tree: &mut DeviceTree,
    comps: Vec<Component>,
    filter: Option<QueryFilter>,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    match target {
        QueryTarget::All => todo!(),
        QueryTarget::Zone(zone_id) => todo!(),
        QueryTarget::Device(floe_id, device_id) => todo!(),
        QueryTarget::Entity(floe_id, device_id, entity_id) => todo!(),
    }
}

async fn handle_get_one_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Option<Component>>,
    comp_type: ComponentType,
    filter: Option<QueryFilter>,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let entities = iter_entities(tree, target, filter)?;

    for (device, entity) in entities {
        if device.presense.has(comp_type)
            && let Some(comp) = entity.get(comp_type).cloned()
        {
            return tx
                .send(Some(comp))
                .map_err(|_| "Failed to send query result. Channel closed".into());
        }
    }

    tx.send(None)
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_get_all_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Vec<Component>>,
    comp_type: ComponentType,
    filter: Option<QueryFilter>,
    target: QueryTarget,
) -> Result<(), Box<dyn Error>> {
    let entities = iter_entities(tree, target, filter)?;

    let components: Vec<Component> = entities
        .filter(|(device, _)| device.presense.has(comp_type))
        .filter_map(|(_, entity)| entity.get(comp_type).cloned())
        .collect();

    tx.send(components)
        .map_err(|_| "Failed to send query result. Channel closed".into())
}

async fn handle_get_avg_query(
    tree: &mut DeviceTree,
    tx: oneshot::Sender<Option<Component>>,
    comp_type: ComponentType,
    filter: Option<QueryFilter>,
    target: QueryTarget,
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

    for ((floe_id, device_id), device_name) in &tree.device_names {
        let mut device_snap = DeviceSnapshot {
            id: device_id.clone(),
            name: device_name.clone(),
            floe_id: floe_id.clone(),
            idx: None,
            floe_idx: None,
            entities: vec![],
        };

        device_snap.floe_idx = tree.floe_idx_lut.get(floe_id).cloned();

        if let Some(floe_idx) = device_snap.floe_idx {
            device_snap.idx = tree.device_idx_lut[floe_idx as usize]
                .get(device_id)
                .cloned();

            if let Some(device_idx) = device_snap.idx {
                let device = &tree.devices[floe_idx as usize][device_idx as usize];

                for (entity_name, entity_idx) in &device.entity_idx_lut {
                    let entity = &device.entities[*entity_idx];

                    device_snap.entities.push(EntitySnapshot {
                        name: entity_name.clone(),
                        components: entity.get_comps().to_vec(),
                    });
                }
            }
        }

        snap.devices.push(device_snap);
    }

    for (floe_id, floe_idx) in &tree.floe_idx_lut {
        let floe = &tree.floes[*floe_idx as usize];
        snap.floes.push(FloeSnapshot {
            id: floe_id.clone(),
            idx: *floe_idx,
            max_supported_component: floe.max_supported_component,
        });
    }

    for (zone_id, zone_idx) in &tree.zone_idx_lut {
        let zone = &tree.zones[*zone_idx as usize];
        snap.zones.push(ZoneSnapshot {
            id: zone_id.clone(),
            idx: *zone_idx,
            name: zone.name.clone(),
            disabled: zone.disabled,
        });
    }

    match tx.send(snap).is_err() {
        true => Err("Failed to send query result. Channel closed".into()),
        false => Ok(()),
    }
}
