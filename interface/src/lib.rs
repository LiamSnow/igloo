use jiff::{
    SignedDuration,
    civil::{Date, DateTime, Time},
};
use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

pub type ObjectType = HashMap<String, Type>;
pub type EnumType = Vec<String>;
pub type TupleType = Vec<Type>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Float,
    Int,
    Long,

    String,
    Bool,
    Trigger,
    Uuid,
    Binary,

    Date,
    Time,
    Datetime,
    // Weekday,
    Duration,

    Color,

    // -- composites -- \\
    List(Box<Type>),
    Tuple(TupleType),
    Optional(Box<Type>),
    Object(ObjectType),
    /// defined type, use function ex `Type::light()`
    Custom(String, ObjectType),
    Enum(EnumType),
}

impl Type {
    fn fields_to_map(fields: Vec<(&str, Type)>) -> ObjectType {
        let mut map = HashMap::new();
        for (field, typ) in fields {
            map.insert(field.to_string(), typ);
        }
        map
    }

    pub fn object(fields: Vec<(&str, Type)>) -> Self {
        Type::Object(Type::fields_to_map(fields))
    }

    pub fn custom(name: &str, fields: Vec<(&str, Type)>) -> Self {
        Type::Custom(name.to_string(), Type::fields_to_map(fields))
    }

    pub fn list(typ: Type) -> Self {
        Type::List(Box::new(typ))
    }

    pub fn optional(typ: Type) -> Self {
        Type::Optional(Box::new(typ))
    }

    pub fn light() -> Self {
        use Type::*;
        Type::custom(
            "light",
            vec![
                ("on", Bool),
                ("brightness", Type::optional(Float)),
                ("color_temp", Type::optional(Float)),
                ("color", Type::optional(Color)),
            ],
        )
    }

    pub fn float_sensor() -> Self {
        use Type::*;
        Type::custom(
            "float_sensor",
            vec![
                ("unit", String),
                ("icon", Type::optional(String)),
                ("value", Float),
            ],
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Float(f64),
    Int(i32),
    Long(i128),

    String(String),
    Bool(bool),
    Trigger,
    Uuid(Uuid),
    Binary(Vec<u8>),

    Date(Date),
    Time(Time),
    Datetime(DateTime),
    // Weekday(Weekday),
    Duration(SignedDuration),

    Color(Color),

    // -- composites -- \\
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Optional(Option<Box<Value>>),
    Object(HashMap<String, Value>),
    Enum(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
