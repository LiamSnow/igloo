use crate::glacier::{
    query::{EngineError, QueryEngine, QueryResponse, collector::QueryCollector},
    tree::DeviceTree,
};
use igloo_interface::query::{Query, QueryAction as A, QueryResult, QueryResultValue as R};

impl QueryEngine {
    pub async fn evaluate_floes(
        &mut self,
        tree: &DeviceTree,
        mut query: Query,
    ) -> Result<QueryResponse, EngineError> {
        let action = query.action.clone();
        let matched = query.collect_matching_floes(self, tree);

        let value = match action {
            A::Count => R::Count(matched.len()),

            A::GetIds => {
                let mut ids = Vec::with_capacity(matched.len());
                for floe in matched {
                    ids.push(floe.id().clone());
                }
                R::FloeIds(ids)
            }

            A::Get => {
                let mut snapshots = Vec::with_capacity(matched.len());
                for floe in &matched {
                    snapshots.push(floe.snapshot());
                }
                R::Floes(snapshots)
            }

            _ => unimplemented!(),
        };

        Ok(Ok(QueryResult {
            value,
            tag: query.tag,
        }))
    }
}
