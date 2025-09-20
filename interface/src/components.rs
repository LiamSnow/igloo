use jiff::{
    SignedDuration,
    civil::{Date, DateTime, Time},
};
use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{standards::Standard, types::Type};

#[derive(Debug, Serialize, Deserialize)]
pub struct Components(HashMap<String, Component>);

#[derive(Debug, Serialize, Deserialize)]
pub enum Component {
    Standards(Vec<Standard>),

    // -- primitives -- \\
    Int(i32),
    Float(f64),
    Long(i128),

    String(String),
    Bool(bool),
    Trigger,
    Uuid(Uuid),
    Binary(Vec<u8>),

    Date(Date),
    Time(Time),
    DateTime(DateTime),
    // Weekday(Weekday),
    Duration(SignedDuration),

    Color(Color),

    // -- composites -- \\
    Object(HashMap<String, Component>),

    IntList(Vec<i32>),
    FloatList(Vec<f64>),
    LongList(Vec<i128>),
    StringList(Vec<String>),
    BoolList(Vec<bool>),
    UuidList(Vec<Uuid>),
    BinaryList(Vec<Vec<u8>>),
    DateList(Vec<Date>),
    TimeList(Vec<Time>),
    DateTimeList(Vec<DateTime>),
    DurationList(Vec<SignedDuration>),
    ColorList(Vec<Color>),

    MixedList(Vec<Component>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Components {
    pub fn has(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Component> {
        self.0.get(name)
    }

    pub fn add(&mut self, name: String, component: Component) {
        self.0.insert(name, component);
    }

    pub fn remove(&mut self, name: &str) -> Option<Component> {
        self.0.remove(name)
    }

    /// returns if contains a required property, IE
    /// element must exist and conform to the type
    pub fn has_req_comp_of<'a>(&self, name: &str, typ: Type<'a>) -> bool {
        match self.get(name) {
            Some(comp) => comp.to_type() == typ,
            None => false,
        }
    }

    /// returns if contains an optional property, IE
    /// if the component exists, it must conform to the type
    /// BUT if it doesn't exist thats fine
    pub fn has_opt_comp_of<'a>(&self, name: &str, typ: Type<'a>) -> bool {
        match self.get(name) {
            Some(comp) => comp.to_type() == typ,
            None => true,
        }
    }

    /// checks if a required String component exists and matches one of the given options, IE
    /// element must exist and conform
    pub fn has_req_string_of(&self, name: &str, options: Vec<&str>) -> bool {
        match self.get(name) {
            // comp exists AND is a String
            Some(Component::String(s)) => options.contains(&s.as_str()),
            // comp doesn't exist or wrong type
            _ => false,
        }
    }

    /// checks if an optional String component exists and matches one of the given options, IE
    /// if the component exists, it must conform
    /// BUT if it doesn't exist thats fine
    pub fn has_opt_string_of(&self, name: &str, options: Vec<&str>) -> bool {
        match self.get(name) {
            // comp exists AND is a String
            Some(Component::String(s)) => options.contains(&s.as_str()),
            // comp exists, but wrong type
            Some(_) => false,
            // comp doesn't exist
            None => true,
        }
    }

    /// checks if a required List component exists and matches the type, IE
    /// element must exist and conform
    pub fn has_req_list_of<'a>(&self, name: &str, wanted_type: Type<'a>) -> bool {
        match self.get(name) {
            Some(comp) => match comp.to_type() {
                // exists AND a list
                Type::List(actual_type, _) => *actual_type == wanted_type,
                // exists, but not list
                _ => false,
            },
            // doesn't exist
            _ => false,
        }
    }

    /// checks if an optional List component exists and matches the type, IE
    /// if the component exists, it must conform
    /// BUT if it doesn't exist thats fine
    pub fn has_opt_list_of<'a>(&self, name: &str, wanted_type: Type<'a>) -> bool {
        match self.get(name) {
            Some(comp) => match comp.to_type() {
                // exists AND a list
                Type::List(actual_type, _) => *actual_type == wanted_type,
                // exists, but not list
                _ => false,
            },
            // doesn't exist
            _ => true,
        }
    }
}
