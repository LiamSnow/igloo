use crate::{
    core::{ClientManager, IglooError},
    ext::ExtensionRequest,
    query::{
        QueryEngine,
        iter::{estimate_entity_count, for_each_entity},
    },
    tree::DeviceTree,
};
use igloo_interface::{
    Aggregator, Component,
    ipc::IglooToExtension,
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
                let mut exts_to_kill = HashSet::with_capacity_and_hasher(2, FxBuildHasher);
                let mut exts_to_flush = HashSet::with_capacity_and_hasher(2, FxBuildHasher);
                let mut msg = ExtensionRequest::Msg(IglooToExtension::WriteComponents {
                    device: u64::MAX,
                    entity: usize::MAX,
                    comps: vec![comp],
                });
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

                        if let ExtensionRequest::Msg(IglooToExtension::WriteComponents {
                            device: d,
                            entity: e,
                            ..
                        }) = &mut msg
                        {
                            *d = *device.id().inner();
                            *e = entity.index().0;
                        }

                        match ext.channel.try_send(msg.clone()) {
                            Ok(_) => exts_to_flush.insert(xindex),
                            // TODO print error
                            Err(_) => exts_to_kill.insert(xindex),
                        };

                        count += 1;

                        if count >= limit {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                );

                for xindex in exts_to_flush {
                    if let Ok(ext) = tree.ext(&xindex) {
                        _ = ext.channel.try_send(ExtensionRequest::Flush);
                    }
                }

                for xindex in exts_to_kill {
                    tree.detach_ext(cm, self, xindex, true)?;
                }

                R::Count(count)
            }

            A::Apply(op) => {
                let mut exts_to_kill = HashSet::with_capacity_and_hasher(2, FxBuildHasher);
                let mut exts_to_flush = HashSet::with_capacity_and_hasher(2, FxBuildHasher);
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

                        let msg = ExtensionRequest::Msg(IglooToExtension::WriteComponents {
                            device: *device.id().inner(),
                            entity: entity.index().0,
                            comps: vec![comp],
                        });

                        match ext.channel.try_send(msg.clone()) {
                            Ok(_) => exts_to_flush.insert(xindex),
                            // TODO print error
                            Err(_) => exts_to_kill.insert(xindex),
                        };

                        count += 1;

                        if count >= limit {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                );

                for xindex in exts_to_flush {
                    if let Ok(ext) = tree.ext(&xindex) {
                        _ = ext.channel.try_send(ExtensionRequest::Flush);
                    }
                }

                for xindex in exts_to_kill {
                    tree.detach_ext(cm, self, xindex, true)?;
                }

                R::Count(count)
            }
        };

        Ok(Ok(result))
    }
}
