//! Watches the values of component(s)
//!
//! # Core Idea
//! Keep track of a matched set, check very minimal things on component_set (hot-path)
//!
//! First, find initial match set. Then subscribe to events that
//! can cause us to expand or contract that match set.
//!
//! # Expansion Events
//!  - component_put that now satifies type_filter
//!  - device added to group in device_filter.group
//!
//! Shouldn't there be way more expansion events?
//!  - device_created, group_craeted :: we assume queries can't know IDs before they're created
//!  - device_renamed, group_renamed :: name filters only exist for entities (which cant be renamed)
//!  - ext_attached :: it's devices don't have entities/components yet
//!  - entity_registered :: doesn't have components yet (can't pass query.component filter)
//!
//! # Contraction Events
//!  - component_put that now doesn't satify type_filter
//!  - device_deleted
//!  - group_deleted|group_device_removed AND device now doesn't satify device_filter.group (in cases of ::InAny, it still may be valid)
//!  - ext_detached :: we know we can recieve component updates from detached devices

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryContext,
        iter::{
            passes_entity_id_filter, passes_group_filter, passes_id_filter, passes_owner_filter,
            watch::{estimate_entity_count, for_each_entity},
        },
        watch::{WatcherID, dispatch::TreeEventResponder, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Entity, Extension, Group},
};
use igloo_interface::{
    Aggregator, Component, ComponentType,
    id::{DeviceID, EntityIndex, ExtensionIndex, GenerationalID, GroupID},
    query::{
        DeviceGroupFilter, TypeFilter, WatchComponentQuery, WatchUpdate as U, check::QueryError,
    },
    types::IglooValue,
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
};

pub struct ComponentWatcher {
    pub id: WatcherID,
    pub subs: Vec<(usize, usize)>,
    pub query: WatchComponentQuery,
    matched: FxHashMap<DeviceID, DeviceMatch>,
}

#[derive(Debug)]
struct DeviceMatch {
    device_id: DeviceID,
    watcher_id: usize,
    entities: FxHashMap<EntityIndex, EntityMatch>,
    owner_ref: Option<ExtensionIndex>,
}

#[derive(Debug)]
struct EntityMatch {
    device_id: DeviceID,
    entity_index: EntityIndex,
    watcher_id: usize,
    value: Component,
}

fn subscribe_to_group_events(subs: &mut TreeSubscribers, gid: GroupID, watcher_id: usize) {
    subs.group_device_added
        .by_gid
        .entry(gid)
        .or_default()
        .all
        .push(watcher_id);

    subs.group_deleted
        .by_gid
        .entry(gid)
        .or_insert_with(|| Vec::with_capacity(2))
        .push(watcher_id);
}

fn unsubscribe_from_group_events(subs: &mut TreeSubscribers, gid: GroupID, watcher_id: usize) {
    if let Some(group_sub) = subs.group_device_added.by_gid.get_mut(&gid) {
        group_sub.all.retain(|o| *o != watcher_id);
    }
    if let Some(watchers) = subs.group_deleted.by_gid.get_mut(&gid) {
        watchers.retain(|o| *o != watcher_id);
    }
}

impl ComponentWatcher {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    pub fn register(
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        watcher_id: usize,
        query: WatchComponentQuery,
    ) -> Result<Self, QueryError> {
        // validate here, and reject before registering
        if let Some(op) = query.post_op
            && Aggregator::new(query.component, op).is_none()
        {
            return Err(QueryError::InvalidAggregation(query.component, op));
        }

        if query.component.igloo_type().is_none() {
            return Err(QueryError::ComponentNoValue(query.component));
        }

        let mut me = Self {
            id: watcher_id,
            subs: Vec::with_capacity(3),
            matched: HashMap::with_capacity_and_hasher(
                estimate_entity_count(tree, &query) + 5,
                FxBuildHasher,
            ),
            query,
        };

        // find initial match set
        let _ = for_each_entity(ctx, tree, me.query.clone(), |device, entity| {
            me.expand_new_match(subs, device, entity);
            ControlFlow::Continue(())
        });

        // new device added to group we care about
        match &me.query.group {
            DeviceGroupFilter::Any => {
                // no filter
            }
            DeviceGroupFilter::In(gid) => {
                subscribe_to_group_events(subs, *gid, watcher_id);
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    subscribe_to_group_events(subs, *gid, watcher_id);
                }
            }
        }

