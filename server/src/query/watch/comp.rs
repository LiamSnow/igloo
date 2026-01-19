//! Watches the values of components
//!
//! # Core Idea
//! Keep track of a matched set, check very minimal things on component_set (hot-path)
//!
//! First, find initial match set. Then subscribe to events than
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
//!  - value_filter, device/entity.last_update, entity_count :: these would cause expansion/contraction spam which actually makes it slower - instead they just get checked before every dispatch
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
            estimate_entity_count, for_each_entity, passes_device_last_update, passes_entity_count,
            passes_entity_id_filter, passes_entity_last_update, passes_group_filter,
            passes_id_filter, passes_owner_filter, passes_value_filter,
        },
        watch::{dispatch::WatchHandler, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Entity, Extension, Group},
};
use igloo_interface::{
    Aggregator, Component, ComponentType,
    id::{DeviceID, EntityIndex, ExtensionIndex, GroupID},
    query::{ComponentQuery, DeviceGroupFilter, TypeFilter, WatchUpdate as U, check::QueryError},
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
    time::Instant,
};

pub struct ComponentWatcher {
    pub client_id: usize,
    pub query_id: usize,
    pub query: ComponentQuery,
    watcher_id: usize,
    matched: FxHashMap<DeviceID, DeviceMatch>,
}

struct DeviceMatch {
    device_id: DeviceID,
    watcher_id: usize,
    entities: FxHashMap<EntityIndex, EntityMatch>,
    owner_ref: Option<ExtensionIndex>,
}

struct EntityMatch {
    device_id: DeviceID,
    entity_index: EntityIndex,
    watcher_id: usize,
}

