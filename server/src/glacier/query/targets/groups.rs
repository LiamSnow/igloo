use crate::glacier::{
    query::{EngineError, QueryEngine, QueryResponse, collector::QueryCollector},
    tree::DeviceTree,
};
use igloo_interface::query::{Query, QueryAction as A, QueryResult, QueryResultValue as R};

impl QueryEngine {
    pub async fn evaluate_groups(
        &mut self,
        tree: &DeviceTree,
        mut query: Query,
    ) -> Result<QueryResponse, EngineError> {
        let action = query.action.clone();
        let matched = query.collect_matching_groups(self, tree);

        let value = match action {
            A::Count => R::Count(matched.len()),

            A::GetIds => {
                let mut ids = Vec::with_capacity(matched.len());
                for group in matched {
                    ids.push(*group.id());
                }
                R::GroupIds(ids)
            }

            A::Get => {
                let mut snapshots = Vec::with_capacity(matched.len());
                for group in matched {
                    snapshots.push(group.snapshot());
                }
                R::Groups(snapshots)
            }

            _ => unimplemented!(),
        };

        Ok(Ok(QueryResult {
            value,
            tag: query.tag,
        }))
    }
}