        // listen to component put of CTs we care about
        // listen to all types, can cause expansion (With, And, Or) OR contraction (Without, Not, And)
        let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
        if let Some(filter) = &me.query.type_filter {
            collect_all_types_in_tf(filter, &mut care);
        }
        care.insert(me.query.component);
        for ct in care {
            subs.component_put
                .by_comp_type
                .entry(ct)
                .or_default()
                .push(watcher_id);
        }

        Ok(me)
    }

    pub fn on_sub(
        &mut self,
        cm: &mut ClientManager,
        client_id: usize,
        query_id: usize,
    ) -> Result<(), IglooError> {
        self.subs.push((client_id, query_id));

        if self.query.post_op.is_some() {
            if let Some(value) = self.compute_aggregate() {
                cm.send(
                    client_id,
                    IglooResponse::WatchUpdate {
                        query_id,
                        value: U::ComponentAggregate(value),
                    },
                )?;
            }
        } else {
            for dm in self.matched.values() {
                for em in dm.entities.values() {
                    let Some(iv) = em.value.to_igloo_value() else {
                        continue;
                    };
                    cm.send(
                        client_id,
                        IglooResponse::WatchUpdate {
                            query_id,
                            value: U::ComponentValue(em.device_id, em.entity_index, iv),
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    fn compute_aggregate(&self) -> Option<IglooValue> {
        let op = self.query.post_op?;
        let mut agg = Aggregator::new(self.query.component, op)?;

        println!("<em>");
        'top: for dm in self.matched.values() {
            for em in dm.entities.values() {
                println!(
                    " - dev={} ent={} value={:?}",
                    em.device_id.take(),
                    em.entity_index.0,
                    em.value
                );
                // sometimes we can exit early
                // ex. if we are checking binary ::Any
                // and when encounter an `true` value
                if agg.push(&em.value).is_break() {
                    break 'top;
                }
            }
        }
        println!("<em/>");

        agg.finish()
    }

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        // clean up matches
        let dids: Vec<DeviceID> = self.matched.keys().cloned().collect();
        for did in dids {
            if let Some(device_match) = self.matched.remove(&did) {
                device_match.cleanup(subs, &self.query);
            }
        }

        // clean up expansion subs
        match &self.query.group {
            DeviceGroupFilter::In(gid) => {
                unsubscribe_from_group_events(subs, *gid, self.id);
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    unsubscribe_from_group_events(subs, *gid, self.id);
                }
            }
            DeviceGroupFilter::Any => {}
        }

        // clean up component_put subscriptions
        let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
        if let Some(filter) = &self.query.type_filter {
            collect_all_types_in_tf(filter, &mut care);
        }
        care.insert(self.query.component);

        for ct in care {
            if let Some(watchers) = subs.component_put.by_comp_type.get_mut(&ct) {
                watchers.retain(|o| *o != self.id);
            }
        }
    }

    /// Expand matching set with a new device/entity pair
    /// WARN: this has no checks, use try_expand_entity if you are unsure to expand or not
    fn expand_new_match(&mut self, subs: &mut TreeSubscribers, device: &Device, entity: &Entity) {
        let Some(comp) = entity.get(self.query.component) else {
            return;
        };

        if let Some(device_match) = self.matched.get_mut(device.id()) {
            // device already matched, just add entity
            device_match.add_entity(*entity.index(), subs, self.query.component, comp.clone());
        } else {
            // new device match
            let mut device_match =
                DeviceMatch::new(*device.id(), self.id, subs, device, &self.query);
            device_match.add_entity(*entity.index(), subs, self.query.component, comp.clone());
            self.matched.insert(*device.id(), device_match);
        }
    }

    pub fn matches(
        &mut self,
        ctx: &mut QueryContext,
        tree: &DeviceTree,
        device: &Device,
        entity: &Entity,
    ) -> bool {
        if !passes_id_filter(device, &self.query.device_id)
            || !passes_group_filter(device, &self.query.group, tree)
            || !passes_owner_filter(device, &self.query.owner)
        {
            return false;
        }

        if let Some(filter) = &self.query.type_filter
            && !entity.matches(filter)
        {
            return false;
        }

        passes_entity_id_filter(ctx, entity, &self.query.entity_id)
    }

    /// Expands matching set IF this is a valid target
    /// Returns if it was expanded
    fn try_expand_entity(
        &mut self,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
        entity: &Entity,
    ) -> bool {
        // check if already matched
        if let Some(device_match) = self.matched.get(device.id())
            && device_match.entities.contains_key(entity.index())
        {
            return false;
        }

        // check if entity has the component
        if !entity.has(self.query.component) {
            return false;
        }

        if self.matches(ctx, tree, device, entity) {
            self.expand_new_match(subs, device, entity);
            return true;
        }

        false
    }

    fn contract_device(
        &mut self,
        subs: &mut TreeSubscribers,
        cm: &mut ClientManager,
        did: DeviceID,
    ) -> Result<(), IglooError> {
        if let Some(device_match) = self.matched.remove(&did) {
            device_match.cleanup(subs, &self.query);
        }

        self.broadcast_aggregate_update(cm)?;
        Ok(())
    }

    fn contract_entity(
        &mut self,
        subs: &mut TreeSubscribers,
        cm: &mut ClientManager,
        did: DeviceID,
        entity_index: EntityIndex,
    ) -> Result<(), IglooError> {
        let Some(device_match) = self.matched.get_mut(&did) else {
            return Ok(());
        };

        let is_empty = device_match.remove_entity(entity_index, subs, self.query.component);

        // remove device if no entities left
        if is_empty {
            if let Some(device_match) = self.matched.remove(&did) {
                device_match.cleanup(subs, &self.query);
            }
        }

        self.broadcast_aggregate_update(cm)?;
        Ok(())
    }

    fn broadcast_aggregate_update(&self, cm: &mut ClientManager) -> Result<(), IglooError> {
        if self.query.post_op.is_some() {
            if let Some(value) = self.compute_aggregate() {
                for (client_id, query_id) in &self.subs {
                    cm.send(
                        *client_id,
                        IglooResponse::WatchUpdate {
                            query_id: *query_id,
                            value: U::ComponentAggregate(value.clone()),
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    fn contract_matching_devices<F>(
        &mut self,
        subs: &mut TreeSubscribers,
        cm: &mut ClientManager,
        filter_fn: F,
    ) -> Result<(), IglooError>
    where
        F: Fn(&DeviceID, &DeviceMatch) -> bool,
    {
        let dids: Vec<DeviceID> = self
            .matched
            .iter()
            .filter(|(did, dm)| filter_fn(did, dm))
            .map(|(did, _)| *did)
            .collect();

        for did in dids {
            self.contract_device(subs, cm, did)?;
        }

        Ok(())
    }
}

impl DeviceMatch {
    fn new(
        device_id: DeviceID,
        watcher_id: usize,
        subs: &mut TreeSubscribers,
        device: &Device,
        query: &WatchComponentQuery,
    ) -> Self {
        // subscribe to device-level events that can cause contraction

        // match deleted
        subs.device_deleted
            .by_did
            .entry(device_id)
            .or_insert_with(|| Vec::with_capacity(2))
            .push(watcher_id);

        // match detached
        if let Some(xindex) = device.owner_ref() {
            subs.ext_detached
                .by_xindex
                .entry(xindex)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(watcher_id);
        }

        // group deleted | this device removed from group
        match &query.group {
            DeviceGroupFilter::Any => {
                // no filter
            }
            DeviceGroupFilter::In(gid) => {
                // always causes a contraction
                subs.group_device_removed
                    .by_gid
                    .entry(*gid)
                    .or_default()
                    .by_did
                    .entry(device_id)
                    .or_insert_with(|| Vec::with_capacity(2))
                    .push(watcher_id);
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                // in any, may cause contraction
                // in all, always causes contraction
                for gid in gids {
                    subs.group_device_removed
                        .by_gid
                        .entry(*gid)
                        .or_default()
                        .by_did
                        .entry(device_id)
                        .or_insert_with(|| Vec::with_capacity(2))
                        .push(watcher_id);
                }
            }
        }

        Self {
            device_id,
            watcher_id,
            entities: HashMap::with_capacity_and_hasher(4, FxBuildHasher),
            owner_ref: device.owner_ref(),
        }
    }

    fn add_entity(
        &mut self,
        entity_index: EntityIndex,
        subs: &mut TreeSubscribers,
        component_type: ComponentType,
        value: Component,
    ) {
        let entity_match = EntityMatch::new(
            self.device_id,
            entity_index,
            self.watcher_id,
            subs,
            component_type,
            value,
        );
        self.entities.insert(entity_index, entity_match);
    }

    fn remove_entity(
        &mut self,
        entity_index: EntityIndex,
        subs: &mut TreeSubscribers,
        component_type: ComponentType,
    ) -> bool {
        if let Some(entity_match) = self.entities.remove(&entity_index) {
            entity_match.cleanup(subs, component_type);
        }
        self.entities.is_empty()
    }

    fn cleanup(&self, subs: &mut TreeSubscribers, query: &WatchComponentQuery) {
        // cleanup all entities
        for entity_match in self.entities.values() {
            entity_match.cleanup(subs, query.component);
        }

        // cleanup device-level subscriptions
        if let Some(watchers) = subs.device_deleted.by_did.get_mut(&self.device_id) {
            watchers.retain(|o| *o != self.watcher_id);
            if watchers.is_empty() {
                subs.device_deleted.by_did.remove(&self.device_id);
            }
        }

        // cleanup ext_detached (we don't know the xindex, so scan all)
        if let Some(xindex) = self.owner_ref
            && let Some(watchers) = subs.ext_detached.by_xindex.get_mut(&xindex)
        {
            watchers.retain(|o| *o != self.watcher_id);
        }

        // cleanup group_device_removed
        match &query.group {
            DeviceGroupFilter::Any => {}
            DeviceGroupFilter::In(gid) => {
                if let Some(group_sub) = subs.group_device_removed.by_gid.get_mut(gid)
                    && let Some(watchers) = group_sub.by_did.get_mut(&self.device_id)
                {
                    watchers.retain(|o| *o != self.watcher_id);
                    if watchers.is_empty() {
                        group_sub.by_did.remove(&self.device_id);
                    }
                }
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    if let Some(group_sub) = subs.group_device_removed.by_gid.get_mut(gid)
                        && let Some(watchers) = group_sub.by_did.get_mut(&self.device_id)
                    {
                        watchers.retain(|o| *o != self.watcher_id);
                        if watchers.is_empty() {
                            group_sub.by_did.remove(&self.device_id);
                        }
                    }
                }
            }
        }
    }
}

impl EntityMatch {
    fn new(
        device_id: DeviceID,
        entity_index: EntityIndex,
        watcher_id: usize,
        subs: &mut TreeSubscribers,
        component_type: ComponentType,
        value: Component,
    ) -> Self {
        // subscribe to triggering event
        subs.component_set
            .0
            .entry((device_id, entity_index, component_type))
            .or_insert_with(|| Vec::with_capacity(2))
            .push(watcher_id);

        Self {
            device_id,
            entity_index,
            watcher_id,
            value,
        }
    }

    fn cleanup(&self, subs: &mut TreeSubscribers, component_type: ComponentType) {
        let key = (self.device_id, self.entity_index, component_type);
        if let Some(watchers) = subs.component_set.0.get_mut(&key) {
            watchers.retain(|o| *o != self.watcher_id);
            if watchers.is_empty() {
                subs.component_set.0.remove(&key);
            }
        }
    }
}

pub fn collect_all_types_in_tf(filter: &TypeFilter, set: &mut FxHashSet<ComponentType>) {
    use TypeFilter::*;
    match filter {
        With(t) | Without(t) => {
            set.insert(*t);
        }
        And(filters) | Or(filters) => {
            for filter in filters {
                collect_all_types_in_tf(filter, set);
            }
        }
        Not(filter) => {
            collect_all_types_in_tf(filter, set);
        }
    }
}

impl TreeEventResponder for ComponentWatcher {
    fn on_component_set(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        _comp_type: ComponentType,
        comp: &Component,
    ) -> Result<(), IglooError> {
        debug_assert!(
            self.matched
                .get(device.id())
                .and_then(|d| d.entities.get(&entity_index))
                .is_some(),
            "Received component_set for unmatched device/entity"
        );

        // no need to check comp_type bc only subscribed to events on our query.component

        if let Some(device_match) = self.matched.get_mut(device.id())
            && let Some(entity_match) = device_match.entities.get_mut(&entity_index)
        {
            entity_match.value = comp.clone();
        }

        let update = match self.query.post_op {
            Some(_op) => {
                let Some(value) = self.compute_aggregate() else {
                    return Ok(());
                };
                U::ComponentAggregate(value)
            }

            None => {
                let Some(iv) = comp.to_igloo_value() else {
                    // ignore bc it should have caught at registration
                    return Ok(());
                };
                U::ComponentValue(*device.id(), entity_index, iv)
            }
        };

        for (client_id, query_id) in &self.subs {
            cm.send(
                *client_id,
                IglooResponse::WatchUpdate {
                    query_id: *query_id,
                    value: update.clone(),
                },
            )?;
        }

        Ok(())
    }

    fn on_component_put(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        comp_type: ComponentType,
        comp: &Component,
    ) -> Result<(), IglooError> {
        let entity = &device.entities()[entity_index.0];

        if let Some(filter) = &self.query.type_filter
            && !entity.matches(filter)
        {
            self.contract_entity(subs, cm, *device.id(), entity_index)?;
        } else if self.try_expand_entity(ctx, subs, tree, device, entity)
            && comp_type == self.query.component
        {
            return self.on_component_set(
                cm,
                ctx,
                subs,
                tree,
                device,
                entity_index,
                comp_type,
                comp,
            );
        }

        Ok(())
    }

    fn on_device_deleted(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.contract_device(subs, cm, *device.id())
    }

    fn on_group_deleted(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        _gid: &GroupID,
    ) -> Result<(), IglooError> {
        let group_filter = self.query.group.clone();
        self.contract_matching_devices(subs, cm, |did, _| {
            tree.device(did)
                .map(|device| !passes_group_filter(device, &group_filter, tree))
                .unwrap_or(false)
        })
    }

    fn on_group_device_added(
        &mut self,
        _cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        _group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        for entity in device.entities() {
            self.try_expand_entity(ctx, subs, tree, device, entity);
        }
        Ok(())
    }

    fn on_group_device_removed(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        _group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        // recheck filter bc it may apply if ::InAny
        if !passes_group_filter(device, &self.query.group, tree) {
            self.contract_device(subs, cm, *device.id())?;
        }
        Ok(())
    }

    fn on_ext_detached(
        &mut self,
        cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        let owner_ref = Some(*ext.index());
        self.contract_matching_devices(subs, cm, |_, dm| dm.owner_ref == owner_ref)
    }

    fn on_device_created(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "ComponentWatcher should never receive device_created events"
        );
        Ok(())
    }

    fn on_device_renamed(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "ComponentWatcher should never receive device_renamed events"
        );
        Ok(())
    }

    fn on_entity_registered(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
        _: EntityIndex,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "ComponentWatcher should never receive entity_registered events"
        );
        Ok(())
    }

    fn on_group_created(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Group,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "ComponentWatcher should never receive group_created events"
        );
        Ok(())
    }

    fn on_group_renamed(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Group,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "ComponentWatcher should never receive group_renamed events"
        );
        Ok(())
    }

    fn on_ext_attached(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Extension,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "ComponentWatcher should never receive ext_attached events"
        );
        Ok(())
    }
}
