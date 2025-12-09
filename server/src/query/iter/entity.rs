use crate::{
    query::{
        ctx::QueryContext,
        iter::{estimate_device_count, for_each_device},
    },
    tree::{Device, DeviceTree, Entity},
};
use igloo_interface::{
    ComponentType,
    id::EntityIndex,
    query::{DeviceFilter, EntityFilter, EntityIDFilter, TypeFilter, ValueFilter},
    types::compare::ComparisonOp,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use smallvec::SmallVec;
use std::{collections::HashSet, ops::ControlFlow};

pub fn estimate_entity_count(
    tree: &DeviceTree,
    device_filter: &DeviceFilter,
    _entity_filter: &EntityFilter,
) -> usize {
    estimate_device_count(tree, device_filter) << 3
}

#[inline]
pub fn for_each_entity<F>(
    ctx: &mut QueryContext,
    tree: &DeviceTree,
    device_filter: &DeviceFilter,
    entity_filter: &EntityFilter,
    mut f: F,
) -> ControlFlow<()>
where
    F: FnMut(&Device, &Entity) -> ControlFlow<()>,
{
    let type_filter = &entity_filter.type_filter;

    let required_types: Option<SmallVec<[ComponentType; 4]>> = type_filter
        .as_ref()
        .and_then(extract_required_comp_types)
        .map(|set| set.into_iter().collect());

    for_each_device(
        *ctx.now(),
        tree,
        device_filter,
        type_filter.as_ref(),
        |device| {
            match &required_types {
                // FIXME if entity has multiple required component
                // types it will be checked multiple times
                Some(types) => {
                    for t in types {
                        let indices: &SmallVec<[EntityIndex; 4]> =
                            &device.comp_to_entity()[*t as usize];
                        for index in indices {
                            let Some(entity) = device.entities().get(index.0) else {
                                continue;
                            };

                            if !check_entity(ctx, entity, entity_filter, type_filter.as_ref()) {
                                continue;
                            }

                            f(device, entity)?;
                        }
                    }
                }
                None => {
                    for entity in device.entities() {
                        if !check_entity(ctx, entity, entity_filter, type_filter.as_ref()) {
                            continue;
                        }

                        f(device, entity)?;
                    }
                }
            }

            ControlFlow::Continue(())
        },
    )
}

#[inline(always)]
fn check_entity(
    ctx: &mut QueryContext,
    entity: &Entity,
    entity_filter: &EntityFilter,
    type_filter: Option<&TypeFilter>,
) -> bool {
    if !passes_entity_last_update(entity, &entity_filter.last_update) {
        return false;
    }

    if let Some(filter) = type_filter
        && !entity.matches(filter)
    {
        return false;
    }

    if let Some(filter) = &entity_filter.value_filter
        && !passes_value_filter(entity, filter)
    {
        return false;
    }

    if !passes_entity_id_filter(ctx, entity, &entity_filter.id) {
        return false;
    }

    true
}

#[inline(always)]
pub fn passes_entity_last_update(entity: &Entity, filter: &Option<(ComparisonOp, usize)>) -> bool {
    match filter {
        None => true,
        Some((op, seconds)) => {
            op.eval_usize(entity.last_updated().elapsed().as_secs() as usize, *seconds)
        }
    }
}

#[inline(always)]
pub fn passes_value_filter(entity: &Entity, filter: &ValueFilter) -> bool {
    use ValueFilter::*;
    match filter {
        If(op, rhs) => entity
            .get(rhs.get_type())
            .and_then(|lhs| {
                let lhs = lhs.to_igloo_value()?;
                let rhs = rhs.to_igloo_value()?;
                Some(op.eval(&lhs, &rhs).unwrap_or(false))
            })
            .unwrap_or(false),
        And(filters) => {
            for filter in filters {
                if !passes_value_filter(entity, filter) {
                    return false;
                }
            }
            true
        }
        Or(filters) => {
            for filter in filters {
                if passes_value_filter(entity, filter) {
                    return true;
                }
            }
            false
        }
        Not(filter) => !passes_value_filter(entity, filter),
    }
}

#[inline(always)]
pub fn passes_entity_id_filter(
    ctx: &mut QueryContext,
    entity: &Entity,
    filter: &EntityIDFilter,
) -> bool {
    match filter {
        EntityIDFilter::Any => true,
        EntityIDFilter::Is(id) => &entity.id().0 == id,
        EntityIDFilter::OneOf(set) => set.contains(&entity.id().0),
        EntityIDFilter::Matches(pattern) => {
            let glob = ctx.glob(pattern);
            glob.is_match(&entity.id().0)
        }
    }
}

/// Extracts component types that are guaranteed to be present if the filter matches.
///
/// Returns `Some(set)` containing component types that MUST exist for any entity
/// to pass this filter. Returns `None` if the filter doesn't guarantee any specific
/// component types are present.
pub fn extract_required_comp_types(filter: &TypeFilter) -> Option<FxHashSet<ComponentType>> {
    let mut set = HashSet::with_capacity_and_hasher(20, FxBuildHasher);

    if extract_required_comp_types_rec(filter, &mut set) {
        Some(set)
    } else {
        None
    }
}

/// Returns `true` if this filter branch guarantees at least one component type
/// is required, `false` otherwise. Populates `set` with discovered required types.
fn extract_required_comp_types_rec(
    filter: &TypeFilter,
    set: &mut FxHashSet<ComponentType>,
) -> bool {
    use TypeFilter::*;
    match filter {
        With(t) => {
            set.insert(*t);
            true
        }
        Without(_) => false,
        And(filters) => {
            // we only need ONE required type to use the reverse index

            // try to find a simple With first (best case)
            for f in filters {
                if let With(t) = f {
                    set.insert(*t);
                    return true;
                }
            }

            // must recurse to find any branch with requirements
            for f in filters {
                if extract_required_comp_types_rec(f, set) {
                    return true;
                }
            }
            false
        }
        Or(filters) => {
            // ALL branches must guarantee a type, otherwise some matches
            // might not have any required types (taking the branch that returned false)
            for f in filters {
                if !extract_required_comp_types_rec(f, set) {
                    return false;
                }
            }
            true
        }
        Not(_) => false,
    }
}
