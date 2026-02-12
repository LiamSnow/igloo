use igloo_interface::query::OneShotQuery;

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::QueryEngine,
    tree::DeviceTree,
};

mod comp;
mod device;
mod entity;
mod ext;
mod group;

impl QueryEngine {
    pub fn eval_oneshot(
        &mut self,
        tree: &mut DeviceTree,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
        mut query: OneShotQuery,
    ) -> Result<(), IglooError> {
        self.ctx.check_gc();
        self.ctx.on_eval_start();

        query.optimize();

        use OneShotQuery::*;
        let result = match query {
            Extension(q) => self.eval_extension(tree, q)?,
            Group(q) => self.eval_group(tree, q)?,
            Device(q) => self.eval_device(tree, q)?,
            Entity(q) => self.eval_entity(tree, q)?,
            Component(q) => self.eval_component(cm, tree, q)?,
        };

        cm.send(client_id, IglooResponse::EvalResult { query_id, result })
    }
}
