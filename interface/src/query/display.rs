use crate::{
    ComponentType,
    query::{ComponentFilter, Query, QueryAction},
};
use std::fmt;

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_component_query = matches!(
            &self.action,
            QueryAction::Get
                | QueryAction::GetAggregate(_)
                | QueryAction::Set(_)
                | QueryAction::Put(_)
                | QueryAction::Increment(_)
                | QueryAction::Watch
        );

        let is_entity_query = matches!(
            &self.action,
            QueryAction::SnapshotEntity | QueryAction::WatchEntity
        );

        match &self.action {
            QueryAction::GetAggregate(agg) => {
                write!(f, "get {}", format!("{:?}", agg).to_lowercase())?;

                if let Some(comp_type) = self.component_filter.as_ref().and_then(extract_type) {
                    write!(f, " {:?}", comp_type)?;
                }
            }
            QueryAction::Get | QueryAction::Watch => {
                write!(f, "{}", self.action)?;

                if let Some(comp_type) = self.component_filter.as_ref().and_then(extract_type) {
                    write!(f, " {:?}", comp_type)?;
                }
            }
            QueryAction::Count => {
                if self.component_filter.is_some() {
                    write!(f, "count components")?;
                } else if self.entity_filter.is_some() {
                    write!(f, "count entities")?;
                } else {
                    write!(f, "count devices")?;
                }
            }
            QueryAction::GetId => {
                if matches!(self.limit, Some(1)) {
                    write!(f, "get id")?;
                } else {
                    write!(f, "get ids")?;
                }
            }
            _ => write!(f, "{}", self.action)?,
        }

        if let Some(cf) = &self.component_filter
            && !matches!(cf, ComponentFilter::Type(_))
        {
            write!(f, " where {cf}")?;
        }

        if is_component_query || is_entity_query {
            write!(f, "\nfrom entities")?;
            if let Some(ef) = &self.entity_filter {
                write!(f, " {ef}")?;
            }
        }

        write!(f, "\nfrom devices")?;
        if let Some(df) = &self.device_filter {
            write!(f, " {df}")?;
        }

        if let Some(limit) = self.limit {
            write!(f, "\nlimit {limit}")?;
        }

        Ok(())
    }
}

fn extract_type(filter: &ComponentFilter) -> Option<ComponentType> {
    match filter {
        ComponentFilter::Type(t) => Some(*t),
        ComponentFilter::Condition(_, c) => Some(c.get_type()),
        ComponentFilter::ListLength(t, ..) => Some(*t),
        ComponentFilter::And(left, ..) => extract_type(left),
        ComponentFilter::Or(left, ..) => extract_type(left),
        ComponentFilter::Not(inner) => extract_type(inner),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Component,
        id::{FloeID, GroupID},
        query::{AggregationOp, ComparisonOp, DeviceFilter, EntityFilter},
    };

    #[test]
    fn test_query_display() {
        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            action: QueryAction::GetId,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Owner(FloeID("ESPHome".to_string()))),
            action: QueryAction::Count,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::NameMatches("bedroom*".to_string())),
            limit: Some(1),
            action: QueryAction::SnapshotDevice,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(2, 0))),
            entity_filter: Some(EntityFilter::HasComponent(ComponentFilter::Type(
                ComponentType::Light,
            ))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Switch)),
            limit: Some(1),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            component_filter: Some(ComponentFilter::Condition(
                ComparisonOp::Gt,
                Component::Dimmer(0.5),
            )),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::GetAggregate(AggregationOp::Mean),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::NameMatches("living*".to_string())),
            entity_filter: Some(EntityFilter::HasComponent(ComponentFilter::Type(
                ComponentType::Light,
            ))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::GetAggregate(AggregationOp::Max),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::GetAggregate(AggregationOp::Sum),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::HasComponent(ComponentFilter::Type(
                ComponentType::Light,
            ))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Switch)),
            action: QueryAction::Set(Component::Switch(true)),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::NameEquals("bedroom_light".to_string())),
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            limit: Some(1),
            action: QueryAction::Increment(Component::Dimmer(0.1)),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(2, 0))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Switch)),
            limit: Some(5),
            action: QueryAction::Put(Component::Switch(false)),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Dimmer)),
            action: QueryAction::Watch,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Owner(FloeID("ESPHome".to_string()))),
            action: QueryAction::WatchDevice,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::HasComponent(ComponentFilter::Type(
                ComponentType::Light,
            ))),
            action: QueryAction::WatchEntity,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::And(
                Box::new(DeviceFilter::Group(GroupID::from_parts(1, 0))),
                Box::new(DeviceFilter::EntityCount(ComparisonOp::Gt, 5)),
            )),
            action: QueryAction::Count,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Or(
                Box::new(DeviceFilter::Group(GroupID::from_parts(1, 0))),
                Box::new(DeviceFilter::Group(GroupID::from_parts(2, 0))),
            )),
            component_filter: Some(ComponentFilter::Type(ComponentType::Switch)),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::Not(Box::new(EntityFilter::HasComponent(
                ComponentFilter::Type(ComponentType::Light),
            )))),
            component_filter: Some(ComponentFilter::Type(ComponentType::Switch)),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::And(
                Box::new(DeviceFilter::Group(GroupID::from_parts(1, 0))),
                Box::new(DeviceFilter::HasEntity(EntityFilter::And(
                    Box::new(EntityFilter::HasComponent(ComponentFilter::Type(
                        ComponentType::Light,
                    ))),
                    Box::new(EntityFilter::ComponentCount(ComparisonOp::Gte, 3)),
                ))),
            )),
            entity_filter: Some(EntityFilter::HasComponent(ComponentFilter::Type(
                ComponentType::Dimmer,
            ))),
            component_filter: Some(ComponentFilter::Condition(
                ComparisonOp::Gt,
                Component::Dimmer(0.3),
            )),
            limit: Some(10),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::ComponentCount(ComparisonOp::Gt, 3)),
            action: QueryAction::SnapshotEntity,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            entity_filter: Some(EntityFilter::NameMatches("*_sensor".to_string())),
            action: QueryAction::Count,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Owner(FloeID("ESPHome".to_string()))),
            entity_filter: Some(EntityFilter::UpdatedWithinSeconds(60)),
            limit: Some(1),
            action: QueryAction::SnapshotEntity,
            ..Default::default()
        };
        println!("{}\n", q);
    }
}
