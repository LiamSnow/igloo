use crate::{
    dash::model::{ColorPickerVariant, Dashboard, Element},
    glacier::query::{Query, QueryFilter, QueryKind, QueryTarget},
};
use igloo_interface::{Component, ComponentType};
use maud::{Markup, html};
use rustc_hash::FxHashMap;
use smallvec::smallvec;

pub type CompDashQuery = (QueryFilter, QueryTarget, ComponentType);

impl Dashboard {
    pub fn render(
        &self,
        dash_id: &u16,
    ) -> Result<
        (
            Markup,
            Vec<(u16, CompDashQuery)>,
            FxHashMap<u16, CompDashQuery>,
        ),
        String,
    > {
        let mut elid = 0;
        let mut watchers = Vec::new();
        let mut setters = FxHashMap::default();

        Ok((
            html! {
                (self.child.render(dash_id, &mut elid, &mut watchers, &mut setters, &self.targets)?)
            },
            watchers,
            setters,
        ))
    }
}

impl Element {
    fn render(
        &self,
        dashid: &u16,
        elid: &mut u16,
        watchers: &mut Vec<(u16, CompDashQuery)>,
        setters: &mut FxHashMap<u16, CompDashQuery>,
        targets: &FxHashMap<String, QueryTarget>,
    ) -> Result<Markup, String> {
        Ok(match self {
            Element::Custom {
                name,
                selected_targets,
            } => todo!(),
            Element::If {
                condition,
                then,
                r#else,
            } => todo!(),
            Element::Repeat { count, each } => todo!(),
            Element::ForEach {} => todo!(),
            Element::HStack {
                justify,
                align,
                scroll,
                children,
            } => {
                let overflow = if *scroll { "overflow-x: auto;" } else { "" };

                let css = format!(
                    "display: flex; flex-direction: row; justify-content: {}; align-items: {}; {}",
                    justify, align, overflow
                );

                html! {
                    div style=(css) {
                        @for child in children {
                            (child.render(dashid, elid, watchers, setters, targets)?)
                        }
                    }
                }
            }
            Element::VStack {
                justify,
                align,
                scroll,
                children,
            } => {
                let overflow = if *scroll { "overflow-y: auto;" } else { "" };

                let css = format!(
                    "display: flex; flex-direction: column; justify-content: {}; align-items: {}; {}",
                    justify, align, overflow
                );

                html! {
                    div style=(css) {
                        @for child in children {
                            (child.render(dashid, elid, watchers, setters, targets)?)
                        }
                    }
                }
            }
            Element::Card { child } => {
                html! {
                    div class="card" {
                        (child.render(dashid, elid, watchers, setters, targets)?)
                    }
                }
            }
            Element::Tabs { pages } => {
                let tab_names: Vec<_> = pages.keys().collect();

                html! {
                    div class="tabs" {
                        div class="tab-headers" {
                            @for (i, name) in tab_names.iter().enumerate() {
                                button class=(format!("tab-header {}", if i == 0 { "active" } else { "" }))
                                    data-tab=(i) {
                                    (name)
                                }
                            }
                        }
                        div class="tab-content" {
                            @for (i, (_, elements)) in pages.iter().enumerate() {
                                div class=(format!("tab-pane {}", if i == 0 { "active" } else { "" }))
                                    data-tab=(i) {
                                    @for child in elements {
                                        (child.render(dashid, elid, watchers, setters, targets)?)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Element::Switch { binding, size } => {
                let class = format!("switch {size}");

                html! {
                    label class=(class) {
                        input type="checkbox" {}
                        span class="switch-slider" {}
                    }
                }
            }
            Element::Checkbox { binding, size } => {
                html! {
                    input type="checkbox" class=(size) { }
                }
            }
            Element::ToggleButton { binding, size } => {
                todo!()
            }
            Element::Icon {
                icon,
                icon_value,
                size,
            } => {
                // TODO add actual icons
                let size_class = size.to_string();

                html! {
                    i class=(format!("icon {} {}", icon, size_class)) {}
                }
            }
            Element::Button {
                on_click,
                size,
                variant,
                children,
            } => {
                let classes = format!("{} {}", size, variant);

                if let Some(on_click) = on_click {
                    let query = Query {
                        filter: on_click.1.clone(),
                        target: targets
                            .get(&on_click.0)
                            .ok_or(format!("Missing {}", on_click.0))?
                            .clone(),
                        kind: QueryKind::Set(smallvec![Component::Trigger]),
                    };
                }

                // TODO use set query

                html! {
                    button class=(classes) onclick="" {
                        @for child in children {
                            (child.render(dashid, elid, watchers, setters, targets)?)
                        }
                    }
                }
            }
            Element::Text {
                value,
                prefix,
                suffix,
                size,
            } => {
                if let Some(value) = value {
                    // TODO
                    // queries.push((
                    //     *next_id,
                    //     Query {
                    //         filter: value.1.clone(),
                    //         target: targets
                    //             .get(&on_click.0)
                    //             .ok_or(format!("Missing {}", on_click.0))?
                    //             .clone(),
                    //         kind: QueryKind::WatchAll(),
                    //     },
                    // ));

                    let res = html! {
                        p class=(size) id=(*elid) {
                            (prefix)(suffix)
                        }
                    };

                    *elid += 1;

                    res
                } else {
                    html! {
                        p class=(size) {
                            (prefix)(suffix)
                        }
                    }
                }
            }
            Element::TextInput {
                title,
                placeholder,
                binding,
                disable_validation,
                is_password,
                multi_line,
            } => {
                let input_type = if *is_password { "password" } else { "text" };

                html! {
                    div class="input-group" {
                        @if !title.is_empty() {
                            label { (title) }
                        }
                        @if *multi_line {
                            textarea placeholder=(placeholder) {}
                        } @else {
                            input type=(input_type) placeholder=(placeholder) {}
                        }
                    }
                }
            }

            Element::NumberInput {
                title,
                placeholder,
                binding,
                disable_validation,
            } => {
                html! {
                    div class="input-group" {
                        @if !title.is_empty() {
                            label { (title) }
                        }
                        input type="number" placeholder=(placeholder) {}
                    }
                }
            }
            Element::TimePicker { binding } => {
                html! {
                    div class="picker time-picker" {
                        input type="time" {}
                    }
                }
            }

            Element::DatePicker { binding } => {
                html! {
                    div class="picker date-picker" {
                        input type="date" {}
                    }
                }
            }

            Element::DateTimePicker { binding } => {
                html! {
                    div class="picker datetime-picker" {
                        input type="datetime-local" {}
                    }
                }
            }

            Element::DurationPicker { binding } => {
                html! {
                    div class="picker duration-picker" {
                        input type="time" placeholder="HH:MM:SS" {}
                    }
                }
            }

            Element::WeekdayPicker { binding, multi } => {
                let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

                html! {
                    div class="picker weekday-picker" {
                        @for day in weekdays {
                            label class="weekday-option" {
                                input type=(if *multi { "checkbox" } else { "radio" })
                                      name="weekday"
                                      value=(day) {}
                                span { (day) }
                            }
                        }
                    }
                }
            }

            Element::Slider {
                binding,
                disable_validation,
                min,
                max,
                step,
            } => {
                // TODO auto check for IntMax, etc.

                let min_val = min.as_ref().and_then(|c| match c {
                    Component::Int(v) => Some(v.to_string()),
                    Component::Float(v) => Some(v.to_string()),
                    _ => None,
                });

                let max_val = max.as_ref().and_then(|c| match c {
                    Component::Int(v) => Some(v.to_string()),
                    Component::Float(v) => Some(v.to_string()),
                    _ => None,
                });

                let step_val = match step {
                    Some(Component::Int(v)) => v.to_string(),
                    Some(Component::Float(v)) => v.to_string(),
                    _ => "any".to_string(),
                };

                let filter = binding.1.clone();
                let target = targets
                    .get(&binding.0)
                    .ok_or(format!("Missing {}", binding.0))?
                    .clone();

                watchers.push((*elid, (binding.1.clone(), target.clone(), binding.2)));
                *elid += 1;

                let call = format!("setValue(event, {dashid}, {elid})");
                setters.insert(*elid, (filter, target, binding.2));
                *elid += 1;

                html! {
                    input type="range"
                        min=[min_val]
                        max=[max_val]
                        step=(step_val)
                        oninput=(call)
                        id=(*elid-2);
                }
            }

            Element::ColorTemperaturePicker { binding } => {
                html! {
                    div class="picker color-temp-picker" {
                        input type="range"
                              min="2000"
                              max="6500"
                              class="temp-slider" {}
                        span class="temp-value" { "4000K" }
                    }
                }
            }

            Element::ColorPicker { binding, variant } => {
                let variant_class = match variant {
                    ColorPickerVariant::Circle => "circle",
                    ColorPickerVariant::HueSlider => "hue-slider",
                    ColorPickerVariant::HSL => "hsl",
                };

                html! {
                    div class=(format!("picker color-picker {}", variant_class)) {
                        input type="color" {}
                    }
                }
            }

            Element::TextSelect { binding, variant } => {
                html! {
                    div class=(format!("select-container {}", variant)) {
                        select {
                            option { "Option 1" }
                            option { "Option 2" }
                            option { "Option 3" }
                        }
                    }
                }
            }

            Element::ModeSelect { binding, variant } => {
                html! {
                    div class=(format!("select-container mode-select {}", variant)) {
                        select {
                            option { "Mode 1" }
                            option { "Mode 2" }
                            option { "Mode 3" }
                        }
                    }
                }
            }

            Element::CustomSelect {
                binding,
                variant,
                options,
            } => {
                html! {
                    div class=(format!("select-container custom-select {}", variant)) {
                        @if variant.to_string() == "dropdown" {
                            select {
                                @for (name, _value) in options {
                                    option { (name) }
                                }
                            }
                        } @else if variant.to_string() == "radio" {
                            div class="radio-group" {
                                @for (name, _value) in options {
                                    label class="radio-option" {
                                        input type="radio" name="custom-select" {}
                                        span { (name) }
                                    }
                                }
                            }
                        } @else {
                            div class="panel-options" {
                                @for (name, _value) in options {
                                    button class="panel-option" {
                                        (name)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Element::Chart => todo!(),
            Element::Table => todo!(),
            Element::VideoFeed => todo!(),
            Element::Link => todo!(),
            Element::Image => todo!(),
            Element::Collapsable => todo!(),
            Element::Hr => {
                html! {
                    hr {}
                }
            }
        })
    }
}
