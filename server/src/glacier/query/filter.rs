use std::time::Duration;

use crate::glacier::{
    query::QueryEngine,
    tree::{Device, DeviceTree, Entity, Floe, Group, HasComponent},
};
use igloo_interface::query::{
    DeviceFilter as D, EntityFilter as E, FloeFilter as F, GroupFilter as G,
};

pub trait EntityFilterMatcher {
    fn matches(&self, engine: &mut QueryEngine, entity: &Entity) -> bool;
}

impl EntityFilterMatcher for E {
    fn matches(&self, engine: &mut QueryEngine, entity: &Entity) -> bool {
        match self {
            E::NameEquals(name) => entity.name() == name,

            E::NameMatches(pattern) => {
                let glob = engine.glob(pattern);
                glob.is_match(entity.name())
            }

            E::UpdatedWithinSeconds(secs) => {
                entity.last_updated() >= &(engine.query_time - Duration::from_secs(*secs))
            }

            E::ComponentCount(op, count) => op.eval_usize(entity.num_comps(), *count),

            E::Condition(op, component) => {
                entity
                    .get(component.get_type())
                    .and_then(|c| {
                        let a = c.to_igloo_value()?; // TODO return or log error
                        let b = component.to_igloo_value()?;
                        Some(op.eval(&a, &b).unwrap_or(false))
                    })
                    .unwrap_or(false)
            }

            E::Has(comp_type) => entity.has(*comp_type),

            E::HasAll(types) => entity.has_all(types),

            E::HasAny(types) => entity.has_any(types),

            E::All(filters) => filters.iter().all(|f| f.matches(engine, entity)),

            E::Any(filters) => filters.iter().any(|f| f.matches(engine, entity)),

            E::Not(inner) => !inner.matches(engine, entity),
        }
    }
}

pub trait DeviceFilterMatcher {
    fn matches(&self, engine: &mut QueryEngine, device: &Device) -> bool;
}

impl DeviceFilterMatcher for D {
    fn matches(&self, engine: &mut QueryEngine, device: &Device) -> bool {
        match self {
            D::Id(did) => device.id() == did,

            D::Ids(dids) => dids.contains(device.id()),

            D::NameEquals(name) => device.name() == name,

            D::NameMatches(pattern) => {
                let glob = engine.glob(pattern);
                glob.is_match(device.name())
            }

            D::UpdatedWithinSeconds(secs) => {
                device.last_updated() >= &(engine.query_time - Duration::from_secs(*secs))
            }

            D::EntityCount(op, count) => op.eval_usize(device.num_entities(), *count),

            D::HasAll(types) => device.has_all(types),

            D::HasEntity(entity_filter) => device
                .entities()
                .iter()
                .any(|entity| entity_filter.matches(engine, entity)),

            D::AllEntities(entity_filter) => {
                !device.entities().is_empty()
                    && device
                        .entities()
                        .iter()
                        .all(|entity| entity_filter.matches(engine, entity))
            }

            D::All(filters) => filters.iter().all(|f| f.matches(engine, device)),

            D::Any(filters) => filters.iter().any(|f| f.matches(engine, device)),

            D::Not(inner) => !inner.matches(engine, device),
        }
    }
}

pub trait GroupFilterMatcher {
    fn matches(&self, engine: &mut QueryEngine, tree: &DeviceTree, group: &Group) -> bool;
}

impl GroupFilterMatcher for G {
    fn matches(&self, engine: &mut QueryEngine, tree: &DeviceTree, group: &Group) -> bool {
        match self {
            G::Id(gid) => group.id() == gid,

            G::Ids(gids) => gids.contains(group.id()),

            G::NameEquals(name) => group.name() == name,

            G::NameMatches(pattern) => {
                let glob = engine.glob(pattern);
                glob.is_match(group.name())
            }

            G::DeviceCount(op, count) => op.eval_usize(group.devices().len(), *count),

            G::HasDevice(device_filter) => group.devices().iter().any(|did| {
                tree.device(did)
                    .map(|device| device_filter.matches(engine, device))
                    .unwrap_or(false)
            }),

            G::AllDevices(device_filter) => {
                let mut found_any = false;
                group
                    .devices()
                    .iter()
                    .filter_map(|did| tree.device(did).ok())
                    .all(|device| {
                        found_any = true;
                        device_filter.matches(engine, device)
                    })
                    && found_any
            }

            G::All(filters) => filters.iter().all(|f| f.matches(engine, tree, group)),

            G::Any(filters) => filters.iter().any(|f| f.matches(engine, tree, group)),

            G::Not(inner) => !inner.matches(engine, tree, group),
        }
    }
}

pub trait FloeFilterMatcher {
    fn matches(&self, engine: &mut QueryEngine, tree: &DeviceTree, floe: &Floe) -> bool;
}

impl FloeFilterMatcher for F {
    fn matches(&self, engine: &mut QueryEngine, tree: &DeviceTree, floe: &Floe) -> bool {
        match self {
            F::Id(fid) => floe.id() == fid,

            F::Ids(fids) => fids.contains(floe.id()),

            F::IdMatches(pattern) => {
                let glob = engine.glob(pattern);
                glob.is_match(floe.id().0.clone())
            }

            F::DeviceCount(op, count) => op.eval_usize(floe.num_device(), *count),

            F::HasDevice(device_filter) => floe.devices().iter().any(|did| {
                tree.device(did)
                    .map(|device| device_filter.matches(engine, device))
                    .unwrap_or(false)
            }),

            F::AllDevices(device_filter) => {
                let mut found_any = false;
                floe.devices()
                    .iter()
                    .filter_map(|did| tree.device(did).ok())
                    .all(|device| {
                        found_any = true;
                        device_filter.matches(engine, device)
                    })
                    && found_any
            }

            F::All(filters) => filters.iter().all(|f| f.matches(engine, tree, floe)),

            F::Any(filters) => filters.iter().any(|f| f.matches(engine, tree, floe)),

            F::Not(inner) => !inner.matches(engine, tree, floe),
        }
    }
}
