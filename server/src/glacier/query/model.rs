use derive_more::From;
use igloo_interface::{
    Component, ComponentType,
    agg::AggregationOp,
    id::{DeviceID, GroupID},
    query::{QueryFilter, QueryTarget, SetQuery, Snapshot},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, From)]
pub enum Query {
    Set(SetQuery),
    GetOne(GetOneQuery),
    GetAll(GetAllQuery),
    GetAggregate(GetAggregateQuery),
    Watch(WatchQuery),
    Snapshot(SnapshotQuery),
}

#[derive(Debug)]
pub struct GetOneQuery {
    pub comp: ComponentType,
    pub target: QueryTarget,
    pub filter: QueryFilter,
    pub tag: u32,
    pub response_tx: oneshot::Sender<Option<QueryResult>>,
}

#[derive(Debug)]
pub struct GetAllQuery {
    pub comp: ComponentType,
    pub target: QueryTarget,
    pub filter: QueryFilter,
    pub tag: u32,
    pub response_tx: oneshot::Sender<Vec<QueryResult>>,
}

#[derive(Debug)]
pub struct GetAggregateQuery {
    pub comp: ComponentType,
    pub target: QueryTarget,
    pub filter: QueryFilter,
    pub op: AggregationOp,
    pub tag: u32,
    pub response_tx: oneshot::Sender<QueryAggregateResult>,
}

#[derive(Debug)]
pub struct WatchQuery {
    pub comp: ComponentType,
    pub target: QueryTarget,
    pub filter: QueryFilter,
    pub tag: u32,
    pub update_tx: mpsc::Sender<QueryResult>,
}

#[derive(Debug)]
pub struct SnapshotQuery {
    pub response_tx: oneshot::Sender<Snapshot>,
}

#[derive(Debug)]
pub struct QueryResult {
    pub device: DeviceID,
    pub entity: usize,
    pub value: Component,
    pub tag: u32,
}

#[derive(Debug)]
pub struct QueryAggregateResult {
    pub result: Option<Component>,
    pub tag: u32,
}

#[derive(Debug, Clone)]
pub struct AttachedQuery {
    pub filter: QueryFilter,
    pub gid: Option<GroupID>,
    pub tag: u32,
    pub tx: mpsc::Sender<QueryResult>,
}
