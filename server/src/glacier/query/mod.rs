use crate::glacier::{
    query::{
        ctx::QueryContext,
        watch::{Producer, Watch, observer::ObserverSet},
    },
    tree::DeviceTree,
};
use igloo_interface::query::{Query, QueryResult, check::QueryError};
use rustc_hash::FxBuildHasher;
use std::collections::HashSet;
use tokio::sync::mpsc::{self, error::TrySendError};

mod ctx;
mod iter;
mod oneshot;
mod watch;

pub struct QueryEngine {
    pub(self) ctx: QueryContext,
    // TODO rename to clients
    pub(self) producers: Vec<Option<Producer>>,
    // TODO better name, maybe side-effects?
    pub(self) observer_set: ObserverSet,
    // TODO rename this to observer?
    pub(self) watches: Vec<Option<Watch>>,
}

#[derive(Debug)]
pub enum QueryEngineRequest {
    Register(mpsc::Sender<QueryEngineResponse>),
    Unregister {
        producer_id: usize,
    },
    // TODO unregister watcher
    Evaluate {
        producer_id: usize,
        query_id: usize,
        query: Box<Query>,
    },
}

#[derive(Debug, Clone)]
pub enum QueryEngineResponse {
    /// sent on registration
    /// use this for all future requests
    Registered { producer_id: usize },
    Result {
        query_id: usize,
        result: Result<QueryResult, QueryError>,
    },
}

/// internal error
#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Producer {0}'s response channel is full")]
    ProducerChannelFull(usize),
    #[error("Invalid producer {0}")]
    InvalidProducer(usize),
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self {
            ctx: QueryContext::default(),
            observer_set: ObserverSet::new(),
            producers: Vec::with_capacity(50),
            watches: Vec::with_capacity(50),
        }
    }
}

impl QueryEngine {
    pub async fn on_request(
        &mut self,
        tree: &mut DeviceTree,
        request: QueryEngineRequest,
    ) -> Result<(), EngineError> {
        self.ctx.check_gc();

        use QueryEngineRequest::*;
        match request {
            Register(channel) => {
                let producer_id =
                    if let Some(free_slot) = self.producers.iter_mut().position(|o| o.is_none()) {
                        free_slot
                    } else {
                        self.producers.push(None);
                        self.producers.len() - 1
                    };

                match channel.try_send(QueryEngineResponse::Registered { producer_id }) {
                    Ok(_) => {
                        self.producers[producer_id] = Some(Producer {
                            channel,
                            watches: HashSet::with_capacity_and_hasher(10, FxBuildHasher),
                        });
                        Ok(())
                    }
                    Err(TrySendError::Full(_)) => {
                        Err(EngineError::ProducerChannelFull(producer_id))
                    }
                    Err(TrySendError::Closed(_)) => {
                        self.unregister(producer_id);
                        Ok(())
                    }
                }
            }
            Unregister { producer_id } => {
                self.unregister(producer_id);
                Ok(())
            }
            Evaluate {
                producer_id,
                query_id,
                query,
            } => self.eval(tree, producer_id, query_id, *query).await,
        }
    }

    // TODO need a way to unregister 1 watch

    fn unregister(&mut self, producer_id: usize) {
        if let Some(Some(producer)) = self.producers.get_mut(producer_id) {
            let watch_ids: Vec<_> = producer.watches.drain().collect();

            for watch_id in watch_ids {
                // Clean up from ALL observer sets
                self.observer_set.remove_watch_from_all(watch_id);
                self.watches[watch_id] = None;
            }
        }

        // FIXME unsafe
        self.producers[producer_id] = None;
    }

    async fn eval(
        &mut self,
        tree: &mut DeviceTree,
        producer_id: usize,
        query_id: usize,
        mut query: Query,
    ) -> Result<(), EngineError> {
        self.ctx.on_eval_start();

        query.optimize();

        if query.is_observer() {
            return self
                .register_watch(tree, producer_id, query_id, query)
                .await;
        }

        let result = match query {
            Query::Floe(q) => self.eval_floe(tree, q)?,
            Query::Group(q) => self.eval_group(tree, q)?,
            Query::Device(q) => self.eval_device(tree, q)?,
            Query::Entity(q) => self.eval_entity(tree, q)?,
            Query::Component(q) => self.eval_component(tree, q)?,
        };

        self.send_result(producer_id, query_id, result).await
    }

    /// used for benchmarking
    #[allow(dead_code)]
    pub fn test(&mut self, tree: &mut DeviceTree, mut query: Query) -> Result<(), EngineError> {
        query.optimize();

        if query.is_observer() {
            panic!()
        }

        let _ = match query {
            Query::Floe(q) => self.eval_floe(tree, q)?,
            Query::Group(q) => self.eval_group(tree, q)?,
            Query::Device(q) => self.eval_device(tree, q)?,
            Query::Entity(q) => self.eval_entity(tree, q)?,
            Query::Component(q) => self.eval_component(tree, q)?,
        };

        Ok(())
    }

    async fn send_result(
        &mut self,
        producer_id: usize,
        query_id: usize,
        result: Result<QueryResult, QueryError>,
    ) -> Result<(), EngineError> {
        let Some(Some(producer)) = self.producers.get(producer_id) else {
            return Err(EngineError::InvalidProducer(producer_id));
        };

        match producer
            .channel
            .try_send(QueryEngineResponse::Result { query_id, result })
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Full(_)) => Err(EngineError::ProducerChannelFull(producer_id)),
            Err(TrySendError::Closed(_)) => {
                self.unregister(producer_id);
                Ok(())
            }
        }
    }
}
