use crate::query::{
    ctx::QueryContext,
    watch::{Watcher, subscriber::TreeSubscribers},
};
use igloo_interface::query::WatchQuery;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::collections::HashMap;

mod ctx;
mod iter;
mod oneshot;
pub mod watch;

pub struct QueryEngine {
    pub(self) ctx: QueryContext,
    pub(self) tree_subs: TreeSubscribers,
    pub(self) watchers: Vec<Option<Watcher>>,
    pub(self) query_to_watcher: FxHashMap<WatchQuery, usize>,
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self {
            ctx: QueryContext::default(),
            tree_subs: TreeSubscribers::default(),
            watchers: Vec::with_capacity(50),
            query_to_watcher: HashMap::with_capacity_and_hasher(50, FxBuildHasher),
        }
    }
}
