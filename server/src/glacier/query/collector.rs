use crate::glacier::{
    query::{
        QueryEngine,
        filter::{DeviceFilterMatcher, EntityFilterMatcher, FloeFilterMatcher, GroupFilterMatcher},
    },
    tree::{Device, DeviceTree, Entity, Floe, Group},
};
use igloo_interface::{
    id::{DeviceID, FloeRef, GroupID},
    query::{DeviceFilter as D, FloeFilter as F, GroupFilter as G, Query},
};
use std::collections::HashSet;

pub trait QueryCollector {
    fn collect_matching_floes<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<&'a Floe>;

    fn collect_matching_groups<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<&'a Group>;

    fn collect_matching_devices<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<&'a Device>;

    fn collect_matching_entities<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<(&'a Device, &'a Entity)>;
}

pub trait QueryNarrowing {
    fn find_min_floes(&mut self, tree: &DeviceTree) -> Option<Vec<FloeRef>>;
    fn find_min_groups(&mut self) -> Option<Vec<GroupID>>;
    fn find_min_devices(&mut self, tree: &DeviceTree) -> Option<Vec<DeviceID>>;
}

pub trait DeviceNarrowing {
    fn narrow_and_consume(&mut self) -> (Option<HashSet<DeviceID>>, bool);
}

pub trait GroupNarrowing {
    fn narrow_and_consume(&mut self) -> (Option<HashSet<GroupID>>, bool);
    fn narrow_to_devices_and_consume(
        &mut self,
        tree: &DeviceTree,
    ) -> (Option<HashSet<DeviceID>>, bool);
}

pub trait FloeNarrowing {
    fn narrow_and_consume(&mut self, tree: &DeviceTree) -> (Option<HashSet<FloeRef>>, bool);
    fn narrow_to_devices_and_consume(
        &mut self,
        tree: &DeviceTree,
    ) -> (Option<HashSet<DeviceID>>, bool);
}

impl QueryCollector for Query {
    fn collect_matching_floes<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<&'a Floe> {
        let limit = self.limit.unwrap_or(usize::MAX);

