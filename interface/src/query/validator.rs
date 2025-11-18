use crate::{
    ComponentType as CT, IglooType,
    query::{
        DeviceFilter, EntityFilter, FloeFilter, GroupFilter, Query, QueryAction as A,
        QueryResultType as R, QueryTarget as QT,
    },
};
use thiserror::Error;

// TODO make sure can't Get, Put, Set, Increment, Watch, etc. non readable value

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Inherit is an invalid action. Queries must be merged first.")]
    Inherit,

    #[error(
        "Cannot get IDs of individual components. Component queries return values grouped by device and entity. Use 'Get' to retrieve values, or query Devices/Entities to get their IDs."
    )]
    ComponentID,

    #[error(
        "Component type '{0:?}' has no readable value. Only components like sensors, switches, or dimmers can be queried with 'Get'."
    )]
    ComponentNotReadable(CT),

    #[error(
        "Cannot aggregate {0}. Aggregation (sum, mean, min, max) only works on Component values. Use 'Count' to count {0}."
    )]
    TargetNotAggregatable(QT),

    #[error(
        "Component type '{0:?}' cannot be aggregated. Only numeric components can use aggregation operations."
    )]
    ComponentNotAggregatable(CT),

    #[error("Cannot aggregate values of type {0}. Aggregation requires numeric types.")]
    TypeNotAggregatable(IglooType),

    #[error(
        "Cannot filter by Floe properties when querying {0}. Change your target to 'Floes', or use hierarchical filters like 'Devices in Floe where...' in your {0} filter."
    )]
    InvalidFloeFilter(QT),

    #[error(
        "Cannot filter by Group properties when querying {0}. Change your target to 'Groups', or use hierarchical filters like 'Devices in Group where...' in your {0} filter."
    )]
    InvalidGroupFilter(QT),

    #[error(
        "Cannot filter by Device properties when querying {0}. Device filters only apply to Devices, Entities, or Components. Use {0}-specific filters, or use hierarchical filters like '{0} that contain Devices where...'."
    )]
    InvalidDeviceFilter(QT),

    #[error(
        "Cannot filter by Entity properties when querying {0}. Entity filters only apply to Entities or Components. Use {0}-specific filters, or use hierarchical filters like 'Devices with Entity where...'."
    )]
    InvalidEntityFilter(QT),

    #[error(
        "'Any' filters combine multiple conditions where at least one must match. You need at least 2 conditions to create an 'Or' filter. If you only have one condition, use it directly without 'Or'."
    )]
    ShortAny,

    #[error(
        "'All' filters combine multiple conditions where all must match. You need at least 2 conditions to create an 'And' filter. If you only have one condition, use it directly without 'And'."
    )]
    ShortAll,

    #[error(
        "Cannot modify {0}. Only Components can use 'Set', 'Put', or 'Apply'. Use 'Get' to query {0}."
    )]
    TargetNotApplicable(QT),

    #[error(
        "Component type '{0:?}' has no writable value. Only components with values (sensors, switches, dimmers, etc.) can be modified with 'Set', 'Put', or 'Apply'."
    )]
    ComponentNotApplicable(CT),

    #[error(
        "Operation not applicable to type {0}. Check that the operation is valid for this component's value type."
    )]
    OperationNotApplicable(IglooType),
}

use ValidationError as E;

impl Query {
    pub fn validate(&self) -> Result<R, E> {
        match &self.target {
            QT::Floes => {
                self.assert_no_group_filter()?;
                self.assert_no_device_filter()?;
                self.assert_no_entity_filter()?;
            }
            QT::Groups => {
                self.assert_no_floe_filter()?;
                self.assert_no_device_filter()?;
                self.assert_no_entity_filter()?;
            }
            QT::Devices => {
                self.assert_no_entity_filter()?;
            }
            QT::Entities | QT::Components(_) => {}
        }

        self.validate_filters()?;

        match &self.action {
            A::Get | A::Watch => Ok(match self.target {
                QT::Floes => R::Floes,
                QT::Groups => R::Groups,
                QT::Devices => R::Devices,
                QT::Entities => R::Entities,
                QT::Components(t) => match t.igloo_type() {
                    Some(t) => R::Components(t),
                    None => return Err(E::ComponentNotReadable(t)),
                },
            }),

            A::GetAggregate(op) | A::WatchAggregate(op) => {
                let t = match self.target {
                    QT::Components(t) => match t.igloo_type() {
                        Some(t) => t,
                        None => return Err(E::ComponentNotAggregatable(t)),
                    },
                    _ => return Err(E::TargetNotAggregatable(self.target)),
                };

                if !op.can_apply(&t) {
                    return Err(E::TypeNotAggregatable(t));
                }

                Ok(R::Aggregate(t))
            }

            A::GetIds => Ok(match self.target {
                QT::Floes => R::FloeIds,
                QT::Groups => R::GroupIds,
                QT::Devices => R::DeviceIds,
                QT::Entities => R::EntityIds,
                QT::Components(_) => return Err(E::ComponentID),
            }),

            A::Set(_) | A::Put(_) => {
                match self.target {
                    QT::Components(comp_type) => {
                        if comp_type.igloo_type().is_none() {
                            return Err(E::ComponentNotApplicable(comp_type));
                        }
                    }
                    _ => return Err(E::TargetNotApplicable(self.target)),
                }

                Ok(R::Ok)
            }

            A::Apply(op) => {
                let t = match self.target {
                    QT::Components(comp_type) => match comp_type.igloo_type() {
                        Some(t) => t,
                        None => return Err(E::ComponentNotApplicable(comp_type)),
                    },
                    _ => return Err(E::TargetNotApplicable(self.target)),
                };

                if !op.can_eval(&t) {
                    return Err(E::OperationNotApplicable(t));
                }

                Ok(R::Ok)
            }

            A::Count => Ok(R::Count),

            A::Inherit => Err(E::Inherit),
        }
    }

