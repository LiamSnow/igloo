use crate::{
    ComponentType as CT, IglooType,
    query::{
        ComponentAction as C, DeviceAction as D, EntityAction as E, ExtensionAction as X,
        GroupAction as G, OneShotQuery, QueryResultType as R, WatchQuery, WatchUpdateType,
    },
    types::agg::AggregationOp,
};
use thiserror::Error;

// TODO validate globs

#[derive(Error, Debug, Clone)]
pub enum QueryError {
    #[error(
        "Component type '{0:?}' has no value. Actions like 'Get', 'Set', and 'Put' can only be done on components like sensors, switches, or dimmers can be queried with 'Get'."
    )]
    ComponentNoValue(CT),

    #[error("Cannot apply aggregation '{1}' to component '{0:?}'.")]
    InvalidAggregation(CT, AggregationOp),

    #[error("Cannot apply operation to type {0}.")]
    OperationNotApplicable(IglooType),

    #[error("Cannot set value of type {0} on component with value type {1}.")]
    ValueTypeMismatch(IglooType, IglooType),

    #[error("Aggregation can only be used with GetValue or WatchValue actions.")]
    AggregationRequiresValueAction,

    #[error(
        "Cannot use aggregation with include_parents. Aggregation combines values into a single result."
    )]
    AggregationWithParents,

    #[error("Limit cannot be placed on an Watcher-type query.")]
    LimitOnWatcher,
}

use QueryError as ERR;

impl WatchQuery {
    /// Runs type inferrence to find the return type of the Query
    /// and ensures it is a valid Query
    pub fn check(&self) -> Result<WatchUpdateType, ERR> {
        match self {
            WatchQuery::Metadata => return Ok(WatchUpdateType::Metadata),
            WatchQuery::Component(q) => {
                let it = q
                    .component
                    .igloo_type()
                    .ok_or(ERR::ComponentNoValue(q.component))?;

                Ok(match q.post_op {
                    Some(op) => match op.can_apply(&q.component) {
                        true => WatchUpdateType::ComponentAggregate(it),
                        false => return Err(ERR::InvalidAggregation(q.component, op)),
                    },
                    None => WatchUpdateType::ComponentValue(it),
                })
            }
        }
    }
}

impl OneShotQuery {
    /// Runs type inferrence to find the return type of the Query
    /// and ensures it is a valid Query
    pub fn check(&self) -> Result<R, ERR> {
        Ok(match self {
            OneShotQuery::Extension(q) => match &q.action {
                X::GetID => R::ExtensionID,
                X::Snapshot => R::ExtensionID,
                X::IsAttached => R::ExtensionAttached,
                X::Count => R::Count,
            },
            OneShotQuery::Group(q) => match &q.action {
                G::GetID => R::GroupID,
                G::Snapshot => R::GroupSnapshot,
                G::Count => R::Count,
            },
            OneShotQuery::Device(q) => match &q.action {
                D::GetID => R::DeviceID,
                D::Snapshot(_) => R::DeviceSnapshot,
                D::IsAttached => R::DeviceAttached,
                D::Count => R::Count,
            },
            OneShotQuery::Entity(q) => match &q.action {
                E::Snapshot => R::EntitySnapshot,
                E::Count => R::Count,
            },
            OneShotQuery::Component(q) => {
                match &q.action {
                    C::Count => return Ok(R::Count),
                    _ => {}
                }

                let it = q
                    .component
                    .igloo_type()
                    .ok_or(ERR::ComponentNoValue(q.component))?;

                if q.post_op.is_some() && !matches!(q.action, C::GetValue) {
                    return Err(ERR::AggregationRequiresValueAction);
                }

                if q.post_op.is_some() && q.include_parents {
                    return Err(ERR::AggregationWithParents);
                }

                match &q.action {
                    C::GetValue => match (q.post_op, q.include_parents) {
                        (Some(op), _) => match op.can_apply(&q.component) {
                            true => R::Aggregate(it),
                            false => return Err(ERR::InvalidAggregation(q.component, op)),
                        },
                        (_, true) => R::ComponentValueWithParents(it),
                        (_, false) => R::ComponentValue(it),
                    },
                    C::Set(iv) | C::Put(iv) => {
                        let it_2 = iv.r#type();

                        if it != it_2 {
                            return Err(ERR::ValueTypeMismatch(it_2, it));
                        }

                        R::Count
                    }
                    C::Apply(op) => {
                        if !op.can_eval(&it) {
                            return Err(ERR::OperationNotApplicable(it));
                        }

                        R::Count
                    }
                    C::Count => unreachable!(),
                }
            }
        })
    }
}
