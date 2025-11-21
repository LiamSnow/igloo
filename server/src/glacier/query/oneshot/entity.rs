use crate::glacier::{
    query::{
        EngineError, QueryEngine,
        iter::{estimate_entity_count, for_each_entity},
    },
    tree::DeviceTree,
};
use igloo_interface::query::{EntityAction as A, EntityQuery, QueryResult as R, check::QueryError};
use std::ops::ControlFlow;

impl QueryEngine {
    pub fn eval_entity(
        &mut self,
        tree: &DeviceTree,
        query: EntityQuery,
    ) -> Result<Result<R, QueryError>, EngineError> {
        let result = match query.action {
            A::Count => {
                let mut count = 0usize;
                let limit = query.limit.unwrap_or(usize::MAX);

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

            A::Snapshot => {
                let limit = query.limit.unwrap_or(usize::MAX);
                let estimate =
                    estimate_entity_count(tree, &query.device_filter, &query.entity_filter)
                        .min(limit);
                let mut snapshots = Vec::with_capacity(estimate);

                let _ = for_each_entity(
                    &mut self.ctx,
                    tree,
                    &query.device_filter,
                    &query.entity_filter,
                    |device, entity| {
                        snapshots.push(entity.snapshot(*device.id()));
                        if snapshots.len() >= limit {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                );
                R::EntitySnapshot(snapshots)
            }

            A::ObserveComponentPut => {
                panic!("Observe should have been dispatched differently")
            }
            A::Inherit => return Ok(Err(QueryError::Inherit)),
        };

        Ok(Ok(result))
    }
}
