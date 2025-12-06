use crate::{
    core::IglooError,
    query::{
        QueryEngine,
        iter::{estimate_ext_count, for_each_ext},
    },
    tree::DeviceTree,
};
use igloo_interface::query::{
    ExtensionAction as A, ExtensionQuery, IDFilter, QueryResult as R, check::QueryError,
};
use std::ops::ControlFlow;

impl QueryEngine {
    pub fn eval_extension(
        &mut self,
        tree: &DeviceTree,
        query: ExtensionQuery,
    ) -> Result<Result<R, QueryError>, IglooError> {
        let result = match query.action {
            A::Count => {
                let mut count = 0usize;
                let limit = query.limit.unwrap_or(usize::MAX);

                let _ = for_each_ext(tree, &query.id, |_| {
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
                let estimate = estimate_ext_count(tree, &query.id).min(limit);
                let mut ids = Vec::with_capacity(estimate);

                let _ = for_each_ext(tree, &query.id, |ext| {
                    ids.push(ext.id().clone());
                    if ids.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::ExtensionId(ids)
            }

            A::Snapshot => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate = estimate_ext_count(tree, &query.id).min(limit);
                let mut snapshots = Vec::with_capacity(estimate);

                let _ = for_each_ext(tree, &query.id, |ext| {
                    snapshots.push(ext.snapshot());
                    if snapshots.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::ExtensionSnapshot(snapshots)
            }

            A::IsAttached => match query.id {
                IDFilter::Any => panic!("Must provide ID to check if ext is attached"),
                IDFilter::Is(id) => {
                    let attached = tree.ext_index(&id).is_ok();
                    R::ExtensionAttached(vec![(id, attached)])
                }
                IDFilter::OneOf(ids) => {
                    let limit = query.limit.unwrap_or(usize::MAX);
                    let mut res = Vec::with_capacity(limit.min(ids.len()));
                    for id in ids {
                        let attached = tree.ext_index(&id).is_ok();
                        res.push((id, attached));
                        if res.len() >= limit {
                            break;
                        }
                    }
                    R::ExtensionAttached(res)
                }
            },

            A::ObserveAttached => panic!("Observe should have been dispatched differently"),
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}
