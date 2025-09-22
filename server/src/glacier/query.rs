/// set queries will set the values of target components
pub struct SetOperation {
    target: ComponentType,
    /// value must be of the same type as ComponentType
    value: ComponentValue,
    filter: QueryFilter,
}

/// get queries will simply return all the values of the components
pub struct ValueQuery {
    get: Vec<ComponentType>,
    filter: QueryFilter,
}

pub struct AggregateQuery {
    targets: Vec<ComponentType>,
    filter: QueryFilter,
    op: AggregateOp,
}

/// lookup queries will return a Vec of strong links to components
pub struct LookupQuery {
    filter: QueryFilter,
}

pub enum QueryFilter {
    With(ComponentType),
    Without(ComponentType),
    And(Box<(QueryFilter, QueryFilter)>),
    Or(Box<(QueryFilter, QueryFilter)>),
    Condition(ComponentType, Operator, ComponentValue),
    NestedCondition(Vec<PathSegment>, Operator, ComponentValue),
}

pub enum PathSegment {
    Field(String),
    Index(usize),
}

pub enum Operator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}

pub enum AggregateOp {
    Average,
    Sum,
    Min,
    Max,
    Count,
    AverageColor,
}
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use uuid::Uuid;

    use crate::*;

    #[test]
    fn test_query() {
        let mut entities = Entities(HashMap::from([
            (
                "LightWithSwitchDimmerColor".to_string(),
                Entity(vec![
                    ComponentValue::Light,
                    ComponentValue::Switch(Switch(true)),
                    ComponentValue::Dimmer(Dimmer(255)),
                    ComponentValue::Color(Color { r: 255, g: 0, b: 0 }),
                ]),
            ),
            (
                "LightWithSwitchDimmer".to_string(),
                Entity(vec![
                    ComponentValue::Light,
                    ComponentValue::Switch(Switch(true)),
                    ComponentValue::Dimmer(Dimmer(255)),
                ]),
            ),
            (
                "Dimmer".to_string(),
                Entity(vec![ComponentValue::Dimmer(Dimmer(255))]),
            ),
            (
                "LightWithSwitch".to_string(),
                Entity(vec![
                    ComponentValue::Light,
                    ComponentValue::Switch(Switch(true)),
                ]),
            ),
            // (
            //     "RGBCT_Bulb".to_string(),
            //     Entity(vec![
            //         ComponentValue::Light,
            //         ComponentValue::Switch(Switch(true)),
            //         ComponentValue::Dimmer(Dimmer(255)),
            //         ComponentValue::Color(Color { r: 255, g: 0, b: 0 }),
            //     ]),
            // ),
            // (
            //     "Status".to_string(),
            //     Entity(vec![ComponentValue::Bool(Bool(true))]),
            // ),
            // (
            //     "Safe Mode".to_string(),
            //     Entity(vec![ComponentValue::Bool(Bool(false))]),
            // ),
            // (
            //     "Uptime Sensor".to_string(),
            //     Entity(vec![
            //         ComponentValue::Unit(Unit::Seconds),
            //         ComponentValue::Int(Int(128211)),
            //     ]),
            // ),
            // (
            //     "IP Address".to_string(),
            //     Entity(vec![ComponentValue::String(StringComponent(
            //         "192.168.1.201".to_string(),
            //     ))]),
            // ),
        ]));

        // let a = serde_json::to_string_pretty(&entities).unwrap();
        // fs::write("out.json", a).unwrap();

        let mut world = HashMap::new();
        world.insert(
            Uuid::now_v7(),
            Device {
                name: "kitchen_ceiling".to_string(),
                entities,
            },
        );
    }

    #[test]
    fn test_query_parsing() {
        // syntax is open to change

        // GET Dimmer WHERE Light
        // should equal
        let expected = ValueQuery {
            get: vec![ComponentType::Dimmer],
            filter: QueryFilter::With(ComponentType::Light),
        };

        // GET Dimmer, Color WITH Light AND Switch
        let expected = ValueQuery {
            get: vec![ComponentType::Dimmer, ComponentType::Color],
            filter: QueryFilter::And(Box::new((
                QueryFilter::With(ComponentType::Light),
                QueryFilter::With(ComponentType::Switch),
            ))),
        };

        // GET Dimmer, Color WITH Light OR Switch
        let expected = ValueQuery {
            get: vec![ComponentType::Dimmer, ComponentType::Color],
            filter: QueryFilter::Or(Box::new((
                QueryFilter::With(ComponentType::Light),
                QueryFilter::With(ComponentType::Switch),
            ))),
        };

        // GET Dimmer WITH NOT Light
        let expected = ValueQuery {
            get: vec![ComponentType::Dimmer],
            filter: QueryFilter::Without(ComponentType::Light),
        };

        // SET Dimmer WITH Light TO 100
        let expected = SetOperation {
            target: ComponentType::Dimmer,
            value: ComponentValue::Dimmer(Dimmer(100)),
            filter: QueryFilter::With(ComponentType::Light),
        };

        // LOOKUP Dimmer AND NOT Light (should give DEVICE_UUID["Dimmer"][0] in this example)
        let expected = LookupQuery {
            filter: QueryFilter::And(Box::new((
                QueryFilter::With(ComponentType::Dimmer),
                QueryFilter::Without(ComponentType::Light),
            ))),
        };
    }
}
