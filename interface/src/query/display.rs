use crate::query::{Query, QueryAction as A, QueryTarget};
use std::fmt;

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.action {
            A::Set(v) => {
                write!(f, "set {} to {v}", self.target)?;
            }
            A::Put(v) => match &self.target {
                QueryTarget::Components(t) => {
                    write!(f, "put {:?}({v})", t)?;
                }
                _ => {
                    write!(f, "put INVALID({v})")?;
                }
            },
            A::Apply(op) => {
                write!(f, "apply {} {op}", self.target)?;
            }
            _ => {
                write!(f, "{} {}", self.action, self.target)?;
            }
        }

        let prefix = match &self.action {
            A::Set(_) | A::Put(_) | A::Apply(_) => "on",
            _ => "from",
        };

        match &self.target {
            QueryTarget::Floes => {
                if let Some(ff) = &self.floe_filter {
                    write!(f, "\n{prefix} Floes {ff}")?;
                }
            }
            QueryTarget::Groups => {
                if let Some(gf) = &self.group_filter {
                    write!(f, "\n{prefix} Groups {gf}")?;
                }
            }
            QueryTarget::Devices => {
                if let Some(df) = &self.device_filter {
                    write!(f, "\n{prefix} Devices {df}")?;
                }
                if let Some(gf) = &self.group_filter {
                    if self.device_filter.is_none() {
                        write!(f, "\n{prefix} Devices")?;
                    }

                    write!(f, "\n{prefix} Groups {gf}")?;
                }
                if let Some(ff) = &self.floe_filter {
                    if self.device_filter.is_none() && self.group_filter.is_none() {
                        write!(f, "\n{prefix} Devices")?;
                    }

                    write!(f, "\n{prefix} Floes {ff}")?;
                }
            }
            QueryTarget::Entities => {
                if let Some(ef) = &self.entity_filter {
                    write!(f, "\n{prefix} Entities {ef}")?;
                }
                if let Some(df) = &self.device_filter {
                    write!(f, "\n{prefix} Devices {df}")?;
                }
                if let Some(gf) = &self.group_filter {
                    if self.device_filter.is_none() {
                        write!(f, "\n{prefix} Devices")?;
                    }

                    write!(f, "\n{prefix} Groups {gf}")?;
                }
                if let Some(ff) = &self.floe_filter {
                    write!(f, "\n{prefix} Floes {ff}")?;
                }
            }
            QueryTarget::Components(_) => {
                if let Some(ef) = &self.entity_filter {
                    write!(f, "\n{prefix} Entities {ef}")?;
                }
                if let Some(df) = &self.device_filter {
                    write!(f, "\n{prefix} Devices {df}")?;
                }
                if let Some(gf) = &self.group_filter {
                    if self.device_filter.is_none() {
                        write!(f, "\n{prefix} Devices")?;
                    }

                    write!(f, "\n{prefix} Groups {gf}")?;
                }
                if let Some(ff) = &self.floe_filter {
                    if self.device_filter.is_none() && self.group_filter.is_none() {
                        write!(f, "\n{prefix} Devices")?;
                    }

                    write!(f, "\n{prefix} Floes {ff}")?;
                }
            }
        }

        if let Some(limit) = self.limit {
            write!(f, "\nlimit {limit}")?;
        }

        Ok(())
    }
}

pub fn pluralize(word: String) -> String {
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
        if let Some(c) = before_y
            && !"aeiou".contains(c)
        {
            let mut result = word.clone();
            result.pop();
            return format!("{}ies", result);
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
        if let Some(c) = before_o
            && !"aeiou".contains(c)
        {
            return format!("{}es", word);
        }
    }

    format!("{}s", word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Component, ComponentType as CT, IglooValue,
        id::{FloeID, GroupID},
        query::{DeviceFilter, EntityFilter, FloeFilter, GroupFilter, QueryAction},
        types::{agg::AggregationOp, compare::ComparisonOp, math::MathOp},
    };

    #[test]
    fn test_query_display() {
        let q = Query {
            target: QueryTarget::Devices,
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::GetIds,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Devices,
            floe_filter: Some(FloeFilter::Id(FloeID("ESPHome".to_string()))),
            action: QueryAction::Count,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Devices,
            device_filter: Some(DeviceFilter::NameMatches("bedroom*".to_string())),
            limit: Some(1),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Switch),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(2, 0))),
            limit: Some(1),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::Condition(
                ComparisonOp::Gt,
                Component::Dimmer(0.5),
            )),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::GetAggregate(AggregationOp::Mean),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            device_filter: Some(DeviceFilter::NameMatches("living*".to_string())),
            action: QueryAction::GetAggregate(AggregationOp::Max),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            action: QueryAction::GetAggregate(AggregationOp::Sum),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Switch),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::Set(IglooValue::Boolean(true)),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            device_filter: Some(DeviceFilter::NameEquals("bedroom_light".to_string())),
            limit: Some(1),
            action: QueryAction::Apply(MathOp::Add(IglooValue::Real(0.1))),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Switch),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(2, 0))),
            limit: Some(5),
            action: QueryAction::Put(IglooValue::Boolean(false)),
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::Watch,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Devices,
            floe_filter: Some(FloeFilter::Id(FloeID("ESPHome".to_string()))),
            action: QueryAction::Watch,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Entities,
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::Watch,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Devices,
            device_filter: Some(DeviceFilter::All(vec![DeviceFilter::EntityCount(
                ComparisonOp::Gt,
                5,
            )])),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            action: QueryAction::Count,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Switch),
            group_filter: Some(GroupFilter::Any(vec![
                GroupFilter::Id(GroupID::from_parts(1, 0)),
                GroupFilter::Id(GroupID::from_parts(2, 0)),
            ])),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Switch),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::Not(Box::new(EntityFilter::Has(CT::Light)))),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Components(CT::Dimmer),
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            device_filter: Some(DeviceFilter::HasEntity(EntityFilter::All(vec![
                EntityFilter::Has(CT::Light),
                EntityFilter::ComponentCount(ComparisonOp::Gte, 3),
            ]))),
            entity_filter: Some(EntityFilter::All(vec![
                EntityFilter::Has(CT::Dimmer),
                EntityFilter::Condition(ComparisonOp::Gt, Component::Dimmer(0.3)),
            ])),
            limit: Some(10),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Entities,
            group_filter: Some(GroupFilter::Id(GroupID::from_parts(1, 0))),
            entity_filter: Some(EntityFilter::ComponentCount(ComparisonOp::Gt, 3)),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Entities,
            entity_filter: Some(EntityFilter::NameMatches("*_sensor".to_string())),
            action: QueryAction::Count,
            ..Default::default()
        };
        println!("{}\n", q);

        let q = Query {
            target: QueryTarget::Entities,
            floe_filter: Some(FloeFilter::Id(FloeID("ESPHome".to_string()))),
            entity_filter: Some(EntityFilter::UpdatedWithinSeconds(60)),
            limit: Some(1),
            action: QueryAction::Get,
            ..Default::default()
        };
        println!("{}\n", q);
    }
}
