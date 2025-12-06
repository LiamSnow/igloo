use crate::tree::{DeviceTree, Extension};
use igloo_interface::{id::ExtensionID, query::IDFilter};
use std::ops::ControlFlow;

pub fn estimate_ext_count(tree: &DeviceTree, xid: &IDFilter<ExtensionID>) -> usize {
    match xid {
        IDFilter::Is(_) => 1,
        IDFilter::OneOf(ids) => ids.len(),
        IDFilter::Any => tree.exts().len(),
    }
}

#[inline]
pub fn for_each_ext<F>(tree: &DeviceTree, xid: &IDFilter<ExtensionID>, mut f: F) -> ControlFlow<()>
where
    F: FnMut(&Extension) -> ControlFlow<()>,
{
    match xid {
        IDFilter::Is(id) => {
            let Ok(fref) = tree.ext_index(id) else {
                return ControlFlow::Continue(());
            };
            if let Ok(floe) = tree.ext(fref) {
                return f(floe);
            }
            ControlFlow::Continue(())
        }
        IDFilter::OneOf(ids) => {
            for id in ids {
                let Ok(fref) = tree.ext_index(id) else {
                    continue;
                };
                if let Ok(floe) = tree.ext(fref) {
                    f(floe)?;
                }
            }
            ControlFlow::Continue(())
        }
        IDFilter::Any => {
            for floe in tree.exts() {
                let Some(floe) = floe else {
                    continue;
                };
                f(floe)?;
            }
            ControlFlow::Continue(())
        }
    }
}