        // try and narrow down so we dont need a global scan
        if let Some(floe_refs) = self.find_min_floes(tree) {
            let mut results = Vec::with_capacity(limit.min(floe_refs.len()));

            for fref in floe_refs {
                if results.len() >= limit {
                    break;
                }

                let Ok(floe) = tree.floe(&fref) else {
                    continue;
                };

                if let Some(filter) = &self.floe_filter
                    && !filter.matches(engine, tree, floe)
                {
                    continue;
                }

                results.push(floe);
            }

            results
        } else {
            // couldn't narrow off filters -> global scan
            let mut results = Vec::with_capacity(limit.min(64));

            for floe in tree.iter_floes() {
                if results.len() >= limit {
                    break;
                }

                if let Some(filter) = &self.floe_filter
                    && !filter.matches(engine, tree, floe)
                {
                    continue;
                }

                results.push(floe);
            }

            results
        }
    }

    fn collect_matching_groups<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<&'a Group> {
        let limit = self.limit.unwrap_or(usize::MAX);

        // try and narrow down so we dont need a global scan
        if let Some(group_ids) = self.find_min_groups() {
            let mut results = Vec::with_capacity(limit.min(group_ids.len()));

            for gid in group_ids {
                if results.len() >= limit {
                    break;
                }

                let Ok(group) = tree.group(&gid) else {
                    continue;
                };

                if let Some(filter) = &self.group_filter
                    && !filter.matches(engine, tree, group)
                {
                    continue;
                }

                results.push(group);
            }

            results
        } else {
            // couldn't narrow off filters -> global scan
            let mut results = Vec::with_capacity(limit.min(64));

            for group in tree.iter_groups() {
                if results.len() >= limit {
                    break;
                }

                if let Some(filter) = &self.group_filter
                    && !filter.matches(engine, tree, group)
                {
                    continue;
                }

                results.push(group);
            }

            results
        }
    }

    fn collect_matching_devices<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<&'a Device> {
        let limit = self.limit.unwrap_or(usize::MAX);

        // try and narrow down so we dont need a global scan
        if let Some(device_ids) = self.find_min_devices(tree) {
            let mut results = Vec::with_capacity(limit.min(device_ids.len()));

            for did in device_ids {
                if results.len() >= limit {
                    break;
                }

                let Ok(device) = tree.device(&did) else {
                    continue;
                };

                if let Some(filter) = &self.device_filter
                    && !filter.matches(engine, device)
                {
                    continue;
                }

                if let Some(filter) = &self.floe_filter {
                    if let Some(owner_ref) = device.owner_ref() {
                        if let Ok(floe) = tree.floe(&owner_ref) {
                            if !filter.matches(engine, tree, floe) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if let Some(filter) = &self.group_filter {
                    let matches_group = device
                        .groups()
                        .iter()
                        .filter_map(|gid| tree.group(gid).ok())
                        .any(|group| filter.matches(engine, tree, group));

                    if !matches_group {
                        continue;
                    }
                }

                results.push(device);
            }

            results
        } else {
            // couldn't narrow off filters -> global scan
            let mut results = Vec::with_capacity(limit.min(256));

            for device in tree.iter_devices() {
                if results.len() >= limit {
                    break;
                }

                if let Some(filter) = &self.device_filter
                    && !filter.matches(engine, device)
                {
                    continue;
                }

                if let Some(filter) = &self.floe_filter {
                    if let Some(owner_ref) = device.owner_ref() {
                        if let Ok(floe) = tree.floe(&owner_ref) {
                            if !filter.matches(engine, tree, floe) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if let Some(filter) = &self.group_filter {
                    let matches_group = device
                        .groups()
                        .iter()
                        .filter_map(|gid| tree.group(gid).ok())
                        .any(|group| filter.matches(engine, tree, group));

                    if !matches_group {
                        continue;
                    }
                }

                results.push(device);
            }

            results
        }
    }

    fn collect_matching_entities<'a>(
        &'a mut self,
        engine: &'a mut QueryEngine,
        tree: &'a DeviceTree,
    ) -> Vec<(&'a Device, &'a Entity)> {
        let limit = self.limit.unwrap_or(usize::MAX);

        // try and narrow down so we dont need a global scan
        if let Some(device_ids) = self.find_min_devices(tree) {
            let estimated_total = device_ids.len().saturating_mul(8);
            let capacity = limit.min(estimated_total);
            let mut results = Vec::with_capacity(capacity);

            'devices: for did in device_ids {
                if results.len() >= limit {
                    break;
                }

                let Ok(device) = tree.device(&did) else {
                    continue;
                };

                if let Some(filter) = &self.device_filter
                    && !filter.matches(engine, device)
                {
                    continue;
                }

                if let Some(filter) = &self.floe_filter {
                    if let Some(owner_ref) = device.owner_ref() {
                        if let Ok(floe) = tree.floe(&owner_ref) {
                            if !filter.matches(engine, tree, floe) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if let Some(filter) = &self.group_filter {
                    let matches_group = device
                        .groups()
                        .iter()
                        .filter_map(|gid| tree.group(gid).ok())
                        .any(|group| filter.matches(engine, tree, group));

                    if !matches_group {
                        continue;
                    }
                }

                // device valid -> scan entities
                for entity in device.entities() {
                    if results.len() >= limit {
                        break 'devices;
                    }

                    if let Some(filter) = &self.entity_filter
                        && !filter.matches(engine, entity)
                    {
                        continue;
                    }

                    results.push((device, entity));
                }
            }

            results
        } else {
            // couldn't narrow off filters -> global scan
            let mut results = Vec::with_capacity(limit.min(2048));

            'devices: for device in tree.iter_devices() {
                if results.len() >= limit {
                    break;
                }

                if let Some(filter) = &self.device_filter
                    && !filter.matches(engine, device)
                {
                    continue;
                }

                if let Some(filter) = &self.floe_filter {
                    if let Some(owner_ref) = device.owner_ref() {
                        if let Ok(floe) = tree.floe(&owner_ref) {
                            if !filter.matches(engine, tree, floe) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if let Some(filter) = &self.group_filter {
                    let matches_group = device
                        .groups()
                        .iter()
                        .filter_map(|gid| tree.group(gid).ok())
                        .any(|group| filter.matches(engine, tree, group));

                    if !matches_group {
                        continue;
                    }
                }

                // device valid -> scan entities now
                for entity in device.entities() {
                    if results.len() >= limit {
                        break 'devices;
                    }

                    if let Some(filter) = &self.entity_filter
                        && !filter.matches(engine, entity)
                    {
                        continue;
                    }

                    results.push((device, entity));
                }
            }

            results
        }
    }
}

impl QueryNarrowing for Query {
    fn find_min_devices(&mut self, tree: &DeviceTree) -> Option<Vec<DeviceID>> {
        let mut device_sets: Vec<HashSet<DeviceID>> = Vec::new();

        if let Some(ref mut filter) = self.device_filter {
            let (set_opt, should_drop) = filter.narrow_and_consume();
            if let Some(set) = set_opt {
                device_sets.push(set);
            }
            if should_drop {
                self.device_filter = None;
            }
        }

        if let Some(ref mut filter) = self.floe_filter {
            let (set_opt, should_drop) = filter.narrow_to_devices_and_consume(tree);
            if let Some(set) = set_opt {
                device_sets.push(set);
            }
            if should_drop {
                self.floe_filter = None;
            }
        }

        if let Some(ref mut filter) = self.group_filter {
            let (set_opt, should_drop) = filter.narrow_to_devices_and_consume(tree);
            if let Some(set) = set_opt {
                device_sets.push(set);
            }
            if should_drop {
                self.group_filter = None;
            }
        }

        if device_sets.is_empty() {
            return None;
        }

        let mut result = device_sets[0].clone();
        for set in &device_sets[1..] {
            result = result.intersection(set).copied().collect();
            if result.is_empty() {
                return None;
            }
        }

        Some(result.into_iter().collect())
    }

    fn find_min_groups(&mut self) -> Option<Vec<GroupID>> {
        if let Some(ref mut filter) = self.group_filter {
            let (set_opt, should_drop) = filter.narrow_and_consume();
            if should_drop {
                self.group_filter = None;
            }
            set_opt.map(|set| set.into_iter().collect())
        } else {
            None
        }
    }

    fn find_min_floes(&mut self, tree: &DeviceTree) -> Option<Vec<FloeRef>> {
        if let Some(ref mut filter) = self.floe_filter {
            let (set_opt, should_drop) = filter.narrow_and_consume(tree);
            if should_drop {
                self.floe_filter = None;
            }
            set_opt.map(|set| set.into_iter().collect())
        } else {
            None
        }
    }
}

impl FloeNarrowing for F {
    fn narrow_and_consume(&mut self, tree: &DeviceTree) -> (Option<HashSet<FloeRef>>, bool) {
        match self {
            F::Id(fid) => {
                let fref = tree.floe_ref(fid).ok().copied().map(|r| HashSet::from([r]));
                (fref, true)
            }

            F::Ids(fids) => {
                let refs: HashSet<FloeRef> = fids
                    .iter()
                    .filter_map(|fid| tree.floe_ref(fid).ok().copied())
                    .collect();

                let result = if refs.is_empty() { None } else { Some(refs) };
                (result, true)
            }

            F::All(filters) => {
                let mut sets: Vec<HashSet<FloeRef>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_and_consume(tree);

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let mut result = sets[0].clone();
                for set in &sets[1..] {
                    result = result.intersection(set).copied().collect();
                    if result.is_empty() {
                        return (None, filters.is_empty());
                    }
                }

                (Some(result), filters.is_empty())
            }

            F::Any(filters) => {
                let mut sets: Vec<HashSet<FloeRef>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_and_consume(tree);

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let result: HashSet<FloeRef> = sets.into_iter().flatten().collect();

                (Some(result), filters.is_empty())
            }

            _ => (None, false),
        }
    }

    fn narrow_to_devices_and_consume(
        &mut self,
        tree: &DeviceTree,
    ) -> (Option<HashSet<DeviceID>>, bool) {
        match self {
            F::Id(fid) => {
                let devices = tree
                    .floe_ref(fid)
                    .ok()
                    .and_then(|fref| tree.floe(fref).ok())
                    .map(|floe| floe.devices().iter().copied().collect());
                (devices, true)
            }

            F::Ids(fids) => {
                let devices: HashSet<_> = fids
                    .iter()
                    .filter_map(|fid| {
                        let fref = tree.floe_ref(fid).ok()?;
                        let floe = tree.floe(fref).ok()?;
                        Some(floe.devices().iter().copied())
                    })
                    .flatten()
                    .collect();

                let result = if devices.is_empty() {
                    None
                } else {
                    Some(devices)
                };
                (result, true)
            }

            F::All(filters) => {
                let mut sets: Vec<HashSet<DeviceID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_to_devices_and_consume(tree);

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let mut result = sets[0].clone();
                for set in &sets[1..] {
                    result = result.intersection(set).copied().collect();
                    if result.is_empty() {
                        return (None, filters.is_empty());
                    }
                }

                (Some(result), filters.is_empty())
            }

            F::Any(filters) => {
                let mut sets: Vec<HashSet<DeviceID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_to_devices_and_consume(tree);

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let result: HashSet<DeviceID> = sets.into_iter().flatten().collect();

                (Some(result), filters.is_empty())
            }

            _ => (None, false),
        }
    }
}

impl GroupNarrowing for G {
    fn narrow_and_consume(&mut self) -> (Option<HashSet<GroupID>>, bool) {
        match self {
            G::Id(gid) => (Some(HashSet::from([*gid])), true),

            G::Ids(gids) => (Some(gids.iter().copied().collect()), true),

            G::All(filters) => {
                let mut sets: Vec<HashSet<GroupID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_and_consume();

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let mut result = sets[0].clone();
                for set in &sets[1..] {
                    result = result.intersection(set).copied().collect();
                    if result.is_empty() {
                        return (None, filters.is_empty());
                    }
                }

                (Some(result), filters.is_empty())
            }

            G::Any(filters) => {
                let mut sets: Vec<HashSet<GroupID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_and_consume();

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let result: HashSet<GroupID> = sets.into_iter().flatten().collect();

                (Some(result), filters.is_empty())
            }

            _ => (None, false),
        }
    }

    fn narrow_to_devices_and_consume(
        &mut self,
        tree: &DeviceTree,
    ) -> (Option<HashSet<DeviceID>>, bool) {
        match self {
            G::Id(gid) => {
                let devices = tree
                    .group(gid)
                    .ok()
                    .map(|g| g.devices().iter().copied().collect());
                (devices, true)
            }

            G::Ids(gids) => {
                let devices: HashSet<_> = gids
                    .iter()
                    .filter_map(|gid| tree.group(gid).ok())
                    .flat_map(|g| g.devices().iter().copied())
                    .collect();

                let result = if devices.is_empty() {
                    None
                } else {
                    Some(devices)
                };
                (result, true)
            }

            G::All(filters) => {
                let mut sets: Vec<HashSet<DeviceID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_to_devices_and_consume(tree);

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let mut result = sets[0].clone();
                for set in &sets[1..] {
                    result = result.intersection(set).copied().collect();
                    if result.is_empty() {
                        return (None, filters.is_empty());
                    }
                }

                (Some(result), filters.is_empty())
            }

            G::Any(filters) => {
                let mut sets: Vec<HashSet<DeviceID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_to_devices_and_consume(tree);

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let result: HashSet<DeviceID> = sets.into_iter().flatten().collect();

                (Some(result), filters.is_empty())
            }

            _ => (None, false),
        }
    }
}

impl DeviceNarrowing for D {
    fn narrow_and_consume(&mut self) -> (Option<HashSet<DeviceID>>, bool) {
        match self {
            D::Id(did) => (Some(HashSet::from([*did])), true),

            D::Ids(dids) => (Some(dids.iter().copied().collect()), true),

            D::All(filters) => {
                let mut sets: Vec<HashSet<DeviceID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_and_consume();

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let mut result = sets[0].clone();
                for set in &sets[1..] {
                    result = result.intersection(set).copied().collect();
                    if result.is_empty() {
                        return (None, filters.is_empty());
                    }
                }

                (Some(result), filters.is_empty())
            }

            D::Any(filters) => {
                let mut sets: Vec<HashSet<DeviceID>> = Vec::new();
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for (i, f) in filters.iter_mut().enumerate() {
                    let (set_opt, should_drop) = f.narrow_and_consume();

                    if let Some(set) = set_opt {
                        sets.push(set);
                    }

                    if should_drop {
                        indices_to_remove.push(i);
                    }
                }

                for i in indices_to_remove.into_iter().rev() {
                    filters.remove(i);
                }

                if sets.is_empty() {
                    return (None, filters.is_empty());
                }

                let result: HashSet<DeviceID> = sets.into_iter().flatten().collect();

                (Some(result), filters.is_empty())
            }

            _ => (None, false),
        }
    }
}
