use igloo_interface::query::Query;
use tokio::sync::mpsc;

use crate::glacier::tree::{DeviceTree, mutation::TreeMutation};

pub struct QueryEngine;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {}

pub type QueryEngineTx = mpsc::Sender<(Query, mpsc::Sender<QueryResponse>)>;
pub type QueryEngineRx = mpsc::Receiver<(Query, mpsc::Sender<QueryResponse>)>;

pub enum QueryResponse {}

impl QueryEngine {
    pub fn new() -> Self {
        // TODO might want to add async or result return?
        Self {}
    }

    pub async fn on_tree_mutation(&mut self, _mutation: TreeMutation) -> Result<(), QueryError> {
        // TODO
        Ok(())
    }

    pub async fn execute(
        &mut self,
        _tree: &mut DeviceTree,
        _query: Query,
        _tx: mpsc::Sender<QueryResponse>,
    ) -> Result<(), QueryError> {
        todo!()
    }
}
