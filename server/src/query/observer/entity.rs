//! Observes entity registration and component puts on entities
//!
//! # Core Idea
//! Keep track of a matched set, check very minimal things on entity_registered/component_put (hot-path)
//!
//! First, find initial match set. Then subscribe to events that
//! can cause us to expand or contract that match set.
//!
//! # Expansion Events
//!  - device_created that satisfies device_filter
//!  - ext_attached with devices satisfying device_filter
//!  - device added to group in device_filter.group
//!  - component_put that now satisfies type_filter
//!  - entity_registered (for ObserveComponentPut, expands device's entity set)
//!
//! Why do we include device_created and ext_attached?
//!  - Unlike ComponentObserver, we care about entity_registered which happens AFTER device creation
//!  - For ObserveRegistered: we want to catch future entity_registered events on new devices
//!  - For ObserveComponentPut: same, plus we want to catch component_put on those future entities
//!
//! Expansion events we DON'T need:
//!  - device_renamed, group_renamed :: name filters only exist for entities (which can't be renamed)
//!  - group_created :: we assume queries can't know IDs before they're created
//!  - value_filter, device/entity.last_update, entity_count :: these would cause expansion/contraction spam - instead they just get checked before every dispatch
//!
//! # Contraction Events
//!  - component_put that now doesn't satisfy type_filter
//!  - device_deleted
//!  - group_deleted|group_device_removed AND device now doesn't satisfy device_filter.group
//!  - ext_detached
//!
//! -- Probably has a bit of repeated code between here and component.rs,
//! -- but logic is slightly different. Maybe at some point combine.