    fn validate_filters(&self) -> Result<(), E> {
        if let Some(f) = &self.floe_filter {
            f.validate()?;
        }
        if let Some(f) = &self.group_filter {
            f.validate()?;
        }
        if let Some(f) = &self.device_filter {
            f.validate()?;
        }
        if let Some(f) = &self.entity_filter {
            f.validate()?;
        }
        Ok(())
    }

    fn assert_no_floe_filter(&self) -> Result<(), E> {
        if self.floe_filter.is_some() {
            Err(E::InvalidFloeFilter(self.target))
        } else {
            Ok(())
        }
    }

    fn assert_no_group_filter(&self) -> Result<(), E> {
        if self.group_filter.is_some() {
            Err(E::InvalidGroupFilter(self.target))
        } else {
            Ok(())
        }
    }

    fn assert_no_device_filter(&self) -> Result<(), E> {
        if self.device_filter.is_some() {
            Err(E::InvalidDeviceFilter(self.target))
        } else {
            Ok(())
        }
    }

    fn assert_no_entity_filter(&self) -> Result<(), E> {
        if self.entity_filter.is_some() {
            Err(E::InvalidEntityFilter(self.target))
        } else {
            Ok(())
        }
    }
}

impl FloeFilter {
    fn validate(&self) -> Result<(), E> {
        match self {
            FloeFilter::All(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAll);
                }
                for f in v {
                    f.validate()?;
                }
            }
            FloeFilter::Any(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAny);
                }
                for f in v {
                    f.validate()?;
                }
            }
            FloeFilter::Not(f) => f.validate()?,
            FloeFilter::HasDevice(f) => f.validate()?,
            FloeFilter::AllDevices(f) => f.validate()?,
            _ => {}
        }
        Ok(())
    }
}

impl GroupFilter {
    fn validate(&self) -> Result<(), E> {
        match self {
            GroupFilter::All(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAll);
                }
                for f in v {
                    f.validate()?;
                }
            }
            GroupFilter::Any(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAny);
                }
                for f in v {
                    f.validate()?;
                }
            }
            GroupFilter::Not(f) => f.validate()?,
            GroupFilter::HasDevice(f) => f.validate()?,
            GroupFilter::AllDevices(f) => f.validate()?,
            _ => {}
        }
        Ok(())
    }
}

impl DeviceFilter {
    fn validate(&self) -> Result<(), E> {
        match self {
            DeviceFilter::All(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAll);
                }
                for f in v {
                    f.validate()?;
                }
            }
            DeviceFilter::Any(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAny);
                }
                for f in v {
                    f.validate()?;
                }
            }
            DeviceFilter::Not(f) => f.validate()?,
            DeviceFilter::HasEntity(f) => f.validate()?,
            DeviceFilter::AllEntities(f) => f.validate()?,
            _ => {}
        }
        Ok(())
    }
}

impl EntityFilter {
    fn validate(&self) -> Result<(), E> {
        match self {
            EntityFilter::All(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAll);
                }
                for f in v {
                    f.validate()?;
                }
            }
            EntityFilter::Any(v) => {
                if v.len() < 2 {
                    return Err(E::ShortAny);
                }
                for f in v {
                    f.validate()?;
                }
            }
            EntityFilter::Not(f) => f.validate()?,
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Component,
        types::{agg::AggregationOp, compare::ComparisonOp},
    };

    #[test]
    fn test_ids() {
        let query = Query {
            action: A::GetIds,
            target: QT::Devices,
            limit: Some(1),
            ..Default::default()
        };
        assert_eq!(query.validate().unwrap(), R::DeviceIds);
    }

    #[test]
    fn test_get_with_type_filter() {
        let query = Query {
            action: A::Get,
            target: QT::Components(CT::Dimmer),
            ..Default::default()
        };
        let result = query.validate().unwrap();
        assert!(matches!(result, R::Components(IglooType::Real)));
    }

    #[test]
    fn test_aggregate_mean() {
        let query = Query {
            action: A::GetAggregate(AggregationOp::Mean),
            target: QT::Components(CT::Dimmer),
            ..Default::default()
        };
        let result = query.validate().unwrap();
        assert!(matches!(result, R::Aggregate(IglooType::Real)));
    }

    #[test]
    fn test_get_switch_with_dimmer_condition() {
        let query = Query {
            action: A::Get,
            target: QT::Components(CT::Switch),
            entity_filter: Some(EntityFilter::Condition(
                ComparisonOp::Gt,
                Component::Dimmer(0.5),
            )),
            limit: Some(1),
            ..Default::default()
        };
        let result = query.validate().unwrap();
        assert!(matches!(result, R::Components(IglooType::Boolean)));
    }
}
