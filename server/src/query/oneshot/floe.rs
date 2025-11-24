use crate::{
    core::IglooError,
    query::{
        QueryEngine,
        iter::{estimate_floe_count, for_each_floe},
    },
    tree::DeviceTree,
};
use igloo_interface::query::{
    FloeAction as A, FloeQuery, IDFilter, QueryResult as R, check::QueryError,
};
use std::ops::ControlFlow;

impl QueryEngine {
    pub fn eval_floe(
        &mut self,
        tree: &DeviceTree,
        query: FloeQuery,
    ) -> Result<Result<R, QueryError>, IglooError> {
        let result = match query.action {
            A::Count => {
                let mut count = 0usize;
                let limit = query.limit.unwrap_or(usize::MAX);

                let _ = for_each_floe(tree, &query.id, |_| {
                    count += 1;
                    if count >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::Count(count)
            }

            A::GetId => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate = estimate_floe_count(tree, &query.id).min(limit);
                let mut ids = Vec::with_capacity(estimate);

                let _ = for_each_floe(tree, &query.id, |floe| {
                    ids.push(floe.id().clone());
                    if ids.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::FloeId(ids)
            }

            A::Snapshot => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate = estimate_floe_count(tree, &query.id).min(limit);
                let mut snapshots = Vec::with_capacity(estimate);

                let _ = for_each_floe(tree, &query.id, |floe| {
                    snapshots.push(floe.snapshot());
                    if snapshots.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::FloeSnapshot(snapshots)
            }

            A::IsAttached => match query.id {
                IDFilter::Any => panic!("Must provide ID to check if floe is attached"),
                IDFilter::Id(id) => {
                    let attached = tree.floe_ref(&id).is_ok();
                    R::FloeAttached(vec![(id, attached)])
                }
                IDFilter::IdIn(ids) => {
                    let limit = query.limit.unwrap_or(usize::MAX);
                    let mut res = Vec::with_capacity(limit.min(ids.len()));
                    for id in ids {
                        let attached = tree.floe_ref(&id).is_ok();
                        res.push((id, attached));
                        if res.len() >= limit {
                            break;
                        }
                    }
                    R::FloeAttached(res)
                }
            },

            A::ObserveAttached => panic!("Observe should have been dispatched differently"),
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}
