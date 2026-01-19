use crate::{
    core::{ClientManager, IglooError},
    query::{
        QueryEngine,
        iter::{estimate_entity_count, for_each_entity},
    },
    tree::DeviceTree,
};
use igloo_interface::{
    Aggregator, Component,
    id::GenerationalID,
    ipc::IglooMessage,
    query::{ComponentAction as A, ComponentQuery, QueryResult as R, check::QueryError},
};
use rustc_hash::FxBuildHasher;
use std::{collections::HashSet, ops::ControlFlow};

impl QueryEngine {
    pub fn eval_component(
        &mut self,
        cm: &mut ClientManager,
        tree: &mut DeviceTree,
        query: ComponentQuery,
    ) -> Result<Result<R, QueryError>, IglooError> {
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

                    R::Aggregate(agg.finish())
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
                            res.push((*device.id(), entity.id().clone(), iv));
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

            A::Set(value) | A::Put(value) => {
                let comp = Component::from_igloo_value(query.component, value).unwrap(); // FIXME unwrap
                let msic = query.component as u16;
                let mut scratch = Vec::with_capacity(64);
                let mut exts_to_kill = HashSet::with_capacity_and_hasher(2, FxBuildHasher);
                let mut msg = IglooMessage::WriteComponents {
                    device: u64::MAX,
                    entity: usize::MAX,
                    comps: vec![comp],
                };
                let mut count = 0;

                let _ = for_each_entity(
                    &mut self.ctx,
                    tree,
                    &query.device_filter,
                    &query.entity_filter,
                    |device, entity| {
                        let Some(xindex) = device.owner_ref() else {
                            return ControlFlow::Continue(());
                        };

                        if exts_to_kill.contains(&xindex) {
                            return ControlFlow::Continue(());
                        }

                        let Ok(ext) = tree.ext(&xindex) else {
                            return ControlFlow::Continue(());
                        };

                        if msic > ext.msic {
                            // extension doesn't support this component so dont send it
                            return ControlFlow::Continue(());
                        }

                        if let IglooMessage::WriteComponents {
                            device: d,
                            entity: e,
                            ..
                        } = &mut msg
                        {
                            *d = device.id().take();
                            *e = entity.index().0;
                        }

                        let res = ext.writer.try_write_immut(&msg, &mut scratch);
                        if res.is_err() {
                            // TODO print error
                            exts_to_kill.insert(xindex);
                        }

                        count += 1;

                        if count >= limit {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                );

                for xindex in exts_to_kill {
                    println!("{xindex}'s unix socket is full. Killing..");
                    // TODO reboot instead of kill
                    tree.detach_ext(cm, self, xindex)?;
                }

                R::Count(count)
            }

            A::Apply(op) => {
                let mut scratch = Vec::with_capacity(64);
                let mut exts_to_kill = HashSet::with_capacity_and_hasher(2, FxBuildHasher);
                let mut count = 0;

                let _ = for_each_entity(
                    &mut self.ctx,
                    tree,
                    &query.device_filter,
                    &query.entity_filter,
                    |device, entity| {
                        let Some(comp_val) = entity.get(query.component) else {
                            return ControlFlow::Continue(());
                        };
                        let Some(cur_value) = comp_val.to_igloo_value() else {
                            // TODO return error
                            return ControlFlow::Break(());
                        };

                        let Some(new_value) = op.eval(&cur_value) else {
                            // TODO return error
                            return ControlFlow::Break(());
                        };

                        let Some(comp) = Component::from_igloo_value(query.component, new_value)
                        else {
                            // TODO return error
                            return ControlFlow::Break(());
                        };

                        let Some(xindex) = device.owner_ref() else {
                            return ControlFlow::Continue(());
                        };

                        if exts_to_kill.contains(&xindex) {
                            return ControlFlow::Continue(());
                        }

                        let Ok(ext) = tree.ext(&xindex) else {
                            return ControlFlow::Continue(());
                        };

                        // no need to check MSIC since we are just modifying
                        // components they put

                        let msg = IglooMessage::WriteComponents {
                            device: device.id().take(),
                            entity: entity.index().0,
                            comps: vec![comp],
                        };

                        let res = ext.writer.try_write_immut(&msg, &mut scratch);
                        if res.is_err() {
                            exts_to_kill.insert(xindex);
                        }

                        count += 1;

                        if count >= limit {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                );

                for xindicies in exts_to_kill {
                    println!("{xindicies}'s unix socket is full. Killing..");
                    // TODO reboot instead of kill
                    tree.detach_ext(cm, self, xindicies)?;
                }

                R::Count(count)
            }

            A::WatchValue => {
                panic!("Observe should have been dispatched differently")
            }
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}
