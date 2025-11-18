use crate::glacier::{
    query::optimizer::QueryOptimizer,
    tree::{DeviceTree, mutation::TreeMutation},
};
use globset::{GlobBuilder, GlobMatcher};
use igloo_interface::query::{Query, QueryResult, QueryTarget as QT};
use rustc_hash::FxHashMap;
use std::time::Instant;
use tokio::sync::mpsc::{self, error::SendError};

mod collector;
mod filter;
mod optimizer;
mod targets;

pub struct QueryEngine {
    // TODO FIXME need a garbage collection system
    globs: FxHashMap<String, GlobMatcher>,
    query_time: Instant,
}

// TODO eventually fix this. We need to tag errors
pub type QueryResponse = Result<QueryResult, QueryError>;

pub type QueryEngineTx = mpsc::Sender<(Query, mpsc::Sender<QueryResponse>)>;
pub type QueryEngineRx = mpsc::Receiver<(Query, mpsc::Sender<QueryResponse>)>;

/// issue with query itself
#[derive(thiserror::Error, Debug)]
pub enum QueryError {}

/// internal error
#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Send error: {0}")]
    Send(#[from] SendError<QueryResponse>),
}

impl QueryEngine {
    pub fn new() -> Self {
        Self {
            globs: FxHashMap::default(),
            query_time: Instant::now(),
        }
    }

    pub async fn on_tree_mutation(&mut self, _mutation: TreeMutation) -> Result<(), EngineError> {
        // TODO
        Ok(())
    }

    pub async fn evaluate(
        &mut self,
        tree: &mut DeviceTree,
        mut query: Query,
        tx: mpsc::Sender<QueryResponse>,
    ) -> Result<(), EngineError> {
        self.query_time = Instant::now();

        query.optimize();

        let res = match query.target {
            // These return Result<QueryResponse, EngineError>
            // which is equal to Result<Result<QueryResult, QueryError>, EngineError>
            // This is very intentional, QueryErrors get sent to user whereas EngineErrors
            // are internal errors that will be logged from higher up
            QT::Floes => self.evaluate_floes(tree, query).await?,
            QT::Groups => self.evaluate_groups(tree, query).await?,
            QT::Devices => self.evaluate_devices(tree, query).await?,
            QT::Entities => self.evaluate_entities(tree, query).await?,
            QT::Components(t) => self.evaluate_comps(tree, query, t).await?,
        };

        tx.send(res).await?;

        Ok(())
    }

    // pub async fn evaluate(
    //     &mut self,
    //     tree: &mut DeviceTree,
    //     mut query: Query,
    //     tx: mpsc::Sender<QueryResponse>,
    // ) -> Result<(), EngineError> {
    //     self.query_time = Instant::now();

    //     query.optimize();

    //     const ITERATIONS: usize = 2_000_000;

    //     let mut res = None;

    //     let start = Instant::now();
    //     self.benchmark_queries(tree, &query, ITERATIONS, &mut res)
    //         .await?;
    //     let elapsed = start.elapsed();
    //     println!(
    //         "DONE. Total time = {:?}. Iteration Time = {:?}",
    //         elapsed,
    //         elapsed / (ITERATIONS as u32)
    //     );

    //     let start = Instant::now();
    //     self.benchmark_queries(tree, &query, ITERATIONS, &mut res)
    //         .await?;
    //     let elapsed = start.elapsed();
    //     println!(
    //         "DONE. Total time = {:?}. Iteration Time = {:?}",
    //         elapsed,
    //         elapsed / (ITERATIONS as u32)
    //     );

    //     tx.send(res.unwrap()).await?;

    //     Ok(())
    // }

    // async fn benchmark_queries(
    //     &mut self,
    //     tree: &mut DeviceTree,
    //     query: &Query,
    //     iterations: usize,
    //     res: &mut Option<QueryResponse>,
    // ) -> Result<(), EngineError> {
    //     for _ in 0..iterations {
    //         *res = Some(match query.target {
    //             QT::Floes => self.evaluate_floes(tree, query.clone()).await?,
    //             QT::Groups => self.evaluate_groups(tree, query.clone()).await?,
    //             QT::Devices => self.evaluate_devices(tree, query.clone()).await?,
    //             QT::Entities => self.evaluate_entities(tree, query.clone()).await?,
    //             QT::Components(t) => self.evaluate_comps(tree, query.clone(), t).await?,
    //         });
    //     }
    //     Ok(())
    // }

    fn glob(&mut self, pattern: &str) -> &GlobMatcher {
        if !self.globs.contains_key(pattern) {
            let glob = GlobBuilder::new(pattern).build().unwrap().compile_matcher(); // FIXME unwrap
            self.globs.insert(pattern.to_string(), glob);
        }
        self.globs.get(pattern).unwrap()
    }
}