impl ComponentWatcher {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        query_id: usize,
        watcher_id: usize,
        client_id: usize,
        query: ComponentQuery,
    ) -> Result<Self, QueryError> {
        // validate here, and reject before registering
        if query.limit.is_some() {
            return Err(QueryError::LimitOnWatcher);
        }

        if let Some(op) = query.post_op
            && Aggregator::new(query.component, op).is_none()
        {
            return Err(QueryError::InvalidAggregation(query.component, op));
        }

        if query.component.igloo_type().is_none() {
            return Err(QueryError::ComponentNoValue(query.component));
        }

        let mut me = Self {
            client_id,
            query_id,
            watcher_id: watcher_id,
            matched: HashMap::with_capacity_and_hasher(
                estimate_entity_count(tree, &query.device_filter, &query.entity_filter) + 5,
                FxBuildHasher,
            ),
            query,
        };

        // find initial match set
        let _ = for_each_entity(
            ctx,
            tree,
            &me.query.device_filter.clone(),
            &me.query.entity_filter.clone(),
            |device, entity| {
                // validate component has value
                me.expand_new_match(subs, device, entity);
                ControlFlow::Continue(())
            },
        );

        // new device added to group we care about
        match &me.query.device_filter.group {
            DeviceGroupFilter::Any => {
                // no filter
            }
            DeviceGroupFilter::In(gid) => {
                subs.group_device_added
                    .by_gid
                    .entry(*gid)
                    .or_default()
                    .all
                    .push(watcher_id);

                subs.group_deleted
                    .by_gid
                    .entry(*gid)
                    .or_insert_with(|| Vec::with_capacity(2))
                    .push(me.watcher_id);
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    subs.group_device_added
                        .by_gid
                        .entry(*gid)
                        .or_default()
                        .all
                        .push(watcher_id);

                    subs.group_deleted
                        .by_gid
                        .entry(*gid)
                        .or_insert_with(|| Vec::with_capacity(2))
                        .push(me.watcher_id);
                }
            }
        }

        // listen to component put of CTs we care about
        // listen to all types, can cause expansion (With, And, Or) OR contraction (Without, Not, And)
        let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
        if let Some(filter) = &me.query.entity_filter.type_filter {
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

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        // clean up matches
        let dids: Vec<DeviceID> = self.matched.keys().cloned().collect();
        for did in dids {
            self.contract_device(subs, did);
        }

        // clean up expansion subs
        match &self.query.device_filter.group {
            DeviceGroupFilter::In(gid) => {
                if let Some(group_sub) = subs.group_device_added.by_gid.get_mut(gid) {
                    group_sub.all.retain(|o| *o != self.watcher_id);
                }
                if let Some(watchers) = subs.group_deleted.by_gid.get_mut(gid) {
                    watchers.retain(|o| *o != self.watcher_id);
                }
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    if let Some(group_sub) = subs.group_device_added.by_gid.get_mut(gid) {
                        group_sub.all.retain(|o| *o != self.watcher_id);
                    }
                    if let Some(watchers) = subs.group_deleted.by_gid.get_mut(gid) {
                        watchers.retain(|o| *o != self.watcher_id);
                    }
                }
            }
            DeviceGroupFilter::Any => {}
        }

        // clean up component_put subscriptions
        let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
        if let Some(filter) = &self.query.entity_filter.type_filter {
            collect_all_types_in_tf(filter, &mut care);
        }
        care.insert(self.query.component);

        for ct in care {
            if let Some(watchers) = subs.component_put.by_comp_type.get_mut(&ct) {
                watchers.retain(|o| *o != self.watcher_id);
            }
        }
    }

    /// Expand matching set with a new device/entity pair
    /// WARN: this has no checks, use try_expand_entity if you are unsure to expand or not
    fn expand_new_match(&mut self, subs: &mut TreeSubscribers, device: &Device, entity: &Entity) {
        if let Some(device_match) = self.matched.get_mut(device.id()) {
            // device already matched, just add entity
            device_match.add_entity(*entity.index(), subs, self.query.component);
        } else {
            // new device match
            let mut device_match =
                DeviceMatch::new(*device.id(), self.watcher_id, subs, device, &self.query);
            device_match.add_entity(*entity.index(), subs, self.query.component);
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
        // skip all runtime checks (last update, value filter, entity count)
        // and only need to check type_filter against entity
        if !passes_id_filter(device, &self.query.device_filter.id)
            || !passes_group_filter(device, &self.query.device_filter.group, tree)
            || !passes_owner_filter(device, &self.query.device_filter.owner)
        {
            return false;
        }

        if let Some(filter) = &self.query.entity_filter.type_filter
            && !entity.matches(filter)
        {
            return false;
        }

        passes_entity_id_filter(ctx, entity, &self.query.entity_filter.id)
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

    fn contract_device(&mut self, subs: &mut TreeSubscribers, did: DeviceID) {
        if let Some(device_match) = self.matched.remove(&did) {
            device_match.cleanup(subs, &self.query);
        }
    }

    fn contract_entity(
        &mut self,
        subs: &mut TreeSubscribers,
        did: DeviceID,
        entity_index: EntityIndex,
    ) {
        let Some(device_match) = self.matched.get_mut(&did) else {
            return;
        };

        let is_empty = device_match.remove_entity(entity_index, subs, self.query.component);

        // remove device if no entities left
        if is_empty && let Some(device_match) = self.matched.remove(&did) {
            device_match.cleanup(subs, &self.query);
        }
    }
}

impl DeviceMatch {
    fn new(
        device_id: DeviceID,
        watcher_id: usize,
        subs: &mut TreeSubscribers,
        device: &Device,
        query: &ComponentQuery,
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
        match &query.device_filter.group {
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
            watcher_id: watcher_id,
            entities: HashMap::with_capacity_and_hasher(4, FxBuildHasher),
            owner_ref: device.owner_ref(),
        }
    }

    fn add_entity(
        &mut self,
        entity_index: EntityIndex,
        subs: &mut TreeSubscribers,
        component_type: ComponentType,
    ) {
        let entity_match = EntityMatch::new(
            self.device_id,
            entity_index,
            self.watcher_id,
            subs,
            component_type,
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

    fn cleanup(&self, subs: &mut TreeSubscribers, query: &ComponentQuery) {
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
        match &query.device_filter.group {
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
            watcher_id: watcher_id,
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

impl WatchHandler for ComponentWatcher {
    fn on_component_set(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        _subs: &mut TreeSubscribers,
        tree: &DeviceTree,
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

        let entity = &device.entities()[entity_index.0];

        if !passes_entity_last_update(entity, &self.query.entity_filter.last_update) {
            return Ok(());
        }

        if !passes_entity_count(device, &self.query.device_filter.entity_count) {
            return Ok(());
        }

        if !passes_device_last_update(
            &Instant::now(),
            device,
            &self.query.device_filter.last_update,
        ) {
            return Ok(());
        }

        if let Some(filter) = &self.query.entity_filter.value_filter
            && !passes_value_filter(entity, filter)
        {
            return Ok(());
        }

        let result = match self.query.post_op {
            Some(op) => {
                let Some(mut agg) = Aggregator::new(self.query.component, op) else {
                    // error is caught during registration, ignore now
                    return Ok(());
                };

                let _ = for_each_entity(
                    ctx,
                    tree,
                    &self.query.device_filter,
                    &self.query.entity_filter,
                    |_, entity| {
                        if let Some(comp) = entity.get(self.query.component) {
                            agg.push(comp)?;
                        }
                        ControlFlow::Continue(())
                    },
                );

                let Some(res) = agg.finish() else {
                    return Ok(());
                };

                U::Aggregate(res)
            }

            None if self.query.include_parents => {
                let Some(iv) = comp.to_igloo_value() else {
                    // error is caught during registration, ignore now
                    return Ok(());
                };
                U::ComponentValueWithParents(*device.id(), entity.id().clone(), iv)
            }

            _ => {
                let Some(iv) = comp.to_igloo_value() else {
                    // error is caught during registration, ignore now
                    return Ok(());
                };
                U::ComponentValue(iv)
            }
        };

        cm.send(
            self.client_id,
            IglooResponse::WatchUpdate {
                query_id: self.query_id,
                result,
            },
        )
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

        if let Some(filter) = &self.query.entity_filter.type_filter
            && !entity.matches(filter)
        {
            self.contract_entity(subs, *device.id(), entity_index);
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
        _cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.contract_device(subs, *device.id());
        Ok(())
    }

    fn on_group_deleted(
        &mut self,
        _cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        _gid: &GroupID,
    ) -> Result<(), IglooError> {
        let dids: Vec<DeviceID> = self.matched.keys().cloned().collect();
        for did in dids {
            let Ok(device) = tree.device(&did) else {
                continue;
            };
            if !passes_group_filter(device, &self.query.device_filter.group, tree) {
                self.contract_device(subs, did);
            }
        }

        Ok(())
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
        _cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        _group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        // recheck filter bc it may apply if ::InAny
        if !passes_group_filter(device, &self.query.device_filter.group, tree) {
            self.contract_device(subs, *device.id());
        }
        Ok(())
    }

    fn on_ext_detached(
        &mut self,
        _cm: &mut ClientManager,
        _ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        let owner_ref = Some(*ext.index());

        let dids: Vec<DeviceID> = self
            .matched
            .iter()
            .filter(|(_, dm)| dm.owner_ref == owner_ref)
            .map(|(did, _)| *did)
            .collect();

        for did in dids {
            self.contract_device(subs, did);
        }

        Ok(())
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
