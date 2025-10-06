use igloo_interface::{Component, ComponentType};
use std::time::Instant;
use uuid::Uuid;

/// Request a Query from the Query Engine
#[derive(Debug)]
pub struct GlobalQueryRequest {
    pub filter: QueryFilter,
    pub area: GlobalArea,
    pub kind: QueryKind,
    pub started_at: Instant,
}

/// Procesed [GlobalQueryRequest], dispatched to each Floe
#[derive(Debug)]
pub struct LocalQueryRequest {
    pub filter: QueryFilter,
    pub area: LocalArea,
    pub kind: QueryKind,
    pub started_at: Instant,
}

#[derive(Debug, Clone)]
pub enum QueryKind {
    Set(Vec<Component>),
    // OneGet(mpsc::Sender<()>, ComponentType),
    // OneAvg(mpsc::Sender<()>, ComponentType),
    // WatchGet(mpsc::Sender<()>, ComponentType),
    // WatchAvg(mpsc::Sender<()>, ComponentType),
    // Snapshot(mpsc::Sender<()>),
}

#[derive(Debug, Clone)]
pub enum GlobalArea {
    All,
    /// Zone ID
    Zone(Uuid),
    /// Global Device ID
    Device(String),
    /// Global Device ID, Entity Name
    Entity(String, String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LocalArea {
    All,
    Device(u16),
    Entity(u16, u16),
}

#[derive(Debug, Clone)]
pub enum QueryFilter {
    /// exclude entities that don't also have this component
    With(ComponentType),
    /// exclude entities that have this component
    Without(ComponentType),
    /// both queries must be true
    And(Box<(QueryFilter, QueryFilter)>),
    /// either query must be true
    Or(Box<(QueryFilter, QueryFilter)>),
    // Condition(ComponentType, Operator, Component),
    // for refering to parts of components, ex. color.r
    // NestedCondition(Vec<PathSegment>, Operator, Component),
    // TODO think it would be cool to have filters for entity names
    // IE `RGBCT_Bulb*`
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}

#[cfg(test)]
mod tests {
    use igloo_interface::Dimmer;

    use super::*;

    #[test]
    fn example_queries() {
        use OneShotQuery::*;
        use QueryFilter::*;

        // get all Dimmer components that exists on an entity that also contains a Light component
        let expected = Query {
            kind: Get(Uuid::now_v7(), vec![ComponentType::Dimmer]),
            filter: With(ComponentType::Light),
            area: GlobalArea::All,
            limit: None,
        };

        // get the first Dimmer component that exists on an entity that also contains a Light component
        let expected = Query {
            kind: Get(Uuid::now_v7(), vec![ComponentType::Dimmer]),
            filter: With(ComponentType::Light),
            area: GlobalArea::All,
            limit: Some(1),
        };

        // get all Dimmer and Color components that exist on entities that also contain Light and Switch components
        // IE entities must have Dimmer, Color, Light, and Switch components, but only return Dimmer and Color
        let expected = Query {
            kind: Get(
                Uuid::now_v7(),
                vec![ComponentType::Dimmer, ComponentType::Color],
            ),
            filter: And(Box::new((
                With(ComponentType::Light),
                With(ComponentType::Switch),
            ))),
            area: GlobalArea::All,
            limit: None,
        };

        // get all Dimmer components that exist on entities without a Light component
        let expected = Query {
            kind: Get(Uuid::now_v7(), vec![ComponentType::Dimmer]),
            filter: Without(ComponentType::Light),
            area: GlobalArea::All,
            limit: None,
        };

        // set all Dimmer components to 50% that exist on entities that also have a Light component
        let expected = Query {
            kind: Set(vec![Component::Dimmer(Dimmer(0.5))]),
            filter: With(ComponentType::Light),
            area: GlobalArea::All,
            limit: None,
        };
    }
}
