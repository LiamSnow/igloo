use crate::{
    core::IglooError,
    query::{
        QueryEngine,
        iter::{estimate_device_count, for_each_device},
    },
    tree::DeviceTree,
};
use igloo_interface::query::{
    DeviceAction as A, DeviceQuery, IDFilter, QueryResult as R, check::QueryError,
};
use std::ops::ControlFlow;

impl QueryEngine {
    pub fn eval_device(
        &mut self,
        tree: &DeviceTree,
        query: DeviceQuery,
    ) -> Result<Result<R, QueryError>, IglooError> {
        let result = match query.action {
            A::Count => {
                let mut count = 0usize;
                let limit = query.limit.unwrap_or(usize::MAX);
                let _ = for_each_device(*self.ctx.now(), tree, &query.filter, None, |_| {
                    count += 1;
                    if count >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::Count(count)
            }

            A::GetID => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate = estimate_device_count(tree, &query.filter).min(limit);
                let mut ids = Vec::with_capacity(estimate);

                let _ = for_each_device(*self.ctx.now(), tree, &query.filter, None, |device| {
                    ids.push(*device.id());
                    if ids.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::DeviceId(ids)
            }
            A::Snapshot(include_comps) => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate = estimate_device_count(tree, &query.filter).min(limit);
                let mut snapshots = Vec::with_capacity(estimate);

                let _ = for_each_device(*self.ctx.now(), tree, &query.filter, None, |device| {
                    snapshots.push(device.snapshot(include_comps));
                    if snapshots.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::DeviceSnapshot(snapshots)
            }
            A::IsAttached => match query.filter.id {
                IDFilter::Any => panic!("Must provide ID to check if device is attached"),
                IDFilter::Is(id) => {
                    let attached = tree.device(&id).is_ok();
                    R::DeviceAttached(vec![(id, attached)])
                }
                IDFilter::OneOf(ids) => {
                    let limit = query.limit.unwrap_or(usize::MAX);
                    let mut res = Vec::with_capacity(limit.min(ids.len()));
                    for id in ids {
                        let attached = tree.device(&id).is_ok();
                        res.push((id, attached));
                        if res.len() >= limit {
                            break;
                        }
                    }
                    R::DeviceAttached(res)
                }
            },
            A::WatchAttached | A::WatchName => {
                panic!("Observe should have been dispatched differently")
            }
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}
