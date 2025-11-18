use crate::glacier::{
    query::{EngineError, QueryEngine, QueryResponse, collector::QueryCollector},
    tree::DeviceTree,
};
use igloo_interface::query::{Query, QueryAction as A, QueryResult, QueryResultValue as R};

impl QueryEngine {
    pub async fn evaluate_entities(
        &mut self,
        tree: &DeviceTree,
        mut query: Query,
    ) -> Result<QueryResponse, EngineError> {
        let action = query.action.clone();
        let matched = query.collect_matching_entities(self, tree);

        let value = match action {
            A::Count => R::Count(matched.len()),

            A::GetIds => {
                let mut ids = Vec::with_capacity(matched.len());
                for (_, entity) in matched {
                    ids.push((entity.name().to_string(), *entity.index()));
                }
                R::EntityIds(ids)
            }

            A::Get => {
                let mut snapshots = Vec::with_capacity(matched.len());
                for (device, entity) in matched {
                    snapshots.push(entity.snapshot(*device.id()));
                }
                R::Entities(snapshots)
            }

            _ => unimplemented!(),
        };

        Ok(Ok(QueryResult {
            value,
            tag: query.tag,
        }))
    }
}
