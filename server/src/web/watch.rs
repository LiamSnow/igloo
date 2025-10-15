use std::collections::HashMap;

use igloo_interface::{
    ComponentType, QueryFilter, QueryTarget,
    dash::{DashElement, Dashboard},
};

pub struct ElementWatcher {
    pub watch_id: u32,
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub comp: ComponentType,
}

pub trait GetWatchers {
    fn attach_watchers(&mut self, dash_id: u16) -> Result<Vec<ElementWatcher>, String>;
}

impl GetWatchers for Dashboard {
    /// attaches `watch_id` to all needed components
    /// and returns all watch requests
    fn attach_watchers(&mut self, dash_id: u16) -> Result<Vec<ElementWatcher>, String> {
        let mut watch_id = (dash_id as u32) << 16;
        let mut watchers = Vec::new();
        self.child
            .add_watchers(&mut watch_id, &mut watchers, &self.targets)?;
        Ok(watchers)
    }
}

pub trait AddWatchers {
    fn add_watchers(
        &mut self,
        watch_id: &mut u32,
        watchers: &mut Vec<ElementWatcher>,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(), String>;
}

impl AddWatchers for DashElement {
    fn add_watchers(
        &mut self,
        watch_id: &mut u32,
        watchers: &mut Vec<ElementWatcher>,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(), String> {
        match self {
            DashElement::Slider(e) => {
                let filter = e.binding.filter.clone();
                let target = targets
                    .get(&e.binding.target)
                    .ok_or(format!("Missing {}", e.binding.target))?
                    .clone();

                e.watch_id = Some(*watch_id);

                watchers.push(ElementWatcher {
                    watch_id: *watch_id,
                    filter,
                    target: target.clone(),
                    comp: e.binding.comp_type,
                });

                *watch_id += 1;
            }
            DashElement::ColorPicker(e) => {
                let filter = e.binding.filter.clone();
                let target = targets
                    .get(&e.binding.target)
                    .ok_or(format!("Missing {}", e.binding.target))?
                    .clone();

                watchers.push(ElementWatcher {
                    watch_id: *watch_id,
                    filter,
                    target: target.clone(),
                    comp: ComponentType::Color,
                });

                *watch_id += 1;
            }
            DashElement::If(e) => {
                // TODO watch expression too
                for child in &mut e.then {
                    child.add_watchers(watch_id, watchers, targets)?;
                }
                for child in &mut e.r#else {
                    child.add_watchers(watch_id, watchers, targets)?;
                }
            }
            DashElement::Repeat(e) => {
                for child in &mut e.each {
                    child.add_watchers(watch_id, watchers, targets)?;
                }
            }
            DashElement::HStack(e) => {
                for child in &mut e.children {
                    child.add_watchers(watch_id, watchers, targets)?;
                }
            }
            DashElement::VStack(e) => {
                for child in &mut e.children {
                    child.add_watchers(watch_id, watchers, targets)?;
                }
            }
            DashElement::Tabs(e) => {
                for page in e.pages.values_mut() {
                    for child in page {
                        child.add_watchers(watch_id, watchers, targets)?;
                    }
                }
            }
            DashElement::Card(e) => {
                e.child.add_watchers(watch_id, watchers, targets)?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}
