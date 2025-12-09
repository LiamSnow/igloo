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
            let Ok(xindex) = tree.ext_index(id) else {
                return ControlFlow::Continue(());
            };
            if let Ok(ext) = tree.ext(xindex) {
                return f(ext);
            }
            ControlFlow::Continue(())
        }
        IDFilter::OneOf(ids) => {
            for id in ids {
                let Ok(xindex) = tree.ext_index(id) else {
                    continue;
                };
                if let Ok(ext) = tree.ext(xindex) {
                    f(ext)?;
                }
            }
            ControlFlow::Continue(())
        }
        IDFilter::Any => {
            for ext in tree.exts() {
                let Some(ext) = ext else {
                    continue;
                };
                f(ext)?;
            }
            ControlFlow::Continue(())
        }
    }
}
