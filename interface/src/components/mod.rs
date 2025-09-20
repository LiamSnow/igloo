use jiff::{
    SignedDuration,
    civil::{Date, DateTime, Time},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::defs::*;
use crate::traits::*;

pub mod defs;
pub mod traits;

#[derive(Debug, Serialize, Deserialize)]
pub struct Components(HashMap<String, Component>);

#[derive(Debug, Serialize, Deserialize)]
pub enum Component {
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

    FloatSensor(FloatSensor),
    IntSensor(IntSensor),
    LongSensor(LongSensor),

    Temperature(Temperature),
    ClimateMode(ClimateMode),
    Thermostat(Thermostat),

    Light(Light),
    Fan(Fan),
    Switch(Switchable),
    Dimmer(Dimmable),
    Color(Colorable),
    Lock(Lock),
    MediaPlayer(MediaPlayer),
    Cover(Cover),
    BinarySensor(BinarySensor),
    Button(Button),
    Number(Number),
    Alarm(Alarm),
    Camera(Camera),
    Vacuum(Vacuum),
    Select(Select),
    Text(Text),

    Object(HashMap<String, Component>),
    List(Vec<Component>),
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
}
