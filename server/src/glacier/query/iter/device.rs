use crate::glacier::tree::{Device, DeviceTree};
use igloo_interface::{
    id::{DeviceID, FloeID},
    query::{DeviceFilter, DeviceGroupFilter, IDFilter, TypeFilter},
    types::compare::ComparisonOp,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::{collections::HashSet, ops::ControlFlow};

pub fn estimate_device_count(tree: &DeviceTree, filter: &DeviceFilter) -> usize {
    match &filter.id {
        IDFilter::Id(_) => 1,
        IDFilter::IdIn(ids) => ids.len(),
        IDFilter::Any => match &filter.owner {
            IDFilter::Id(fid) => tree
                .floe_ref(fid)
                .ok()
                .and_then(|fref| tree.floe(fref).ok())
                .map(|f| f.devices().len())
                .unwrap_or(0),
            IDFilter::IdIn(fids) => fids.len() << 4,
            IDFilter::Any => match &filter.group {
                DeviceGroupFilter::InGroup(gid) => {
                    tree.group(gid).map(|g| g.devices().len()).unwrap_or(0)
                }
                DeviceGroupFilter::InAnyGroup(gids) => gids
                    .iter()
                    .filter_map(|gid| tree.group(gid).ok())
                    .map(|g| g.devices().len())
                    .sum(),
                DeviceGroupFilter::InAllGroups(gids) => gids
                    .iter()
                    .filter_map(|gid| tree.group(gid).ok())
                    .map(|g| g.devices().len())
                    .min()
                    .unwrap_or(0),
                // overestimate, since some slots can be empty
                DeviceGroupFilter::Any => tree.devices().len(),
            },
        },
    }
}

#[inline]
pub fn for_each_device<F>(
    tree: &DeviceTree,
    filter: &DeviceFilter,
    type_filter: Option<&TypeFilter>,
    mut f: F,
) -> ControlFlow<()>
where
    F: FnMut(&Device) -> ControlFlow<()>,
{
    // iterate over specified IDs (best case)
    match &filter.id {
        IDFilter::Id(id) => {
            let Some(device) = tree.device(id).ok() else {
                return ControlFlow::Continue(());
            };
            if passes_entity_count(device, &filter.entity_count)
                && passes_last_update(device, &filter.last_update)
                && passes_group_filter(device, &filter.group, tree)
                && passes_type_filter(device, type_filter)
                && passes_owner_filter(device, &filter.owner)
            {
                return f(device);
            }
            return ControlFlow::Continue(());
        }
        IDFilter::IdIn(ids) => {
            for did in ids {
                let Ok(device) = tree.device(did) else {
                    continue;
                };
                if !passes_entity_count(device, &filter.entity_count)
                    || !passes_last_update(device, &filter.last_update)
                    || !passes_group_filter(device, &filter.group, tree)
                    || !passes_type_filter(device, type_filter)
                    || !passes_owner_filter(device, &filter.owner)
                {
                    continue;
                }
                f(device)?;
            }
            return ControlFlow::Continue(());
        }
        IDFilter::Any => {}
    }

    // iterate over Floe's devices
    match &filter.owner {
        IDFilter::Id(fid) => {
            let Some(dids) = tree
                .floe_ref(fid)
                .ok()
                .and_then(|fref| tree.floe(fref).ok())
                .map(|f| f.devices())
            else {
                return ControlFlow::Continue(());
            };
            for did in dids {
                let Ok(device) = tree.device(did) else {
                    continue;
                };
                if !passes_id_filter(device, &filter.id)
                    || !passes_entity_count(device, &filter.entity_count)
                    || !passes_last_update(device, &filter.last_update)
                    || !passes_group_filter(device, &filter.group, tree)
                    || !passes_type_filter(device, type_filter)
                {
                    continue;
                }
                f(device)?;
            }
            return ControlFlow::Continue(());
        }
        IDFilter::IdIn(fids) => {
            for fid in fids {
                let Ok(fref) = tree.floe_ref(fid) else {
                    continue;
                };
                let Ok(floe) = tree.floe(fref) else {
                    continue;
                };
                for did in floe.devices() {
                    let Ok(device) = tree.device(did) else {
                        continue;
                    };
                    if !passes_id_filter(device, &filter.id)
                        || !passes_entity_count(device, &filter.entity_count)
                        || !passes_last_update(device, &filter.last_update)
                        || !passes_group_filter(device, &filter.group, tree)
                        || !passes_type_filter(device, type_filter)
                    {
                        continue;
                    }
                    f(device)?;
                }
            }
            return ControlFlow::Continue(());
        }
        IDFilter::Any => {}
    }

    // iterate over Group's devices
    match &filter.group {
        DeviceGroupFilter::InGroup(group_id) => {
            let Ok(group) = tree.group(group_id) else {
                return ControlFlow::Continue(());
            };
            for did in group.devices() {
                let Ok(device) = tree.device(did) else {
                    continue;
                };
                if !passes_id_filter(device, &filter.id)
                    || !passes_entity_count(device, &filter.entity_count)
                    || !passes_last_update(device, &filter.last_update)
                    || !passes_type_filter(device, type_filter)
                    || !passes_owner_filter(device, &filter.owner)
                {
                    continue;
                }
                f(device)?;
            }
            return ControlFlow::Continue(());
        }
        DeviceGroupFilter::InAnyGroup(group_ids) => {
            let mut seen = HashSet::with_capacity_and_hasher(group_ids.len() << 3, FxBuildHasher);
            for gid in group_ids {
                let Ok(group) = tree.group(gid) else { continue };
                for did in group.devices() {
                    seen.insert(*did);
                }
            }
            for did in seen {
                let Ok(device) = tree.device(&did) else {
                    continue;
                };
                if !passes_id_filter(device, &filter.id)
                    || !passes_entity_count(device, &filter.entity_count)
                    || !passes_last_update(device, &filter.last_update)
                    || !passes_type_filter(device, type_filter)
                    || !passes_owner_filter(device, &filter.owner)
                {
                    continue;
                }
                f(device)?;
            }
            return ControlFlow::Continue(());
        }
        DeviceGroupFilter::InAllGroups(group_ids) => {
            if group_ids.is_empty() {
                return ControlFlow::Continue(());
            }

            let mut iter = group_ids.iter();
            let first_gid = iter.next().unwrap();
            let Ok(first_group) = tree.group(first_gid) else {
                return ControlFlow::Continue(());
            };

            let mut candidates: FxHashSet<_> = first_group.devices().clone();

            for gid in iter {
                let Ok(group) = tree.group(gid) else {
                    return ControlFlow::Continue(());
                };
                candidates.retain(|did| group.devices().contains(did));
                if candidates.is_empty() {
                    return ControlFlow::Continue(());
                }
            }

            for did in candidates {
                let Ok(device) = tree.device(&did) else {
                    continue;
                };
                if !passes_id_filter(device, &filter.id)
                    || !passes_entity_count(device, &filter.entity_count)
                    || !passes_last_update(device, &filter.last_update)
                    || !passes_type_filter(device, type_filter)
                    || !passes_owner_filter(device, &filter.owner)
                {
                    continue;
                }
                f(device)?;
            }
            return ControlFlow::Continue(());
        }
        DeviceGroupFilter::Any => {}
    }

    // worst case we have to full scan
    for device in tree.devices() {
        let Some(device) = device else {
            continue;
        };

        if !passes_id_filter(device, &filter.id)
            || !passes_entity_count(device, &filter.entity_count)
            || !passes_last_update(device, &filter.last_update)
            || !passes_type_filter(device, type_filter)
            || !passes_group_filter(device, &filter.group, tree)
            || !passes_owner_filter(device, &filter.owner)
        {
            continue;
        }

        f(device)?;
    }

    ControlFlow::Continue(())
}

#[inline(always)]
fn passes_id_filter(device: &Device, filter: &IDFilter<DeviceID>) -> bool {
    match filter {
        IDFilter::Any => true,
        IDFilter::Id(id) => device.id() == id,
        IDFilter::IdIn(ids) => ids.contains(device.id()),
    }
}

#[inline(always)]
fn passes_owner_filter(device: &Device, filter: &IDFilter<FloeID>) -> bool {
    match filter {
        IDFilter::Any => true,
        IDFilter::Id(fid) => device.owner() == fid,
        IDFilter::IdIn(fids) => fids.contains(device.owner()),
    }
}

#[inline(always)]
fn passes_group_filter(device: &Device, filter: &DeviceGroupFilter, tree: &DeviceTree) -> bool {
    match filter {
        DeviceGroupFilter::Any => true,
        DeviceGroupFilter::InGroup(gid) => tree
            .group(gid)
            .map(|g| g.devices().contains(device.id()))
            .unwrap_or(false),
        DeviceGroupFilter::InAnyGroup(gids) => gids.iter().any(|gid| {
            tree.group(gid)
                .map(|g| g.devices().contains(device.id()))
                .unwrap_or(false)
        }),
        DeviceGroupFilter::InAllGroups(gids) => gids.iter().all(|gid| {
            tree.group(gid)
                .map(|g| g.devices().contains(device.id()))
                .unwrap_or(false)
        }),
    }
}

#[inline(always)]
fn passes_entity_count(device: &Device, filter: &Option<(ComparisonOp, usize)>) -> bool {
    match filter {
        None => true,
        Some((op, count)) => op.eval_usize(device.num_entities(), *count),
    }
}

#[inline(always)]
fn passes_last_update(device: &Device, filter: &Option<(ComparisonOp, usize)>) -> bool {
    match filter {
        None => true,
        Some((op, seconds)) => {
            op.eval_usize(device.last_updated().elapsed().as_secs() as usize, *seconds)
        }
    }
}

#[inline(always)]
fn passes_type_filter(device: &Device, filter: Option<&TypeFilter>) -> bool {
    match filter {
        None => true,
        Some(filter) => device.matches(filter),
    }
}
