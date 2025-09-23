use igloo_interface::{Component, ComponentType};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Request a Query from the Query Engine
#[allow(dead_code)]
pub enum QueryEngineRequest {
    /// One shot queries are intended for operations that don't happen very often
    /// Every execution requires a full device tree lookup
    ExecuteOneShot(Query<OneShotQuery>),

    /// Request the registration of a persistent query
    /// Persistent queries are optimized by
    RegisterPersistent(Query<PersistentQuery>),

    /// After registering a PersistentQuery::Set(id, types)
    /// You can call this function with the same IDs and components of the same types
    /// To execute it
    CallSetPersistent(Uuid, Vec<Component>),
}

/// This is send over the broadcast channel whenever a OneShotQuery::Get or Avg is executed
#[allow(dead_code)]
pub enum OneShotQueryResult {
    Get(Uuid, Vec<OneShotGetQueryResult>),
    Avg(Uuid, Vec<Component>),
}

#[allow(dead_code)]
pub struct OneShotGetQueryResult {
    pub device_name: String,
    pub device_id: Uuid,
    pub entity: String,
    pub values: Vec<Component>,
}

#[allow(dead_code)]
pub struct PersistentQueryUpdate {}

#[allow(dead_code)]
pub struct Query<T> {
    kind: T,
    filter: QueryFilter,
    area: Area,
    limit: Option<usize>,
}

#[allow(dead_code)]
pub enum OneShotQuery {
    Set(Vec<Component>),
    /// Returns the value of every component that matches this type
    /// The Query Engine will send out the result over the broadcast channel
    /// using the UUID you provide here
    Get(Uuid, Vec<ComponentType>),
    /// Returns the average of each of the components that match this type
    /// The Query Engine will send out the result over the broadcast channel
    /// using the UUID you provide here
    Avg(Uuid, Vec<ComponentType>),
    /// Gets a snapshot of the entire device tree
    /// The Query Engine will send out the result over the broadcast channel
    /// using the UUID you provide here
    Snapshot(Uuid),
}

#[allow(dead_code)]
pub enum PersistentQuery {
    /// This simply specifies what you will be giving when you send QueryEngineRequest::CallSetPersistent
    /// The UUID between this registration and your calls must match
    Set(Uuid, Vec<ComponentType>),
    /// Any time the components change, the query engine will send the result over this mpsc channel
    /// In most cases, you should be using Avg
    Get(mpsc::Sender<PersistentQueryUpdate>, Vec<ComponentType>),
    /// Any time the components change, the query engine will send the result over this mpsc channel
    /// This is additionally optimized by only tracking component differences
    Avg(mpsc::Sender<PersistentQueryUpdate>, Vec<ComponentType>),
}

#[allow(dead_code)]
pub enum Area {
    All,
    /// zone ID
    Zone(String),
    /// device name
    Device(String),
    /// device name, entity name
    Entity(String, String),
}

#[allow(dead_code)]
pub enum QueryFilter {
    With(ComponentType),
    Without(ComponentType),
    And(Box<(QueryFilter, QueryFilter)>),
    Or(Box<(QueryFilter, QueryFilter)>),
    Condition(ComponentType, Operator, Component),
    NestedCondition(Vec<PathSegment>, Operator, Component),
}

#[allow(dead_code)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

#[allow(dead_code)]
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
            area: Area::All,
            limit: None,
        };

        // get the first Dimmer component that exists on an entity that also contains a Light component
        let expected = Query {
            kind: Get(Uuid::now_v7(), vec![ComponentType::Dimmer]),
            filter: With(ComponentType::Light),
            area: Area::All,
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
            area: Area::All,
            limit: None,
        };

        // get all Dimmer components that exist on entities without a Light component
        let expected = Query {
            kind: Get(Uuid::now_v7(), vec![ComponentType::Dimmer]),
            filter: Without(ComponentType::Light),
            area: Area::All,
            limit: None,
        };

        // set all Dimmer components to 155 that exist on entities that also have a Light component
        let expected = Query {
            kind: Set(vec![Component::Dimmer(Dimmer(155))]),
            filter: With(ComponentType::Light),
            area: Area::All,
            limit: None,
        };
    }
}
