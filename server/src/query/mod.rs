use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        ctx::QueryContext,
        observer::{Observer, ObserverID, subscriber::TreeSubscribers},
    },
    tree::DeviceTree,
};
use igloo_interface::query::Query;

mod ctx;
mod iter;
pub mod observer;
mod oneshot;

pub struct QueryEngine {
    pub(self) ctx: QueryContext,
    pub(self) subscribers: TreeSubscribers,
    pub(self) observers: Vec<Option<Observer>>,
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self {
            ctx: QueryContext::default(),
            subscribers: TreeSubscribers::default(),
            observers: Vec::with_capacity(50),
        }
    }
}

impl QueryEngine {
    pub fn eval(
        &mut self,
        tree: &mut DeviceTree,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
        mut query: Query,
    ) -> Result<(), IglooError> {
        self.ctx.check_gc();
        self.ctx.on_eval_start();

        query.optimize();

        if query.is_observer() {
            return match self.register_observer(tree, cm, client_id, query_id, query)? {
                Err(e) => cm.send(
                    client_id,
                    IglooResponse::QueryResult {
                        query_id,
                        result: Err(e),
                    },
                ),
                // observer registered successfully, no response
                // will be given until an event occurs
                Ok(()) => Ok(()),
            };
        }

        let result = match query {
            Query::Extension(q) => self.eval_extension(tree, q)?,
            Query::Group(q) => self.eval_group(tree, q)?,
            Query::Device(q) => self.eval_device(tree, q)?,
            Query::Entity(q) => self.eval_entity(tree, q)?,
            Query::Component(q) => self.eval_component(cm, tree, q)?,
        };

        cm.send(client_id, IglooResponse::QueryResult { query_id, result })
    }

    pub fn drop_observers(&mut self, observers: Vec<ObserverID>) {
        for observer in observers {
            self.subscribers.unsubscribe(observer);
            self.observers[observer] = None;
        }
    }
}
