use crate::glacier::{entity::HasComponent, query::*, tree::DeviceTree};
use igloo_interface::{
    Component, SelectEntity, StartTransaction,
    query::{
        DeviceSnapshot, EntitySnapshot, FloeSnapshot, GroupSnapshot, QueryFilter, QueryTarget,
        SetQuery, Snapshot,
    },
};
use std::{error::Error, time::Duration};

pub trait QueryExec {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>>;
}

impl QueryExec for Query {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
        match self {
            Query::Set(q) => q.execute(tree).await,
            Query::GetOne(q) => q.execute(tree).await,
            Query::GetAll(q) => q.execute(tree).await,
            Query::GetAggregate(q) => q.execute(tree).await,
            Query::Watch(q) => q.execute(tree).await,
            Query::Snapshot(q) => q.execute(tree).await,
        }
    }
}

impl QueryExec for SetQuery {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
        let mut applicable = Vec::with_capacity(10);

        match self.target {
            QueryTarget::All => {
                for (did, device) in tree.iter_devices() {
                    if device.owner_ref().is_none() || !device.presense.matches_filter(&self.filter)
                    {
                        continue;
                    }

                    let mut applicable_entities = Vec::with_capacity(5);
                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&self.filter) {
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
                    if device.owner_ref().is_none() || !device.presense.matches_filter(&self.filter)
                    {
                        continue;
                    }

                    let mut applicable_entities = Vec::with_capacity(5);
                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&self.filter) {
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
                    if device.presense.matches_filter(&self.filter) {
                        for (eidx, entity) in device.entities.iter().enumerate() {
                            if entity.matches_filter(&self.filter) {
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

                    if entity.matches_filter(&self.filter) {
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

                for value in &self.values {
                    floe.writer.write_component(value).await?;
                }

                floe.writer.deselect_entity().await?;
            }

            floe.writer.end_transaction().await?;

            floe.writer.flush().await?;
        }

        Ok(())
    }
}

impl QueryExec for GetOneQuery {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
        let filter = match self.filter {
            QueryFilter::None => QueryFilter::With(self.comp),
            f => QueryFilter::And(Box::new((f, QueryFilter::With(self.comp)))),
        };

        let res = match self.target {
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

                        res = Some(QueryResult {
                            device: did,
                            entity: eidx,
                            value: entity.get(self.comp).unwrap().clone(),
                            tag: self.tag,
                        });
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

                        res = Some(QueryResult {
                            device: did,
                            entity: eidx,
                            value: entity.get(self.comp).unwrap().clone(),
                            tag: self.tag,
                        });

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

                        res = Some(QueryResult {
                            device: did,
                            entity: eidx,
                            value: entity.get(self.comp).unwrap().clone(),
                            tag: self.tag,
                        });

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
                    Some(QueryResult {
                        device: did,
                        entity: *eidx,
                        value: entity.get(self.comp).unwrap().clone(),
                        tag: self.tag,
                    })
                } else {
                    None
                }
            }
        };

        self.response_tx
            .send(res)
            .map_err(|_| "Failed to send query result. Channel closed".into())
    }
}

impl QueryExec for GetAllQuery {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
        let mut res = Vec::with_capacity(20);

        let filter = match self.filter {
            QueryFilter::None => QueryFilter::With(self.comp),
            f => QueryFilter::And(Box::new((f, QueryFilter::With(self.comp)))),
        };

        match self.target {
            QueryTarget::All => {
                for (did, device) in tree.iter_devices() {
                    if !device.presense.matches_filter(&filter) {
                        continue;
                    }

                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            res.push(QueryResult {
                                device: did,
                                entity: eidx,
                                value: entity.get(self.comp).unwrap().clone(),
                                tag: self.tag,
                            });
                        }
                    }
                }
            }
            QueryTarget::Group(gid) => {
                for (did, device) in tree.iter_devices_in_group(gid) {
                    if !device.presense.matches_filter(&filter) {
                        continue;
                    }

                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            res.push(QueryResult {
                                device: did,
                                entity: eidx,
                                value: entity.get(self.comp).unwrap().clone(),
                                tag: self.tag,
                            });
                        }
                    }
                }
            }
            QueryTarget::Device(did) => {
                let device = tree.device(did)?;
                if device.presense.matches_filter(&filter) {
                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            res.push(QueryResult {
                                device: did,
                                entity: eidx,
                                value: entity.get(self.comp).unwrap().clone(),
                                tag: self.tag,
                            });
                        }
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
                    res.push(QueryResult {
                        device: did,
                        entity: *eidx,
                        value: entity.get(self.comp).unwrap().clone(),
                        tag: self.tag,
                    });
                }
            }
        }

        self.response_tx
            .send(res)
            .map_err(|_| "Failed to send query result. Channel closed".into())
    }
}

