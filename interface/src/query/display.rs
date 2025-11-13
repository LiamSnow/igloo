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
            QueryAction::SnapshotEntities | QueryAction::WatchEntities
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
                    write!(f, " {}", pluralize(format!("{comp_type:?}")))?;
                }
            }
            QueryAction::Set(_) | QueryAction::Put(_) | QueryAction::Increment(_) => {
                write!(f, "{}", pluralize(self.action.to_string()))?;
            }
            QueryAction::Count => {
                if self.component_filter.is_some() {
                    write!(f, "count Components")?;
                } else if self.entity_filter.is_some() {
                    write!(f, "count Entities")?;
                } else {
                    write!(f, "count Devices")?;
                }
            }
            _ => write!(f, "{}", self.action)?,
        }

        if let Some(cf) = &self.component_filter
            && !matches!(cf, ComponentFilter::Type(_))
        {
            write!(f, " with {cf}")?;
        }

        if is_component_query || is_entity_query {
            write!(f, "\nfrom Entities")?;
            if let Some(ef) = &self.entity_filter {
                write!(f, " {ef}")?;
            }
        }

        write!(f, "\nfrom Devices")?;
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

fn pluralize(word: String) -> String {
    let lower = word.to_lowercase();

    if lower.ends_with("s")
        || lower.ends_with("x")
        || lower.ends_with("z")
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        return format!("{}es", word);
    }

    // consonant + y -> consonant + ies
    if lower.ends_with("y") {
        let before_y = lower.chars().rev().nth(1);
        if let Some(c) = before_y {
            if !"aeiou".contains(c) {
                let mut result = word.clone();
                result.pop(); // Remove 'y'
                return format!("{}ies", result);
            }
        }
    }

    if lower.ends_with("fe") {
        let mut result = word.clone();
        result.pop();
        result.pop();
        return format!("{}ves", result);
    }
    if lower.ends_with("f") {
        let mut result = word.clone();
        result.pop();
        return format!("{}ves", result);
    }

    // consonant + o -> add es
    if lower.ends_with("o") {
        let before_o = lower.chars().rev().nth(1);
        if let Some(c) = before_o {
            if !"aeiou".contains(c) {
                return format!("{}es", word);
            }
        }
    }

    format!("{}s", word)
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
            action: QueryAction::GetIds,
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
            action: QueryAction::SnapshotDevices,
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
            action: QueryAction::WatchDevices,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            device_filter: Some(DeviceFilter::Group(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::HasComponent(ComponentFilter::Type(
                ComponentType::Light,
            ))),
            action: QueryAction::WatchEntities,
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
            action: QueryAction::SnapshotEntities,
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
            action: QueryAction::SnapshotEntities,
            ..Default::default()
        };
        println!("{}\n", q);
    }
}
