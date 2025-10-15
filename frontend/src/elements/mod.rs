use std::collections::HashMap;

use igloo_interface::{
    Component, ComponentType, QueryFilter, QueryTarget,
    dash::{Element, HStackElement, SliderElement, VStackElement},
};
use maud::{Markup, html};

pub struct HandlerData {
    pub elid: u32,
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub comp_type: ComponentType,
}

pub trait RenderElement {
    fn render(
        &self,
        elid: &mut u32,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(Markup, Vec<HandlerData>), String>;
}

impl RenderElement for Element {
    fn render(
        &self,
        elid: &mut u32,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(Markup, Vec<HandlerData>), String> {
        match self {
            Element::HStack(e) => e.render(elid, targets),
            Element::VStack(e) => e.render(elid, targets),
            Element::Slider(e) => e.render(elid, targets),
            _ => todo!(),
        }
    }
}

impl RenderElement for HStackElement {
    fn render(
        &self,
        elid: &mut u32,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(Markup, Vec<HandlerData>), String> {
        let overflow = if self.scroll { "overflow-x: auto;" } else { "" };
        let css = format!(
            "display: flex; flex-direction: row; justify-content: {}; align-items: {}; {}",
            self.justify, self.align, overflow
        );

        let mut all_handlers = Vec::new();
        let mut child_markups = Vec::new();

        for child in &self.children {
            let (markup, handlers) = child.render(elid, targets)?;
            child_markups.push(markup);
            all_handlers.extend(handlers);
        }

        let markup = html! {
            div style=(css) {
                @for child_markup in child_markups {
                    (child_markup)
                }
            }
        };

        Ok((markup, all_handlers))
    }
}

impl RenderElement for VStackElement {
    fn render(
        &self,
        elid: &mut u32,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(Markup, Vec<HandlerData>), String> {
        let overflow = if self.scroll { "overflow-y: auto;" } else { "" };
        let css = format!(
            "display: flex; flex-direction: column; justify-content: {}; align-items: {}; {}",
            self.justify, self.align, overflow
        );

        let mut all_handlers = Vec::new();
        let mut child_markups = Vec::new();

        for child in &self.children {
            let (markup, handlers) = child.render(elid, targets)?;
            child_markups.push(markup);
            all_handlers.extend(handlers);
        }

        let markup = html! {
            div style=(css) {
                @for child_markup in child_markups {
                    (child_markup)
                }
            }
        };

        Ok((markup, all_handlers))
    }
}

impl RenderElement for SliderElement {
    fn render(
        &self,
        elid: &mut u32,
        targets: &HashMap<String, QueryTarget>,
    ) -> Result<(Markup, Vec<HandlerData>), String> {
        let min_val = self.min.as_ref().and_then(|c| match c {
            Component::Int(v) => Some(v.to_string()),
            Component::Float(v) => Some(v.to_string()),
            _ => None,
        });

        let max_val = self.max.as_ref().and_then(|c| match c {
            Component::Int(v) => Some(v.to_string()),
            Component::Float(v) => Some(v.to_string()),
            _ => None,
        });

        let step_val = match self.step {
            Some(Component::Int(v)) => v.to_string(),
            Some(Component::Float(v)) => v.to_string(),
            _ => "any".to_string(),
        };

        let this_elid = *elid;
        *elid += 1;

        let markup = html! {
            input type="range"
                min=[min_val]
                max=[max_val]
                step=(step_val)
                id=(this_elid);
        };

        let handlers = vec![HandlerData {
            elid: this_elid,
            filter: self.binding.filter.clone(),
            target: targets
                .get(&self.binding.target)
                .ok_or(format!("Missing {}", self.binding.target))?
                .clone(),
            comp_type: self.binding.comp_type,
        }];

        Ok((markup, handlers))
    }
}
