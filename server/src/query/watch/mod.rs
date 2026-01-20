//! A watcher is a persistent/continous query on the DeviceTree
//!
//! When a client subscribes to a watcher, it will either subscribe
//! to an existing watcher (by checking if the query hash exists)
//! or register a new watcher and subscribe to. Once the # of clients
//! for a watcher reaches 0, it is garbage collected (reference counting).
//!
//! Whenever someone subscribes to a watcher, it's current state is
//! sent out. Then updates are send out as they occur.
//!
//! The backbone of watchers are tree subscriptions (see [subscriber.rs]).
//! Each watcher subscribes to many events on the DeviceTree (ex. on_device_created)
//! and may do some internal bookkeeping and/or dispatch a new update to it's clients.

use igloo_interface::query::{WatchQuery, check::QueryError};

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryEngine,
        watch::{comp::ComponentWatcher, meta::MetadataWatcher, subscriber::TreeSubscribers},
    },
    tree::DeviceTree,
};

mod comp;
pub mod dispatch;
mod meta;
pub mod subscriber;

pub type WatcherID = usize;
pub type WatcherList = Vec<WatcherID>;

pub enum Watcher {
    Component(ComponentWatcher),
    Metadata(MetadataWatcher),
}

impl QueryEngine {
    fn get_watcher_by_query(&mut self, query: &WatchQuery) -> Option<&mut Watcher> {
        let id = self.query_to_watcher.get(query)?;
        self.watchers.get_mut(*id).and_then(|o| o.as_mut())
    }

    /// Subscribe to watcher if exists, otherwise register + subscribe
    pub fn sub_watch(
        &mut self,
        tree: &mut DeviceTree,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
        mut query: WatchQuery,
    ) -> Result<(), IglooError> {
        self.ctx.check_gc();

        query.optimize();

        let watcher = match self.get_watcher_by_query(&query) {
            Some(w) => w,
            // register
            None => match self.reg_watcher(tree, query) {
                Ok(w) => w,
                Err(error) => {
                    cm.send(client_id, IglooResponse::InvalidWatch { query_id, error })?;
                    return Ok(());
                }
            },
        };

        watcher.sub(cm, client_id, query_id)?;

        cm.add_watcher(client_id, watcher.id())
    }

    pub fn unsub_watches(
        &mut self,
        client_id: usize,
        watchers: Vec<WatcherID>,
    ) -> Result<(), IglooError> {
        for watcher_id in watchers {
            if let Some(Some(watcher)) = self.watchers.get_mut(watcher_id) {
                watcher.unsub(client_id);

                // Reference count == 0 -> garbage collect
                if watcher.subs().is_empty() {
                    watcher.cleanup(&mut self.tree_subs);
                    self.query_to_watcher.remove(&watcher.query());
                    self.watchers[watcher_id] = None;
                }
            }
        }

        Ok(())
    }

    fn reg_watcher(
        &mut self,
        tree: &mut DeviceTree,
        query: WatchQuery,
    ) -> Result<&mut Watcher, QueryError> {
        let watcher_id = if let Some(slot) = self.watchers.iter().position(|w| w.is_none()) {
            slot
        } else {
            self.watchers.push(None);
            self.watchers.len() - 1
        };

        let w = match &query {
            WatchQuery::Metadata => {
                let w = MetadataWatcher::register(tree, &mut self.tree_subs, watcher_id);
                Watcher::Metadata(w)
            }
            WatchQuery::Component(query) => {
                let w = ComponentWatcher::register(
                    &mut self.ctx,
                    &mut self.tree_subs,
                    tree,
                    watcher_id,
                    query.clone(),
                )?;
                Watcher::Component(w)
            }
        };

        self.query_to_watcher.insert(query, watcher_id);
        self.watchers[watcher_id] = Some(w);
        Ok(self.watchers[watcher_id].as_mut().unwrap())
    }
}

impl Watcher {
    fn sub(
        &mut self,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
    ) -> Result<(), IglooError> {
        match self {
            Watcher::Component(w) => w.on_sub(cm, client_id, query_id),
            Watcher::Metadata(w) => w.on_sub(cm, client_id, query_id),
        }
    }

    fn unsub(&mut self, client_id: usize) {
        match self {
            Watcher::Component(w) => w.subs.retain(|(cid, _)| *cid != client_id),
            Watcher::Metadata(w) => w.subs.retain(|(cid, _)| *cid != client_id),
        }
    }

    fn id(&self) -> WatcherID {
        match self {
            Watcher::Component(w) => w.id,
            Watcher::Metadata(w) => w.id,
        }
    }

    fn subs(&self) -> &Vec<(usize, usize)> {
        match self {
            Watcher::Component(w) => &w.subs,
            Watcher::Metadata(w) => &w.subs,
        }
    }

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        match self {
            Watcher::Component(w) => w.cleanup(subs),
            Watcher::Metadata(w) => w.cleanup(subs),
        }
    }

    pub fn query(&self) -> WatchQuery {
        match self {
            Watcher::Component(w) => WatchQuery::Component(w.query.clone()),
            Watcher::Metadata(_) => WatchQuery::Metadata,
        }
    }
}
