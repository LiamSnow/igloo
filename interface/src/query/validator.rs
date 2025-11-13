use thiserror::Error;

use crate::{
    ComponentType, IglooType,
    query::{ComponentFilter, Query, QueryAction, QueryResultType},
};

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Watch queries cannot have a limit")]
    WatchWithLimit,
    #[error("{0} action requires a component_filter")]
    MissingComponentFilter(String),
    #[error("Cannot determine component type from component_filter")]
    AmbiguousComponentType,
    #[error("Component type {0:?} has no associated IglooType")]
    ComponentHasNoType(ComponentType),
    #[error("Type {0} cannot be aggregated")]
    TypeNotAggregatable(IglooType),
    #[error("Inherit action cannot be validated without runtime context")]
    InheritNotValidatable,
    #[error("{0} action cannot have filters")]
    GFSnapshotWithFilters(String),
}

impl Query {
    pub fn validate(&self) -> Result<QueryResultType, ValidationError> {
        self.validate_watch_limit()?;

        match &self.action {
            QueryAction::Inherit => Err(ValidationError::InheritNotValidatable),
            QueryAction::GetIds => Ok(QueryResultType::Ids),
            QueryAction::SnapshotDevices => Ok(QueryResultType::Devices),
            QueryAction::SnapshotEntities => Ok(QueryResultType::Entities),
            QueryAction::Get => Ok(QueryResultType::Components(self.extract_type()?)),
            QueryAction::GetAggregate(_) => {
                let r#type = self.extract_type()?;
                if !r#type.is_aggregatable() {
                    return Err(ValidationError::TypeNotAggregatable(r#type));
                }
                Ok(QueryResultType::Aggregate(r#type))
            }
            QueryAction::Count => Ok(QueryResultType::Count),
            QueryAction::Set(_)
            | QueryAction::Put(_)
            | QueryAction::Increment(_)
            | QueryAction::WatchDevices
            | QueryAction::WatchEntities => {
                if self.action.is_component_action() {
                    self.require_component_filter()?;
                }
                Ok(QueryResultType::Ok)
            }
            QueryAction::Watch => {
                self.require_component_filter()?;
                Ok(QueryResultType::Components(self.extract_type()?))
            }
            QueryAction::WatchAggregate(_) => {
                let r#type = self.extract_type()?;
                if !r#type.is_aggregatable() {
                    return Err(ValidationError::TypeNotAggregatable(r#type));
                }
                Ok(QueryResultType::Aggregate(r#type))
            }
            QueryAction::SnapshotFloes => {
                self.validate_no_filters()?;
                Ok(QueryResultType::Floes)
            }

            QueryAction::SnapshotGroups => {
                self.validate_no_filters()?;
                Ok(QueryResultType::Groups)
            }
        }
    }

    fn validate_watch_limit(&self) -> Result<(), ValidationError> {
        if matches!(
            self.action,
            QueryAction::Watch
                | QueryAction::WatchDevices
                | QueryAction::WatchEntities
                | QueryAction::WatchAggregate(_)
        ) && self.limit.is_some()
        {
            return Err(ValidationError::WatchWithLimit);
        }
        Ok(())
    }

    fn validate_no_filters(&self) -> Result<(), ValidationError> {
        if self.device_filter.is_some()
            || self.entity_filter.is_some()
            || self.component_filter.is_some()
        {
            return Err(ValidationError::GFSnapshotWithFilters(format!(
                "{}",
                self.action
            )));
        }
        Ok(())
    }

    fn require_component_filter(&self) -> Result<(), ValidationError> {
        if self.component_filter.is_none() {
            return Err(ValidationError::MissingComponentFilter(format!(
                "{}",
                self.action
            )));
        }
        Ok(())
    }

    fn extract_type(&self) -> Result<IglooType, ValidationError> {
        let filter = self
            .component_filter
            .as_ref()
            .ok_or_else(|| ValidationError::MissingComponentFilter(format!("{}", self.action)))?;

        let component_type = filter.extract_comp_type()?;
        component_type
            .igloo_type()
            .ok_or(ValidationError::ComponentHasNoType(component_type))
    }
}

impl QueryAction {
    pub fn is_component_action(&self) -> bool {
        matches!(
            self,
            QueryAction::Get
                | QueryAction::GetAggregate(_)
                | QueryAction::Set(_)
                | QueryAction::Put(_)
                | QueryAction::Increment(_)
                | QueryAction::Watch
                | QueryAction::WatchAggregate(_)
        )
    }
}

impl ComponentFilter {
    fn extract_comp_type(&self) -> Result<ComponentType, ValidationError> {
        match &self {
            ComponentFilter::Type(ct) => Ok(*ct),
            ComponentFilter::Condition(_, comp) => Ok(comp.get_type()),
            ComponentFilter::ListLength(ct, ..) => Ok(*ct),
            ComponentFilter::And(left, right) => left
                .extract_comp_type()
                .or_else(|_| right.extract_comp_type()),
            ComponentFilter::Or(left, right) => left
                .extract_comp_type()
                .or_else(|_| right.extract_comp_type()),
            ComponentFilter::Not(inner) => inner.extract_comp_type(),
            ComponentFilter::UpdatedWithinSeconds(_) => {
                Err(ValidationError::AmbiguousComponentType)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Component,
        id::GroupID,
        query::{AggregationOp, ComparisonOp, DeviceFilter},
    };

    #[test]
    fn test_get_id_single() {
        let query = Query {
            limit: Some(1),
            action: QueryAction::GetIds,
            ..Default::default()
        };
        assert_eq!(query.validate().unwrap(), QueryResultType::Ids);
    }

    #[test]
    fn test_get_id_multiple() {
        let query = Query {
            limit: None,
            action: QueryAction::GetIds,
            ..Default::default()
        };
        assert_eq!(query.validate().unwrap(), QueryResultType::Ids);
    }

    #[test]
    fn test_watch_with_limit_fails() {
        let query = Query {
            limit: Some(5),
            action: QueryAction::Watch,
            component_filter: Some(ComponentFilter::Type(ComponentType::Switch)),
            ..Default::default()
        };
        assert!(matches!(
            query.validate(),
            Err(ValidationError::WatchWithLimit)
        ));
    }

    #[test]
    fn test_get_requires_component_filter() {
        let query = Query {
            action: QueryAction::Get,
            ..Default::default()
        };
        assert!(matches!(
            query.validate(),
            Err(ValidationError::MissingComponentFilter(_))
        ));
    }

    #[test]
    fn test_get_with_type_filter() {
        let query = Query {
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::Get,
            ..Default::default()
        };
        let result = query.validate().unwrap();
        assert!(matches!(
            result,
            QueryResultType::Components(IglooType::Real)
        ));
    }

    #[test]
    fn test_aggregate_mean() {
        let query = Query {
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::GetAggregate(AggregationOp::Mean),
            ..Default::default()
        };
        let result = query.validate().unwrap();
        assert!(matches!(
            result,
            QueryResultType::Aggregate(IglooType::Real)
        ));
    }

    #[test]
    fn test_extract_type_from_condition() {
        let query = Query {
            component_filter: Some(ComponentFilter::Condition(
                ComparisonOp::Gt,
                Component::Dimmer(0.5),
            )),
            action: QueryAction::Get,
            limit: Some(1),
            ..Default::default()
        };
        let result = query.validate().unwrap();
        assert!(matches!(
            result,
            QueryResultType::Components(IglooType::Real)
        ));
    }

    #[test]
    fn test_inherit_fails() {
        let query = Query {
            action: QueryAction::Inherit,
            ..Default::default()
        };
        assert!(matches!(
            query.validate(),
            Err(ValidationError::InheritNotValidatable)
        ));
    }

    #[test]
    fn test_count_always_valid() {
        let query = Query {
            action: QueryAction::Count,
            ..Default::default()
        };
        assert_eq!(query.validate().unwrap(), QueryResultType::Count);
    }

    #[test]
    fn test_floes_snapshot() {
        let query = Query {
            action: QueryAction::SnapshotFloes,
            ..Default::default()
        };
        assert_eq!(query.validate().unwrap(), QueryResultType::Floes);
    }

    #[test]
    fn test_floes_snapshot_with_filter_fails() {
        let query = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            action: QueryAction::SnapshotFloes,
            ..Default::default()
        };
        assert!(matches!(
            query.validate(),
            Err(ValidationError::GFSnapshotWithFilters(_))
        ));
    }

    #[test]
    fn test_groups_snapshot() {
        let query = Query {
            action: QueryAction::SnapshotGroups,
            ..Default::default()
        };
        assert_eq!(query.validate().unwrap(), QueryResultType::Groups);
    }
}