use crate::{
    core::{ClientManager, IglooError, IglooResponse},
    query::{
        QueryContext,
        iter::{
            passes_device_last_update, passes_entity_count, passes_entity_id_filter,
            passes_entity_last_update, passes_group_filter, passes_id_filter, passes_owner_filter,
            passes_value_filter,
        },
        observer::{dispatch::ObserverHandler, subscriber::TreeSubscribers},
    },
    tree::{Device, DeviceTree, Entity, Extension, Group},
};
use igloo_interface::{
    Component, ComponentType,
    id::{DeviceID, EntityIndex, ExtensionIndex, GroupID},
    query::{
        DeviceGroupFilter, EntityAction, EntityQuery, ObserverUpdate as U, TypeFilter,
        check::QueryError,
    },
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

pub struct EntityObserver {
    pub client_id: usize,
    pub query_id: usize,
    pub query: EntityQuery,
    observer_id: usize,
    matched: FxHashMap<DeviceID, DeviceMatch>,
}

struct DeviceMatch {
    device_id: DeviceID,
    observer_id: usize,
    entities: FxHashMap<EntityIndex, EntityMatch>,
    owner_ref: Option<ExtensionIndex>,
}

struct EntityMatch {
    device_id: DeviceID,
    entity_index: EntityIndex,
    observer_id: usize,
}

impl EntityObserver {
    /// Subscriptions are registered here
    /// Then the subsequent `on_*` below will trigger
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        query_id: usize,
        observer_id: usize,
        client_id: usize,
        query: EntityQuery,
    ) -> Result<Self, QueryError> {
        // validate here, and reject before registering
        if query.limit.is_some() {
            return Err(QueryError::LimitOnObserver);
        }

        let mut me = Self {
            client_id,
            query_id,
            observer_id,
            matched: HashMap::with_capacity_and_hasher(32, FxBuildHasher),
            query,
        };

        // find initial match set
        use crate::query::iter::for_each_entity;
        let _ = for_each_entity(
            ctx,
            tree,
            &me.query.device_filter.clone(),
            &me.query.entity_filter.clone(),
            |device, entity| {
                me.expand_new_match(subs, device, entity);
                std::ops::ControlFlow::Continue(())
            },
        );

        // subscribe to device-level expansion events
        subs.device_created.all.push(observer_id);
        subs.ext_attached.all.push(observer_id);

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
                    .push(observer_id);

                subs.group_deleted
                    .by_gid
                    .entry(*gid)
                    .or_insert_with(|| Vec::with_capacity(2))
                    .push(observer_id);
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    subs.group_device_added
                        .by_gid
                        .entry(*gid)
                        .or_default()
                        .all
                        .push(observer_id);

                    subs.group_deleted
                        .by_gid
                        .entry(*gid)
                        .or_insert_with(|| Vec::with_capacity(2))
                        .push(observer_id);
                }
            }
        }

        // listen to component put of CTs we care about
        // listen to all types, can cause expansion (With, And, Or) OR contraction (Without, Not, And)
        if let Some(filter) = &me.query.entity_filter.type_filter {
            let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
            collect_all_types_in_tf(filter, &mut care);
            for ct in care {
                subs.component_put
                    .by_comp_type
                    .entry(ct)
                    .or_default()
                    .push(observer_id);
            }
        }

        Ok(me)
    }

    pub fn cleanup(&mut self, subs: &mut TreeSubscribers) {
        // clean up matches
        let dids: Vec<DeviceID> = self.matched.keys().cloned().collect();
        for did in dids {
            self.contract_device(subs, did);
        }

        // clean up device-level expansion subs
        subs.device_created.all.retain(|o| *o != self.observer_id);
        subs.ext_attached.all.retain(|o| *o != self.observer_id);

        match &self.query.device_filter.group {
            DeviceGroupFilter::In(gid) => {
                if let Some(group_sub) = subs.group_device_added.by_gid.get_mut(gid) {
                    group_sub.all.retain(|o| *o != self.observer_id);
                }
                if let Some(observers) = subs.group_deleted.by_gid.get_mut(gid) {
                    observers.retain(|o| *o != self.observer_id);
                }
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    if let Some(group_sub) = subs.group_device_added.by_gid.get_mut(gid) {
                        group_sub.all.retain(|o| *o != self.observer_id);
                    }
                    if let Some(observers) = subs.group_deleted.by_gid.get_mut(gid) {
                        observers.retain(|o| *o != self.observer_id);
                    }
                }
            }
            DeviceGroupFilter::Any => {}
        }

        // clean up component_put subscriptions
        if let Some(filter) = &self.query.entity_filter.type_filter {
            let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
            collect_all_types_in_tf(filter, &mut care);

            for ct in care {
                if let Some(observers) = subs.component_put.by_comp_type.get_mut(&ct) {
                    observers.retain(|o| *o != self.observer_id);
                }
            }
        }
    }

    /// Expand matching set with a new device/entity pair
    /// WARN: this has no checks, use try_expand_* if you are unsure to expand or not
    fn expand_new_match(&mut self, subs: &mut TreeSubscribers, device: &Device, entity: &Entity) {
        if let Some(device_match) = self.matched.get_mut(device.id()) {
            // device already matched, just add entity
            device_match.add_entity(*entity.index(), subs, &self.query);
        } else {
            // new device match
            let mut device_match =
                DeviceMatch::new(*device.id(), self.observer_id, subs, device, &self.query);
            device_match.add_entity(*entity.index(), subs, &self.query);
            self.matched.insert(*device.id(), device_match);
        }
    }

    pub fn matches_device(&self, tree: &DeviceTree, device: &Device) -> bool {
        passes_id_filter(device, &self.query.device_filter.id)
            && passes_group_filter(device, &self.query.device_filter.group, tree)
            && passes_owner_filter(device, &self.query.device_filter.owner)
    }

    pub fn matches_entity(&mut self, ctx: &mut QueryContext, entity: &Entity) -> bool {
        if let Some(filter) = &self.query.entity_filter.type_filter
            && !entity.matches(filter)
        {
            return false;
        }

        passes_entity_id_filter(ctx, entity, &self.query.entity_filter.id)
    }

    /// Expands matching set IF this is a valid target
    /// Returns if it was expanded
    fn try_expand_device(
        &mut self,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> bool {
        // check if already matched
        if self.matched.contains_key(device.id()) {
            return false;
        }

        if !self.matches_device(tree, device) {
            return false;
        }

        // create device match
        let device_match =
            DeviceMatch::new(*device.id(), self.observer_id, subs, device, &self.query);
        self.matched.insert(*device.id(), device_match);

        // try to expand existing entities
        let mut expanded = false;
        for entity in device.entities() {
            if self.try_expand_entity(ctx, subs, device, entity) {
                expanded = true;
            }
        }

        expanded
    }

    fn try_expand_entity(
        &mut self,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        device: &Device,
        entity: &Entity,
    ) -> bool {
        // check if already matched
        if let Some(device_match) = self.matched.get(device.id())
            && device_match.entities.contains_key(entity.index())
        {
            return false;
        }

        if !self.matches_entity(ctx, entity) {
            return false;
        }

        // ensure device is matched first
        if !self.matched.contains_key(device.id()) {
            let device_match =
                DeviceMatch::new(*device.id(), self.observer_id, subs, device, &self.query);
            self.matched.insert(*device.id(), device_match);
        }

        // add entity
        if let Some(device_match) = self.matched.get_mut(device.id()) {
            device_match.add_entity(*entity.index(), subs, &self.query);
        }

        true
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

        let is_empty = device_match.remove_entity(entity_index, subs, &self.query);

        // remove device if no entities left
        if is_empty && let Some(device_match) = self.matched.remove(&did) {
            device_match.cleanup(subs, &self.query);
        }
    }

    fn passes_runtime_filters(&self, device: &Device, entity: &Entity) -> bool {
        if !passes_entity_last_update(entity, &self.query.entity_filter.last_update) {
            return false;
        }

        if !passes_entity_count(device, &self.query.device_filter.entity_count) {
            return false;
        }

        if !passes_device_last_update(
            &Instant::now(),
            device,
            &self.query.device_filter.last_update,
        ) {
            return false;
        }

        if let Some(filter) = &self.query.entity_filter.value_filter
            && !passes_value_filter(entity, filter)
        {
            return false;
        }

        true
    }
}

