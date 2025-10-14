use crate::{
    dash::model::{Dashboard, Element},
    glacier::query::{QueryFilter, QueryTarget},
};
use igloo_interface::ComponentType;
use rustc_hash::FxHashMap;

pub type CompDashQuery = (QueryFilter, QueryTarget, ComponentType);

impl Dashboard {
    pub fn get_watchers(&self, dash_id: u16) -> Result<Vec<(u32, CompDashQuery)>, String> {
        let mut elid = (dash_id as u32) << 16;
        let mut watchers = Vec::new();
        self.child
            .add_watchers(&mut elid, &mut watchers, &self.targets)?;
        Ok(watchers)
    }
}

impl Element {
    fn add_watchers(
        &self,
        elid: &mut u32,
        watchers: &mut Vec<(u32, CompDashQuery)>,
        targets: &FxHashMap<String, QueryTarget>,
    ) -> Result<(), String> {
        match self {
            Element::Slider { binding, .. } => {
                let filter = binding.1.clone();
                let target = targets
                    .get(&binding.0)
                    .ok_or(format!("Missing {}", binding.0))?
                    .clone();

                watchers.push((*elid, (filter, target.clone(), binding.2)));
                *elid += 1;
            }
            Element::Custom {
                name,
                selected_targets,
            } => todo!(),
            Element::If { then, r#else, .. } => {
                for child in then {
                    child.add_watchers(elid, watchers, targets)?;
                }
                for child in r#else {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::Repeat { each, .. } => {
                for child in each {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::ForEach {} => todo!(),
            Element::HStack { children, .. } => {
                for child in children {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::VStack { children, .. } => {
                for child in children {
                    child.add_watchers(elid, watchers, targets)?;
                }
            }
            Element::Tabs { pages } => {
                for (_, page) in pages {
                    for child in page {
                        child.add_watchers(elid, watchers, targets)?;
                    }
                }
            }
            Element::Card { child } => {
                child.add_watchers(elid, watchers, targets)?;
            }
            Element::Switch { binding, size } => todo!(),
            Element::Checkbox { binding, size } => todo!(),
            Element::ToggleButton { binding, size } => todo!(),
            Element::Icon {
                icon,
                icon_value,
                size,
            } => todo!(),
            Element::Button {
                on_click,
                size,
                variant,
                children,
            } => todo!(),
            Element::Text {
                value,
                prefix,
                suffix,
                size,
            } => todo!(),
            Element::TextInput {
                title,
                placeholder,
                binding,
                disable_validation,
                is_password,
                multi_line,
            } => todo!(),
            Element::NumberInput {
                title,
                placeholder,
                binding,
                disable_validation,
            } => todo!(),
            Element::TimePicker { binding } => todo!(),
            Element::DatePicker { binding } => todo!(),
            Element::DateTimePicker { binding } => todo!(),
            Element::DurationPicker { binding } => todo!(),
            Element::WeekdayPicker { binding, multi } => todo!(),
            Element::ColorTemperaturePicker { binding } => todo!(),
            Element::ColorPicker { binding, variant } => todo!(),
            Element::TextSelect { binding, variant } => todo!(),
            Element::ModeSelect { binding, variant } => todo!(),
            Element::CustomSelect {
                binding,
                variant,
                options,
            } => todo!(),
            Element::Chart => todo!(),
            Element::Table => todo!(),
            Element::VideoFeed => todo!(),
            Element::Link => todo!(),
            Element::Image => todo!(),
            Element::Collapsable => todo!(),
            Element::Hr => todo!(),
        }

        Ok(())
    }
}
