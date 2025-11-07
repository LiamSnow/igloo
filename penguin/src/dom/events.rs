use crate::app::APP;
use crate::dom::builder::DomBuilder;
use crate::dom::node::DomNode;
use crate::graph::input::WebInputType;
use crate::viewport::{ClientPoint, toolbar::ToolbarButton};
use igloo_interface::penguin::graph::{PenguinNodeID, PenguinWireID};
use igloo_interface::penguin::{PenguinNodeDefnRef, PenguinPinRef};
use std::any::Any;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    ClipboardEvent, Element, EventTarget as WebEventTarget, HtmlElement, KeyboardEvent, MouseEvent,
    ResizeObserver, WheelEvent,
};

#[derive(Debug, Clone)]
pub struct Event {
    pub value: EventValue,
    pub target: EventTarget,
}

#[derive(Debug, Clone)]
pub enum EventValue {
    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseClick(MouseEvent),
    MouseDoubleClick(MouseEvent),
    ContextMenu(MouseEvent),
    Wheel(WheelEvent),
    KeyDown(KeyboardEvent),
    Copy(ClipboardEvent),
    Paste(ClipboardEvent),
    Cut(ClipboardEvent),
    Input(String),
    InputNoValue,
    Resize((i32, i32)),
}

#[derive(Debug, Clone)]
pub enum EventTarget {
    Global,
    Node(PenguinNodeID),
    Wire(PenguinWireID),
    Pin(PenguinPinRef),
    MenuBackdrop,
    MenuSearch,
    MenuSearchItem(PenguinNodeDefnRef),
    ToolbarButton(ToolbarButton),
    NodeInput(PenguinNodeID, WebInputType),
    NodeVariadic(PenguinNodeID, String),
}

pub trait Clientable {
    fn client_pos(&self) -> ClientPoint;
}

impl Clientable for MouseEvent {
    fn client_pos(&self) -> ClientPoint {
        ClientPoint::new(self.client_x(), self.client_y())
    }
}

fn add_listener<E, F>(
    element: &impl AsRef<WebEventTarget>,
    event_name: &str,
    event_target: EventTarget,
    handler: F,
) -> Box<dyn Any>
where
    E: wasm_bindgen::convert::FromWasmAbi + JsCast + 'static,
    F: Fn(E) -> EventValue + 'static,
{
    let closure = Closure::wrap(Box::new(move |e: E| {
        let we = e.unchecked_ref::<web_sys::Event>();
        we.stop_propagation();

        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.handle(Event {
                    value: handler(e),
                    target: event_target.clone(),
                });
            }
        });
    }) as Box<dyn FnMut(E)>);

    element
        .as_ref()
        .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())
        .unwrap();

    Box::new(closure)
}

fn add_mouseevent_conditional_listener<E, F, C>(
    element: &impl AsRef<WebEventTarget>,
    event_name: &str,
    event_target: EventTarget,
    handler: F,
    condition: C,
) -> Box<dyn Any>
where
    E: wasm_bindgen::convert::FromWasmAbi + JsCast + Clone + 'static,
    F: Fn(E) -> EventValue + 'static,
    C: Fn(&E) -> bool + 'static,
{
    let closure = Closure::wrap(Box::new(move |e: E| {
        let we = e.unchecked_ref::<web_sys::Event>();
        we.stop_propagation();

        if condition(&e) {
            APP.with(|app| {
                if let Some(app) = app.borrow_mut().as_mut() {
                    app.handle(Event {
                        value: handler(e),
                        target: event_target.clone(),
                    });
                }
            });
        }
    }) as Box<dyn FnMut(E)>);

    element
        .as_ref()
        .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())
        .unwrap();

    Box::new(closure)
}

fn add_resize_listener(
    observe_el: &Element,
    size_element: HtmlElement,
    event_target: EventTarget,
) -> (Box<dyn Any>, ResizeObserver) {
    let onresize = Closure::wrap(Box::new(move |_: web_sys::Event| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.handle(Event {
                    value: EventValue::Resize((
                        size_element.offset_width(),
                        size_element.offset_height(),
                    )),
                    target: event_target.clone(),
                });
            }
        });
    }) as Box<dyn FnMut(_)>);

    let observer = ResizeObserver::new(onresize.as_ref().unchecked_ref()).unwrap();
    observer.observe(observe_el);

    (Box::new(onresize), observer)
}

