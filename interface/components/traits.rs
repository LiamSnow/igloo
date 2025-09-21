use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Unitable {
    pub unit: Unit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Unit {
    Seconds,
    Minutes,
    Hours,
    Days,

    Celsius,
    Fahrenheit,
    Kelvin,

    Meters,
    Centimeters,
    Feet,
    Inches,

    Percent,

    Watts,
    Kilowatts,

    Pascal,
    Bar,
    Psi,

    Liters,
    Gallons,

    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sensor<T> {
    #[serde(flatten)]
    pub unitable: Unitable,
    pub value: T,
}

pub type FloatSensor = Sensor<f64>;
pub type IntSensor = Sensor<i32>;
pub type LongSensor = Sensor<i128>;
