use crate::tree::{DeviceTree, Floe};
use igloo_interface::{id::FloeID, query::IDFilter};
use std::ops::ControlFlow;

pub fn estimate_floe_count(tree: &DeviceTree, id: &IDFilter<FloeID>) -> usize {
    match id {
        IDFilter::Id(_) => 1,
        IDFilter::IdIn(ids) => ids.len(),
        IDFilter::Any => tree.floes().len(),
    }
}

#[inline]
pub fn for_each_floe<F>(tree: &DeviceTree, id: &IDFilter<FloeID>, mut f: F) -> ControlFlow<()>
where
    F: FnMut(&Floe) -> ControlFlow<()>,
{
    match id {
        IDFilter::Id(id) => {
            let Ok(fref) = tree.floe_ref(id) else {
                return ControlFlow::Continue(());
            };
            if let Ok(floe) = tree.floe(fref) {
                return f(floe);
            }
            ControlFlow::Continue(())
        }
        IDFilter::IdIn(ids) => {
            for id in ids {
                let Ok(fref) = tree.floe_ref(id) else {
                    continue;
                };
                if let Ok(floe) = tree.floe(fref) {
                    f(floe)?;
                }
            }
            ControlFlow::Continue(())
        }
        IDFilter::Any => {
            for floe in tree.floes() {
                let Some(floe) = floe else {
                    continue;
                };
                f(floe)?;
            }
            ControlFlow::Continue(())
        }
    }
}
