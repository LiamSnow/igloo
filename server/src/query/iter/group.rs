use crate::tree::{DeviceTree, Group};
use igloo_interface::{id::GroupID, query::IDFilter};
use std::ops::ControlFlow;

pub fn estimate_group_count(tree: &DeviceTree, id: &IDFilter<GroupID>) -> usize {
    match id {
        IDFilter::Id(_) => 1,
        IDFilter::IdIn(ids) => ids.len(),
        IDFilter::Any => tree.groups().len(),
    }
}

#[inline]
pub fn for_each_group<F>(tree: &DeviceTree, id: &IDFilter<GroupID>, mut f: F) -> ControlFlow<()>
where
    F: FnMut(&Group) -> ControlFlow<()>,
{
    match id {
        IDFilter::Id(id) => {
            if let Ok(group) = tree.group(id) {
                return f(group);
            }
            ControlFlow::Continue(())
        }
        IDFilter::IdIn(ids) => {
            for id in ids {
                if let Ok(group) = tree.group(id) {
                    f(group)?;
                }
            }
            ControlFlow::Continue(())
        }
        IDFilter::Any => {
            for group in tree.groups() {
                let Some(group) = group else {
                    continue;
                };
                f(group)?;
            }
            ControlFlow::Continue(())
        }
    }
}
