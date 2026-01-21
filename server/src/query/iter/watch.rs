//! Used for WatchQuery
//! Technically this could be faster with its own implementation
//! but the mapping is decently fast and reduced repeated optimization code

use crate::{
    query::{
        ctx::QueryContext,
        iter::{estimate_device_count, for_each_device, passes_entity_id_filter},
    },
    tree::{Device, DeviceTree, Entity},
};
use igloo_interface::query::{DeviceFilter, EntityFilter, WatchComponentQuery};
use std::ops::ControlFlow;

pub fn estimate_entity_count(tree: &DeviceTree, query: &WatchComponentQuery) -> usize {
    let device_filter = DeviceFilter {
        id: query.device_filter.id.clone(),
        owner: query.device_filter.owner.clone(),
        group: query.device_filter.group.clone(),
        entity_count: None,
        last_update: None,
    };

    estimate_device_count(tree, &device_filter) << 3
}

#[inline]
pub fn for_each_entity<F>(
    ctx: &mut QueryContext,
    tree: &DeviceTree,
    query: WatchComponentQuery,
    mut f: F,
) -> ControlFlow<()>
where
    F: FnMut(&Device, &Entity) -> ControlFlow<()>,
{
    let device_filter = DeviceFilter {
        id: query.device_filter.id,
        owner: query.device_filter.owner,
        group: query.device_filter.group,
        entity_count: None,
        last_update: None,
    };

    let entity_filter = EntityFilter {
        id: query.entity_filter.id,
        type_filter: query.entity_filter.type_filter,
        value_filter: None,
        last_update: None,
    };

    let type_filter = &entity_filter.type_filter;

    for_each_device(
        *ctx.now(),
        tree,
        &device_filter,
        type_filter.as_ref(),
        |device| {
            let indices = &device.comp_to_entity()[query.component as usize];

            for index in indices {
                let Some(entity) = device.entities().get(index.0) else {
                    continue;
                };

                if !passes_entity_id_filter(ctx, entity, &entity_filter.id) {
                    continue;
                }

                if let Some(filter) = type_filter
                    && !entity.matches(filter)
                {
                    continue;
                }

                f(device, entity)?;
            }

            ControlFlow::Continue(())
        },
    )
}