impl QueryExec for GetAggregateQuery {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
        let filter = match self.filter {
            QueryFilter::None => QueryFilter::With(self.comp),
            f => QueryFilter::And(Box::new((f, QueryFilter::With(self.comp)))),
        };

        let mut items = Vec::with_capacity(20);

        match self.target {
            QueryTarget::All => {
                for (_, device) in tree.iter_devices() {
                    if !device.presense.matches_filter(&filter) {
                        continue;
                    }

                    for entity in &device.entities {
                        if !entity.matches_filter(&filter) {
                            continue;
                        }

                        items.push(entity.get(self.comp).unwrap().clone());
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

                        items.push(entity.get(self.comp).unwrap().clone());
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

                        items.push(entity.get(self.comp).unwrap().clone());
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
                    items.push(entity.get(self.comp).unwrap().clone());
                }
            }
        }

        let res = QueryAggregateResult {
            result: Component::aggregate(items, self.op),
            tag: self.tag,
        };

        self.response_tx
            .send(res)
            .map_err(|_| "Failed to send query result. Channel closed".into())
    }
}

impl QueryExec for WatchQuery {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
        let filter = match self.filter {
            QueryFilter::None => QueryFilter::With(self.comp),
            f => QueryFilter::And(Box::new((f, QueryFilter::With(self.comp)))),
        };

        let mut query = AttachedQuery {
            filter: filter.clone(),
            tx: self.update_tx.clone(),
            gid: None,
            tag: self.tag,
        };

        // send all initially + register for persistence
        match self.target {
            QueryTarget::All => {
                tree.attach_query_to_all(self.comp, query)?;

                for (did, device) in tree.iter_devices() {
                    if !device.presense.matches_filter(&filter) {
                        continue;
                    }

                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            let comp = entity.get(self.comp).unwrap().clone();
                            if let Err(e) = self
                                .update_tx
                                .send_timeout(
                                    QueryResult {
                                        device: did,
                                        entity: eidx,
                                        value: comp,
                                        tag: self.tag,
                                    },
                                    Duration::from_millis(10),
                                )
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
                tree.attach_query_to_group(gid, self.comp, query)?;

                for (did, device) in tree.iter_devices_in_group(gid) {
                    if !device.presense.matches_filter(&filter) {
                        continue;
                    }

                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            let comp = entity.get(self.comp).unwrap().clone();
                            if let Err(e) = self
                                .update_tx
                                .send_timeout(
                                    QueryResult {
                                        device: did,
                                        entity: eidx,
                                        value: comp,
                                        tag: self.tag,
                                    },
                                    Duration::from_millis(10),
                                )
                                .await
                            {
                                eprintln!("Failed to send init watch result: {e}");
                            }
                        }
                    }
                }
            }
            QueryTarget::Device(did) => {
                tree.attach_query(did, self.comp, query)?;

                let device = tree.device(did)?;
                if device.presense.matches_filter(&filter) {
                    for (eidx, entity) in device.entities.iter().enumerate() {
                        if entity.matches_filter(&filter) {
                            let comp = entity.get(self.comp).unwrap().clone();
                            if let Err(e) = self
                                .update_tx
                                .send_timeout(
                                    QueryResult {
                                        device: did,
                                        entity: eidx,
                                        value: comp,
                                        tag: self.tag,
                                    },
                                    Duration::from_millis(10),
                                )
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
                        let comp = entity.get(self.comp).unwrap().clone();
                        if let Err(e) = self
                            .update_tx
                            .send_timeout(
                                QueryResult {
                                    device: did,
                                    entity: *eidx,
                                    value: comp,
                                    tag: self.tag,
                                },
                                Duration::from_millis(10),
                            )
                            .await
                        {
                            eprintln!("Failed to send init watch result: {e}");
                        }
                    }

                    tree.attach_entity_query(did, *eidx, self.comp, query)?;
                } else {
                    tree.attach_pending_entity_query(did, eid, self.comp, query)?;
                }
            }
        }

        Ok(())
    }
}

impl QueryExec for SnapshotQuery {
    async fn execute(self, tree: &mut DeviceTree) -> Result<(), Box<dyn Error>> {
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

        match self.response_tx.send(snap).is_err() {
            true => Err("Failed to send query result. Channel closed".into()),
            false => Ok(()),
        }
    }
}
