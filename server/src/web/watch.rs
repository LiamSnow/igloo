use std::collections::HashMap;

use igloo_interface::{
    ComponentType, QueryFilter, QueryTarget,
    dash::{Dashboard, Element},
};

pub struct ElementWatcher {
    pub elid: u32,
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub comp: ComponentType,
}

pub trait GetWatchers {
    fn get_watchers(&self, dash_id: u16) -> Result<Vec<ElementWatcher>, String>;
}

impl GetWatchers for Dashboard {
    fn get_watchers(&self, dash_id: u16) -> Result<Vec<ElementWatcher>, String> {
        let mut elid = (dash_id as u32) << 16;
        let mut watchers = Vec::new();
        self.child
            .add_watchers(&mut elid, &mut watchers, &self.targets)?;
        Ok(watchers)
    }
}

pub trait AddWatchers {
    fn add_watchers(
        &self,
        elid: &mut u32,
        watchers: &mut Vec<ElementWatcher>,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(), String>;
}

impl AddWatchers for Element {
    fn add_watchers(
        &self,
        elid: &mut u32,
        watchers: &mut Vec<ElementWatcher>,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(), String> {
        match self {
            Element::Slider(e) => {
                let filter = e.binding.filter.clone();
                let target = targets
                    .get(&e.binding.target)
                    .ok_or(format!("Missing {}", e.binding.target))?
                    .clone();

                watchers.push(ElementWatcher {
                    elid: *elid,
                    filter,
                    target: target.clone(),
                    comp: e.binding.comp_type,
                });

                *elid += 1;
            }
            Element::If(e) => {
                for child in &e.then {
                    child.add_watchers(elid, watchers, targets)?;
                }
                for child in &e.r#else {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::Repeat(e) => {
                for child in &e.each {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::HStack(e) => {
                for child in &e.children {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::VStack(e) => {
                for child in &e.children {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::Tabs(e) => {
                for (_, page) in &e.pages {
                    for child in page {
                        child.add_watchers(elid, watchers, targets)?;
                    }
                }
            }
            Element::Card(e) => {
                e.child.add_watchers(elid, watchers, targets)?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}
