use igloo_interface::{
    PenguinNodeDefnRef, PenguinPinRef,
    graph::{PenguinNodeID, PenguinWireID},
};
use std::any::Any;
use wasm_bindgen::{JsCast, JsValue, convert::FromWasmAbi, prelude::Closure};
use web_sys::{
    ClipboardEvent, Document, Element, HtmlElement, KeyboardEvent, MouseEvent, ResizeObserver,
    WheelEvent,
};

use crate::{
    app::APP,
    graph::input::WebInputType,
    viewport::{ClientPoint, toolbar::ToolbarButton},
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
    ContextMenu(MouseEvent),
    Wheel(WheelEvent),
    KeyDown(KeyboardEvent),
    Copy(ClipboardEvent),
    Paste(ClipboardEvent),
    Cut(ClipboardEvent),
    Input(String),
    Resize((i32, i32)),
}

#[derive(Debug, Clone)]
pub enum EventTarget {
    Global,
    Node(PenguinNodeID),
    Wire(PenguinWireID),
    Pin(PenguinPinRef),
    MenuBackdrop,
    MenuSearchItem(PenguinNodeDefnRef),
    ToolbarButton(ToolbarButton),
    NodeInput(PenguinNodeID, WebInputType),
    NodeVariadic(PenguinNodeID, String),
}

#[derive(Debug, Default)]
pub struct Listeners {
    closures: Vec<Box<dyn Any>>,
    observer: Option<ResizeObserver>,
}

impl Drop for Listeners {
    fn drop(&mut self) {
        if let Some(observer) = &self.observer {
            observer.disconnect();
        }
    }
}

impl Listeners {
    pub fn new(capacity: usize) -> Self {
        Self {
            closures: Vec::with_capacity(capacity),
            observer: None,
        }
    }

    fn add<E, F>(
        &mut self,
        element: &impl AsRef<web_sys::EventTarget>,
        event_name: &str,
        event_target: EventTarget,
        handler: F,
    ) -> Result<(), JsValue>
    where
        E: FromWasmAbi + JsCast + 'static,
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
            .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;

        self.closures.push(Box::new(closure));

        Ok(())
    }

    pub fn add_resize(
        &mut self,
        attached_element: &Element,
        size_element: HtmlElement,
        event_target: EventTarget,
    ) -> Result<(), JsValue> {
        if self.observer.is_some() {
            panic!("Cannot have multiple resize observers on the same element!");
        }

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

        let o = ResizeObserver::new(onresize.as_ref().unchecked_ref())?;
        o.observe(attached_element);
        self.closures.push(Box::new(onresize));
        self.observer = Some(o);
        Ok(())
    }
}

pub struct ListenerBuilder<'a, E: AsRef<web_sys::EventTarget>> {
    element: &'a E,
    target: EventTarget,
    listeners: Listeners,
}

impl<'a, E: AsRef<web_sys::EventTarget>> ListenerBuilder<'a, E> {
    pub fn new(element: &'a E, target: EventTarget) -> Self {
        Self {
            element,
            target,
            listeners: Listeners::new(9),
        }
    }

    pub fn with_capacity(element: &'a E, target: EventTarget, capacity: usize) -> Self {
        Self {
            element,
            target,
            listeners: Listeners::new(capacity),
        }
    }

    pub fn add_mousemove(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "mousemove",
            self.target.clone(),
            |e: MouseEvent| EventValue::MouseMove(e),
        )?;
        Ok(self)
    }

    pub fn add_mousedown(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "mousedown",
            self.target.clone(),
            |e: MouseEvent| EventValue::MouseDown(e),
        )?;
        Ok(self)
    }

    pub fn add_mouseup(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "mouseup",
            self.target.clone(),
            |e: MouseEvent| EventValue::MouseUp(e),
        )?;
        Ok(self)
    }

    pub fn add_mouseclick(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "click",
            self.target.clone(),
            |e: MouseEvent| EventValue::MouseClick(e),
        )?;
        Ok(self)
    }

    pub fn add_contextmenu(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "contextmenu",
            self.target.clone(),
            |e: MouseEvent| EventValue::ContextMenu(e),
        )?;
        Ok(self)
    }

    pub fn add_wheel(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "wheel",
            self.target.clone(),
            |e: WheelEvent| EventValue::Wheel(e),
        )?;
        Ok(self)
    }

    pub fn add_keydown(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "keydown",
            self.target.clone(),
            |e: KeyboardEvent| EventValue::KeyDown(e),
        )?;
        Ok(self)
    }

    pub fn add_copy(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "copy",
            self.target.clone(),
            |e: ClipboardEvent| EventValue::Copy(e),
        )?;
        Ok(self)
    }

    pub fn add_paste(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "paste",
            self.target.clone(),
            |e: ClipboardEvent| EventValue::Paste(e),
        )?;
        Ok(self)
    }

    pub fn add_cut(mut self) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "cut",
            self.target.clone(),
            |e: ClipboardEvent| EventValue::Cut(e),
        )?;
        Ok(self)
    }

    pub fn add_input<F: Fn() -> String + 'static>(mut self, get_value: F) -> Result<Self, JsValue> {
        self.listeners.add(
            self.element,
            "input",
            self.target.clone(),
            move |_: web_sys::Event| EventValue::Input(get_value()),
        )?;
        Ok(self)
    }

    pub fn build(self) -> Listeners {
        self.listeners
    }
}

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

pub trait Clientable {
    fn client_pos(&self) -> ClientPoint;
}

impl Clientable for MouseEvent {
    fn client_pos(&self) -> ClientPoint {
        ClientPoint::new(self.client_x(), self.client_y())
    }
}
