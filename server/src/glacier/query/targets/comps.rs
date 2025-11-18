use crate::glacier::{
    query::{EngineError, QueryEngine, QueryResponse, collector::QueryCollector},
    tree::DeviceTree,
};
use igloo_interface::{
    Component, ComponentType, SelectEntity, StartTransaction,
    id::{DeviceID, FloeRef},
    query::{ComponentResult, Query, QueryAction as A, QueryResult, QueryResultValue as R},
    types::{IglooValue, agg::Aggregatable},
};

impl QueryEngine {
    pub async fn evaluate_comps(
        &mut self,
        tree: &mut DeviceTree,
        mut query: Query,
        comp_type: ComponentType,
    ) -> Result<QueryResponse, EngineError> {
        let action = query.action.clone();
        let matched = query.collect_matching_entities(self, tree);

        let value = match action {
            A::Count => R::Count(matched.len()),

            A::GetIds => {
                panic!("GetIds is invalid for Components target");
            }

            A::Get => {
                let mut results = Vec::with_capacity(matched.len());

                for (device, entity) in matched {
                    results.push(ComponentResult {
                        device: *device.id(),
                        entity: *entity.index(),
                        value: entity
                            .get(comp_type)
                            .expect("optimizer ensures component exists")
                            .to_igloo_value()
                            .expect("tried to read non-readable value"), // FIXME unwrap
                    });
                }

                R::Components(results)
            }

            A::GetAggregate(op) => {
                let mut values = Vec::with_capacity(matched.len());

                for (_, entity) in matched {
                    values.push(
                        entity.get(comp_type).unwrap().to_igloo_value().unwrap(), // FIXME unwraps
                    );
                }

                R::Aggregate(IglooValue::aggregate(values, op))
            }

            A::Set(value) | A::Put(value) => {
                let comp = Component::from_igloo_value(comp_type, value).unwrap();
                let mut result: Vec<(DeviceID, FloeRef, Vec<usize>)> = Vec::new();

                for (device, entity) in matched {
                    let did = *device.id();
                    let index = *entity.index();

                    if let Some((last_did, _, indices)) = result.last_mut()
                        && *last_did == did
                    {
                        indices.push(index);
                    } else {
                        // FIXME unwrap
                        result.push((did, device.owner_ref().unwrap(), vec![index]));
                    }
                }

                for (did, floe_ref, indexes) in result {
                    let floe = tree.floe_mut(&floe_ref).unwrap();

                    floe.writer
                        .start_transaction(&StartTransaction {
                            device_id: did.take(),
                        })
                        .await
                        .unwrap();

                    for index in indexes {
                        floe.writer
                            .select_entity(&SelectEntity {
                                entity_idx: index as u32,
                            })
                            .await
                            .unwrap();

                        floe.writer.write_component(&comp).await.unwrap();
                        floe.writer.deselect_entity().await.unwrap();
                    }

                    floe.writer.end_transaction().await.unwrap();
                    floe.writer.flush().await.unwrap();
                }

                R::Ok
            }

            A::Apply(op) => {
                let mut result: Vec<(DeviceID, FloeRef, Vec<(usize, Component)>)> = Vec::new();

                for (device, entity) in matched {
                    let did = *device.id();
                    let index = *entity.index();

                    let current = entity.get(comp_type).unwrap().to_igloo_value().unwrap();
                    let new_value = op.eval(&current).unwrap();
                    let new_comp = Component::from_igloo_value(comp_type, new_value).unwrap();

                    if let Some((last_did, _, entries)) = result.last_mut()
                        && *last_did == did
                    {
                        entries.push((index, new_comp));
                    } else {
                        // FIXME unwrap
                        result.push((did, device.owner_ref().unwrap(), vec![(index, new_comp)]));
                    }
                }

                for (did, floe_ref, entries) in result {
                    let floe = tree.floe_mut(&floe_ref).unwrap();

                    floe.writer
                        .start_transaction(&StartTransaction {
                            device_id: did.take(),
                        })
                        .await
                        .unwrap();

                    for (index, comp) in entries {
                        floe.writer
                            .select_entity(&SelectEntity {
                                entity_idx: index as u32,
                            })
                            .await
                            .unwrap();

                        floe.writer.write_component(&comp).await.unwrap();
                        floe.writer.deselect_entity().await.unwrap();
                    }

                    floe.writer.end_transaction().await.unwrap();
                    floe.writer.flush().await.unwrap();
                }

                R::Ok
            }

            _ => unimplemented!(),
        };

        Ok(Ok(QueryResult {
            value,
            tag: query.tag,
        }))
    }
}
