use crate::{
    core::IglooError,
    query::{
        QueryEngine,
        iter::{estimate_group_count, for_each_group},
    },
    tree::DeviceTree,
};
use igloo_interface::query::{GroupAction as A, GroupQuery, QueryResult as R, check::QueryError};
use std::ops::ControlFlow;

impl QueryEngine {
    pub fn eval_group(
        &mut self,
        tree: &DeviceTree,
        query: GroupQuery,
    ) -> Result<Result<R, QueryError>, IglooError> {
        let result = match query.action {
            A::Count => {
                let mut count = 0usize;
                let limit = query.limit.unwrap_or(usize::MAX);

                let _ = for_each_group(tree, &query.id, |_| {
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
                let estimate = estimate_group_count(tree, &query.id).min(limit);
                let mut ids = Vec::with_capacity(estimate);

                let _ = for_each_group(tree, &query.id, |group| {
                    ids.push(*group.id());
                    if ids.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::GroupId(ids)
            }

            A::Snapshot => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate = estimate_group_count(tree, &query.id).min(limit);
                let mut snapshots = Vec::with_capacity(estimate);

                let _ = for_each_group(tree, &query.id, |group| {
                    snapshots.push(group.snapshot());
                    if snapshots.len() >= limit {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
                R::GroupSnapshot(snapshots)
            }

            A::ObserveRename | A::ObserveMembershipChanged => {
                panic!("Observe should have been dispatched differently")
            }
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}
