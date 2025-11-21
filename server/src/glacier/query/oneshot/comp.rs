use crate::glacier::{
    query::{
        EngineError, QueryEngine,
        iter::{estimate_entity_count, for_each_entity},
    },
    tree::DeviceTree,
};
use igloo_interface::{
    Aggregator,
    query::{ComponentAction as A, ComponentQuery, QueryResult as R, check::QueryError},
};
use std::ops::ControlFlow;

impl QueryEngine {
    pub fn eval_component(
        &mut self,
        tree: &mut DeviceTree,
        query: ComponentQuery,
    ) -> Result<Result<R, QueryError>, EngineError> {
        let limit = query.limit.unwrap_or(usize::MAX);

        let result = match query.action {
            A::Count => {
                let mut count = 0usize;

                let _ = for_each_entity(
                    &mut self.ctx,
                    tree,
                    &query.device_filter,
                    &query.entity_filter,
                    |_, _| {
                        count += 1;
                        if count >= limit {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                );

                R::Count(count)
            }

            A::GetValue => match query.post_op {
                Some(op) => {
                    let Some(mut agg) = Aggregator::new(query.component, op) else {
                        return Ok(Err(QueryError::InvalidAggregation(query.component, op)));
                    };

                    let _ = for_each_entity(
                        &mut self.ctx,
                        tree,
                        &query.device_filter,
                        &query.entity_filter,
                        |_, entity| {
                            if let Some(comp) = entity.get(query.component) {
                                agg.push(comp)?;
                            }
                            ControlFlow::Continue(())
                        },
                    );

                    R::ComponentValue(match agg.finish() {
                        Some(v) => vec![v],
                        None => vec![],
                    })
                }

                None if query.include_parents => {
                    let estimate =
                        estimate_entity_count(tree, &query.device_filter, &query.entity_filter)
                            .min(limit);
                    let mut res = Vec::with_capacity(estimate);
                    let mut error = None;

                    let _ = for_each_entity(
                        &mut self.ctx,
                        tree,
                        &query.device_filter,
                        &query.entity_filter,
                        |device, entity| {
                            let Some(comp_val) = entity.get(query.component) else {
                                return ControlFlow::Continue(());
                            };
                            let Some(iv) = comp_val.to_igloo_value() else {
                                error = Some(QueryError::ComponentNoValue(query.component));
                                return ControlFlow::Break(());
                            };
                            res.push((*device.id(), entity.name().to_string(), iv));
                            if res.len() >= limit {
                                ControlFlow::Break(())
                            } else {
                                ControlFlow::Continue(())
                            }
                        },
                    );

                    if let Some(e) = error {
                        return Ok(Err(e));
                    }
                    R::ComponentValueWithParents(res)
                }

                _ => {
                    let estimate =
                        estimate_entity_count(tree, &query.device_filter, &query.entity_filter)
                            .min(limit);
                    let mut res = Vec::with_capacity(estimate);
                    let mut error = None;

                    let _ = for_each_entity(
                        &mut self.ctx,
                        tree,
                        &query.device_filter,
                        &query.entity_filter,
                        |_, entity| {
                            let Some(comp_val) = entity.get(query.component) else {
                                return ControlFlow::Continue(());
                            };
                            let Some(iv) = comp_val.to_igloo_value() else {
                                error = Some(QueryError::ComponentNoValue(query.component));
                                return ControlFlow::Break(());
                            };
                            res.push(iv);
                            if res.len() >= limit {
                                ControlFlow::Break(())
                            } else {
                                ControlFlow::Continue(())
                            }
                        },
                    );

                    if let Some(e) = error {
                        return Ok(Err(e));
                    }
                    R::ComponentValue(res)
                }
            },

            A::Set(_) | A::Put(_) | A::Apply(_) => unimplemented!(),
            A::ObserveValue => {
                panic!("Observe should have been dispatched differently")
            }
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}

// A::Set(value) | A::Put(value) => {
//     let comp = Component::from_igloo_value(query.component, value).unwrap();
//     let mut result: Vec<(DeviceID, FloeRef, Vec<usize>)> =
//         Vec::with_capacity(matched.len());

//     for (device, entity) in matched {
//         let did = *device.id();
//         let index = *entity.index();

//         if let Some((last_did, _, indices)) = result.last_mut()
//             && *last_did == did
//         {
//             indices.push(index);
//         } else {
//             // FIXME unwrap
//             result.push((did, device.owner_ref().unwrap(), vec![index]));
//         }
//     }

//     // TODO probably should extract to DeviceTree method with timeouts and stuff
//     // for (did, floe_ref, indexes) in result {
//     //     let floe = tree.floe_mut(&floe_ref).unwrap();

//     //     floe.writer
//     //         .start_transaction(&StartTransaction {
//     //             device_id: did.take(),
//     //         })
//     //         .await
//     //         .unwrap();

//     //     for index in indexes {
//     //         floe.writer
//     //             .select_entity(&SelectEntity {
//     //                 entity_idx: index as u32,
//     //             })
//     //             .await
//     //             .unwrap();

//     //         floe.writer.write_component(&comp).await.unwrap();
//     //         floe.writer.deselect_entity().await.unwrap();
//     //     }

//     //     floe.writer.end_transaction().await.unwrap();
//     //     floe.writer.flush().await.unwrap();
//     // }

//     R::Ok
// }

// A::Apply(op) => {
//     let mut result: Vec<(DeviceID, FloeRef, Vec<(usize, Component)>)> =
//         Vec::with_capacity(matched.len());

//     for (device, entity) in matched {
//         let did = *device.id();
//         let index = *entity.index();

//         let current = entity
//             .get(query.component)
//             .unwrap()
//             .to_igloo_value()
//             .unwrap();
//         let new_value = op.eval(&current).unwrap();
//         let new_comp = Component::from_igloo_value(query.component, new_value).unwrap();

//         if let Some((last_did, _, entries)) = result.last_mut()
//             && *last_did == did
//         {
//             entries.push((index, new_comp));
//         } else {
//             // FIXME unwrap
//             result.push((did, device.owner_ref().unwrap(), vec![(index, new_comp)]));
//         }
//     }

//     // for (did, floe_ref, entries) in result {
//     //     let floe = tree.floe_mut(&floe_ref).unwrap();

//     //     floe.writer
//     //         .start_transaction(&StartTransaction {
//     //             device_id: did.take(),
//     //         })
//     //         .await
//     //         .unwrap();

//     //     for (index, comp) in entries {
//     //         floe.writer
//     //             .select_entity(&SelectEntity {
//     //                 entity_idx: index as u32,
//     //             })
//     //             .await
//     //             .unwrap();

//     //         floe.writer.write_component(&comp).await.unwrap();
//     //         floe.writer.deselect_entity().await.unwrap();
//     //     }

//     //     floe.writer.end_transaction().await.unwrap();
//     //     floe.writer.flush().await.unwrap();
//     // }

//     R::Ok
// }
