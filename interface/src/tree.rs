use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// a "strong component link" is defined as
//   DEVICE_UUID[ENTITY_NAME][COMPONENT_INDEX]

// TODO remove
pub struct World {
    pub devices: HashMap<Uuid, Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub entities: Entities,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entities(pub HashMap<String, Entity>);

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entity(pub Vec<ComponentValue>);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ComponentType {
    Int,
    Float,
    Bool,
    String,
    Object,
    List,
    Light,
    Switch,
    Dimmer,
    Color,
    Unit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComponentValue {
    // primitives
    Int(Int),
    Float(Float),
    Bool(Bool),
    String(StringComponent),

    // composites
    Object(Object),
    List(List),

    // custom
    Light,
    Switch(Switch),
    Dimmer(Dimmer),
    Color(Color),
    Unit(Unit),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Int(pub i32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Float(pub f64);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bool(pub bool);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StringComponent(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Object(pub HashMap<String, ComponentValue>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List(pub Vec<ComponentValue>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Switch(pub bool);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dimmer(pub u8);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Unit {
    Seconds,
    // TODO
}

impl ComponentValue {
    pub fn get_type(&self) -> ComponentType {
        match self {
            ComponentValue::Int(_) => ComponentType::Int,
            ComponentValue::Float(_) => ComponentType::Float,
            ComponentValue::Bool(_) => ComponentType::Bool,
            ComponentValue::String(_) => ComponentType::String,
            ComponentValue::Object(_) => ComponentType::Object,
            ComponentValue::List(_) => ComponentType::List,
            ComponentValue::Light => ComponentType::Light,
            ComponentValue::Switch(_) => ComponentType::Switch,
            ComponentValue::Dimmer(_) => ComponentType::Dimmer,
            ComponentValue::Color(_) => ComponentType::Color,
            ComponentValue::Unit(_) => ComponentType::Unit,
        }
    }
}

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

impl Entities {
    pub fn values(&self) -> Vec<&Entity> {
        self.0.values().collect()
    }

    pub fn values_mut(&mut self) -> Vec<&mut Entity> {
        self.0.values_mut().collect()
    }

    pub fn query_dimmers(&self, filter: Option<&Vec<ComponentType>>) -> Vec<&Dimmer> {
        let mut res = Vec::new();
        for entity in self.0.values() {
            if let Some(filter) = filter
                && !entity.matches_filter(filter)
            {
                continue;
            }

            res.append(&mut entity.get_dimmers());
        }
        res
    }
}

impl Entity {
    pub fn matches_filter(&self, filter: &Vec<ComponentType>) -> bool {
        let mut typs = Vec::new();
        for comp in &self.0 {
            typs.push(comp.get_type());
        }
        for filter in filter {
            if !typs.contains(filter) {
                return false;
            }
        }
        true
    }

    pub fn get_dimmers(&self) -> Vec<&Dimmer> {
        let mut res = Vec::new();
        for comp in &self.0 {
            if let ComponentValue::Dimmer(dimmer) = &comp {
                res.push(dimmer);
            }
        }
        res
    }
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
                provider: "ESPHome".to_string(),
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
