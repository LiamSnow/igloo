use crate::{
    ComponentType as CT, IglooType,
    query::{
        ComponentAction as C, DeviceAction as D, EntityAction as E, ExtensionAction as X,
        GroupAction as G, Query, QueryResultType as R,
    },
    types::agg::AggregationOp,
};
use thiserror::Error;

// TODO validate globs

#[derive(Error, Debug, Clone)]
pub enum QueryError {
    #[error("Inherit is an invalid action. Queries must be merged first.")]
    Inherit,

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

    #[error("Aggregation can only be used with GetValue or ObserveValue actions.")]
    AggregationRequiresValueAction,

    #[error(
        "Cannot use aggregation with include_parents. Aggregation combines values into a single result."
    )]
    AggregationWithParents,
}

use QueryError as ERR;

impl Query {
    /// Runs type inferrence to find the return type of the Query
    /// and ensures it is a valid Query
    pub fn check(&self) -> Result<R, ERR> {
        Ok(match self {
            Query::Extension(q) => match &q.action {
                X::GetID => R::ExtensionID,
                X::Snapshot => R::ExtensionID,
                X::IsAttached => R::ExtensionAttached,
                X::ObserveAttached => R::ExtensionAttached,
                X::Count => R::Count,
                X::Inherit => return Err(ERR::Inherit),
            },
            Query::Group(q) => match &q.action {
                G::GetID => R::GroupID,
                G::Snapshot => R::GroupSnapshot,
                G::ObserveRename => R::Ok,
                G::ObserveMembershipChanged => R::Ok,
                G::Count => R::Count,
                G::Inherit => return Err(ERR::Inherit),
            },
            Query::Device(q) => match &q.action {
                D::GetID => R::DeviceID,
                D::Snapshot(_) => R::DeviceSnapshot,
                D::IsAttached => R::DeviceAttached,
                D::ObserveAttached => R::DeviceAttached,
                D::ObserveName => R::DeviceID,
                D::ObserveEntityAdded => R::DeviceID,
                D::ObserveComponentPut => R::DeviceID,
                D::Count => R::Count,
                D::Inherit => return Err(ERR::Inherit),
            },
            Query::Entity(q) => match &q.action {
                E::Snapshot => R::EntitySnapshot,
                E::ObserveComponentPut => R::EntitySnapshot,
                E::Count => R::Count,
                E::Inherit => return Err(ERR::Inherit),
            },
            Query::Component(q) => {
                match &q.action {
                    C::Count => return Ok(R::Count),
                    C::Inherit => return Err(ERR::Inherit),
                    _ => {}
                }

                let it = q
                    .component
                    .igloo_type()
                    .ok_or(ERR::ComponentNoValue(q.component))?;

                if q.post_op.is_some() && !matches!(q.action, C::GetValue | C::ObserveValue) {
                    return Err(ERR::AggregationRequiresValueAction);
                }

                if q.post_op.is_some() && q.include_parents {
                    return Err(ERR::AggregationWithParents);
                }

                match &q.action {
                    C::GetValue | C::ObserveValue => {
                        if let Some(op) = q.post_op
                            && !op.can_apply(&q.component)
                        {
                            return Err(ERR::InvalidAggregation(q.component, op));
                        }

                        match q.include_parents {
                            true => R::ComponentValueWithParents(it),
                            false => R::ComponentValue(it),
                        }
                    }
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
                    C::Count | C::Inherit => unreachable!(),
                }
            }
        })
    }
}
