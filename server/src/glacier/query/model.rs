use derive_more::From;
use igloo_interface::{
    Component, ComponentType, DeviceID, GroupID, QueryFilter, QueryTarget, SetQuery, Snapshot,
};
use rustc_hash::FxHashMap;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, From)]
pub enum Query {
    Set(SetQuery),
    GetOne(GetOneQuery),
    GetAll(GetAllQuery),
    GetAvg(GetAvgQuery),
    WatchAll(WatchAllQuery),
    Snapshot(SnapshotQuery),
}

#[derive(Debug)]
pub struct GetOneQuery {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub response_tx: oneshot::Sender<Option<OneQueryResult>>,
    pub comp: ComponentType,
}

#[derive(Debug)]
pub struct GetAllQuery {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub response_tx: oneshot::Sender<GetAllQueryResult>,
    pub comp: ComponentType,
}

#[derive(Debug)]
pub struct GetAvgQuery {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub response_tx: oneshot::Sender<Option<Component>>,
    pub comp: ComponentType,
}

#[derive(Debug)]
pub struct WatchAllQuery {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub update_tx: mpsc::Sender<PrefixedOneQueryResult>,
    pub comp: ComponentType,
    pub prefix: u32,
}

#[derive(Debug)]
pub struct SnapshotQuery {
    pub response_tx: oneshot::Sender<Snapshot>,
}

// TODO this is horrible
pub type PrefixedOneQueryResult = (u32, DeviceID, usize, Component);
pub type OneQueryResult = (DeviceID, usize, Component);
pub type GetAllQueryResult = FxHashMap<DeviceID, FxHashMap<usize, Component>>;

#[derive(Debug, Clone)]
pub struct WatchQuery {
    pub prefix: u32,
    pub filter: QueryFilter,
    pub tx: mpsc::Sender<PrefixedOneQueryResult>,
    pub gid: Option<GroupID>,
}