impl DeviceMatch {
    fn new(
        device_id: DeviceID,
        observer_id: usize,
        subs: &mut TreeSubscribers,
        device: &Device,
        query: &EntityQuery,
    ) -> Self {
        // subscribe to device-level events that can cause contraction

        subs.device_deleted
            .by_did
            .entry(device_id)
            .or_insert_with(|| Vec::with_capacity(2))
            .push(observer_id);

        if let Some(xindex) = device.owner_ref() {
            subs.ext_detached
                .by_xindex
                .entry(xindex)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(observer_id);
        }

        if matches!(query.action, EntityAction::ObserveRegistered) {
            subs.entity_registered
                .by_did
                .entry(device_id)
                .or_insert_with(|| Vec::with_capacity(2))
                .push(observer_id);
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
                    .push(observer_id);
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
                        .push(observer_id);
                }
            }
        }

        Self {
            device_id,
            observer_id,
            entities: HashMap::with_capacity_and_hasher(4, FxBuildHasher),
            owner_ref: device.owner_ref(),
        }
    }

    fn add_entity(
        &mut self,
        entity_index: EntityIndex,
        subs: &mut TreeSubscribers,
        query: &EntityQuery,
    ) {
        let entity_match =
            EntityMatch::new(self.device_id, entity_index, self.observer_id, subs, query);
        self.entities.insert(entity_index, entity_match);
    }

    fn remove_entity(
        &mut self,
        entity_index: EntityIndex,
        subs: &mut TreeSubscribers,
        query: &EntityQuery,
    ) -> bool {
        if let Some(entity_match) = self.entities.remove(&entity_index) {
            entity_match.cleanup(subs, query);
        }
        self.entities.is_empty()
    }

    fn cleanup(&self, subs: &mut TreeSubscribers, query: &EntityQuery) {
        for entity_match in self.entities.values() {
            entity_match.cleanup(subs, query);
        }

        if let Some(observers) = subs.device_deleted.by_did.get_mut(&self.device_id) {
            observers.retain(|o| *o != self.observer_id);
            if observers.is_empty() {
                subs.device_deleted.by_did.remove(&self.device_id);
            }
        }

        if let Some(xindex) = self.owner_ref
            && let Some(observers) = subs.ext_detached.by_xindex.get_mut(&xindex)
        {
            observers.retain(|o| *o != self.observer_id);
        }

        if matches!(query.action, EntityAction::ObserveRegistered) {
            if let Some(observers) = subs.entity_registered.by_did.get_mut(&self.device_id) {
                observers.retain(|o| *o != self.observer_id);
                if observers.is_empty() {
                    subs.entity_registered.by_did.remove(&self.device_id);
                }
            }
        }

        match &query.device_filter.group {
            DeviceGroupFilter::Any => {}
            DeviceGroupFilter::In(gid) => {
                if let Some(group_sub) = subs.group_device_removed.by_gid.get_mut(gid)
                    && let Some(observers) = group_sub.by_did.get_mut(&self.device_id)
                {
                    observers.retain(|o| *o != self.observer_id);
                    if observers.is_empty() {
                        group_sub.by_did.remove(&self.device_id);
                    }
                }
            }
            DeviceGroupFilter::InAny(gids) | DeviceGroupFilter::InAll(gids) => {
                for gid in gids {
                    if let Some(group_sub) = subs.group_device_removed.by_gid.get_mut(gid)
                        && let Some(observers) = group_sub.by_did.get_mut(&self.device_id)
                    {
                        observers.retain(|o| *o != self.observer_id);
                        if observers.is_empty() {
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
        observer_id: usize,
        subs: &mut TreeSubscribers,
        query: &EntityQuery,
    ) -> Self {
        if let Some(filter) = &query.entity_filter.type_filter {
            let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
            collect_all_types_in_tf(filter, &mut care);

            for ct in care {
                subs.component_put
                    .by_did
                    .entry(device_id)
                    .or_default()
                    .by_eindex
                    .entry(entity_index)
                    .or_default()
                    .by_comp_type
                    .entry(ct)
                    .or_insert_with(|| Vec::with_capacity(2))
                    .push(observer_id);
            }
        }

        Self {
            device_id,
            entity_index,
            observer_id,
        }
    }

    fn cleanup(&self, subs: &mut TreeSubscribers, query: &EntityQuery) {
        if let Some(filter) = &query.entity_filter.type_filter {
            let mut care = HashSet::with_capacity_and_hasher(20, FxBuildHasher);
            collect_all_types_in_tf(filter, &mut care);

            if let Some(device_sub) = subs.component_put.by_did.get_mut(&self.device_id)
                && let Some(entity_sub) = device_sub.by_eindex.get_mut(&self.entity_index)
            {
                for ct in care {
                    if let Some(observers) = entity_sub.by_comp_type.get_mut(&ct) {
                        observers.retain(|o| *o != self.observer_id);
                        if observers.is_empty() {
                            entity_sub.by_comp_type.remove(&ct);
                        }
                    }
                }

                if entity_sub.is_empty() {
                    device_sub.by_eindex.remove(&self.entity_index);
                }

                if device_sub.is_empty() {
                    subs.component_put.by_did.remove(&self.device_id);
                }
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

impl ObserverHandler for EntityObserver {
    fn on_entity_registered(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
    ) -> Result<(), IglooError> {
        let entity = &device.entities()[entity_index.0];

        if matches!(self.query.action, EntityAction::ObserveRegistered)
            && self.matched.contains_key(device.id())
        {
            if !self.matches_entity(ctx, entity) {
                return Ok(());
            }

            if !self.passes_runtime_filters(device, entity) {
                return Ok(());
            }

            if !self
                .matched
                .get(device.id())
                .unwrap()
                .entities
                .contains_key(&entity_index)
            {
                if let Some(device_match) = self.matched.get_mut(device.id()) {
                    device_match.add_entity(entity_index, subs, &self.query);
                }
            }

            cm.send(
                self.client_id,
                IglooResponse::ObserverUpdate {
                    query_id: self.query_id,
                    result: U::EntityRegistered(*device.id(), entity.id().clone()),
                },
            )?;
        } else {
            self.try_expand_entity(ctx, subs, device, entity);
        }

        Ok(())
    }

    fn on_component_put(
        &mut self,
        cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        _: &DeviceTree,
        device: &Device,
        entity_index: EntityIndex,
        _: ComponentType,
        comp: &Component,
    ) -> Result<(), IglooError> {
        let entity = &device.entities()[entity_index.0];

        let is_matched = self
            .matched
            .get(device.id())
            .and_then(|d| d.entities.get(&entity_index))
            .is_some();

        if is_matched {
            if let Some(filter) = &self.query.entity_filter.type_filter
                && !entity.matches(filter)
            {
                self.contract_entity(subs, *device.id(), entity_index);
                return Ok(());
            }

            if matches!(self.query.action, EntityAction::ObserveComponentPut) {
                if !self.passes_runtime_filters(device, entity) {
                    return Ok(());
                }

                cm.send(
                    self.client_id,
                    IglooResponse::ObserverUpdate {
                        query_id: self.query_id,
                        result: U::EntityComponentPut(
                            *device.id(),
                            entity.id().clone(),
                            comp.clone(),
                        ),
                    },
                )?;
            }
        } else {
            let expanded = self.try_expand_entity(ctx, subs, device, entity);

            if expanded && matches!(self.query.action, EntityAction::ObserveComponentPut) {
                if !self.passes_runtime_filters(device, entity) {
                    return Ok(());
                }

                cm.send(
                    self.client_id,
                    IglooResponse::ObserverUpdate {
                        query_id: self.query_id,
                        result: U::EntityComponentPut(
                            *device.id(),
                            entity.id().clone(),
                            comp.clone(),
                        ),
                    },
                )?;
            }
        }

        Ok(())
    }

    fn on_device_created(
        &mut self,
        _cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.try_expand_device(ctx, subs, tree, device);
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

    fn on_ext_attached(
        &mut self,
        _cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        ext: &Extension,
    ) -> Result<(), IglooError> {
        for did in ext.devices() {
            if let Ok(device) = tree.device(did) {
                self.try_expand_device(ctx, subs, tree, device);
            }
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

    fn on_group_device_added(
        &mut self,
        _cm: &mut ClientManager,
        ctx: &mut QueryContext,
        subs: &mut TreeSubscribers,
        tree: &DeviceTree,
        _group: &Group,
        device: &Device,
    ) -> Result<(), IglooError> {
        self.try_expand_device(ctx, subs, tree, device);
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
            "EntityObserver should never receive device_renamed events"
        );
        Ok(())
    }

    fn on_component_set(
        &mut self,
        _: &mut ClientManager,
        _: &mut QueryContext,
        _: &mut TreeSubscribers,
        _: &DeviceTree,
        _: &Device,
        _: EntityIndex,
        _: ComponentType,
        _: &Component,
    ) -> Result<(), IglooError> {
        debug_assert!(
            false,
            "EntityObserver should never receive component_set events"
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
            "EntityObserver should never receive group_created events"
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
            "EntityObserver should never receive group_renamed events"
        );
        Ok(())
    }
}