impl<T> DomNode<T> {
    fn get_event_target(&self) -> EventTarget {
        self.event_target
            .clone()
            .expect("EventTarget not set. Call .event_target() first")
    }

    pub fn listen_mousedown(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "mousedown", target, |e: MouseEvent| {
            EventValue::MouseDown(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_mousedown_conditional(
        &mut self,
        condition: impl Fn(&MouseEvent) -> bool + 'static,
    ) {
        let target = self.get_event_target();
        let closure = add_mouseevent_conditional_listener(
            &self.element,
            "mousedown",
            target,
            |e: MouseEvent| EventValue::MouseDown(e),
            condition,
        );
        self.add_closure(closure);
    }

    pub fn listen_mouseup(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "mouseup", target, |e: MouseEvent| {
            EventValue::MouseUp(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_mousemove(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "mousemove", target, |e: MouseEvent| {
            EventValue::MouseMove(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_click(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "click", target, |e: MouseEvent| {
            EventValue::MouseClick(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_dblclick(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "dblclick", target, |e: MouseEvent| {
            EventValue::MouseDoubleClick(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_contextmenu(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "contextmenu", target, |e: MouseEvent| {
            EventValue::ContextMenu(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_wheel(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "wheel", target, |e: WheelEvent| {
            EventValue::Wheel(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_keydown(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "keydown", target, |e: KeyboardEvent| {
            EventValue::KeyDown(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_input<F: Fn() -> String + 'static>(&mut self, get_value: F) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "input", target, move |_: web_sys::Event| {
            EventValue::Input(get_value())
        });
        self.add_closure(closure);
    }

    pub fn listen_input_no_value(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "input", target, |_: web_sys::Event| {
            EventValue::InputNoValue
        });
        self.add_closure(closure);
    }

    pub fn listen_copy(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "copy", target, |e: ClipboardEvent| {
            EventValue::Copy(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_paste(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "paste", target, |e: ClipboardEvent| {
            EventValue::Paste(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_cut(&mut self) {
        let target = self.get_event_target();
        let closure = add_listener(&self.element, "cut", target, |e: ClipboardEvent| {
            EventValue::Cut(e)
        });
        self.add_closure(closure);
    }

    pub fn listen_resize(&mut self, observe_el: &Element) {
        let target = self.get_event_target();
        let element = self.element.clone().dyn_into::<HtmlElement>().unwrap();
        let (closure, observer) = add_resize_listener(observe_el, element, target);

        self.add_closure(closure);
        self.set_observer(observer);
    }
}

impl<T> DomBuilder<T> {
    pub fn listen_mousedown(mut self) -> Self {
        self.node.listen_mousedown();
        self
    }

    #[allow(dead_code)]
    pub fn listen_mousedown_conditional(
        mut self,
        condition: impl Fn(&MouseEvent) -> bool + 'static,
    ) -> Self {
        self.node.listen_mousedown_conditional(condition);
        self
    }

    pub fn listen_mouseup(mut self) -> Self {
        self.node.listen_mouseup();
        self
    }

    pub fn listen_mousemove(mut self) -> Self {
        self.node.listen_mousemove();
        self
    }

    pub fn listen_click(mut self) -> Self {
        self.node.listen_click();
        self
    }

    #[allow(dead_code)]
    pub fn listen_dblclick(mut self) -> Self {
        self.node.listen_dblclick();
        self
    }

    pub fn listen_contextmenu(mut self) -> Self {
        self.node.listen_contextmenu();
        self
    }

    pub fn listen_wheel(mut self) -> Self {
        self.node.listen_wheel();
        self
    }

    pub fn listen_keydown(mut self) -> Self {
        self.node.listen_keydown();
        self
    }

    #[allow(dead_code)]
    pub fn listen_input<F: Fn() -> String + 'static>(mut self, get_value: F) -> Self {
        self.node.listen_input(get_value);
        self
    }

    pub fn listen_input_no_value(mut self) -> Self {
        self.node.listen_input_no_value();
        self
    }

    pub fn listen_copy(mut self) -> Self {
        self.node.listen_copy();
        self
    }

    pub fn listen_paste(mut self) -> Self {
        self.node.listen_paste();
        self
    }

    pub fn listen_cut(mut self) -> Self {
        self.node.listen_cut();
        self
    }
}
