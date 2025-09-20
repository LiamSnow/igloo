use std::collections::HashMap;

use crate::Component;

#[derive(Debug, PartialEq, Eq)]
pub enum Type<'a> {
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
    DateTime,
    // Weekday,
    Duration,

    Color,

    // -- composites -- \\
    /// type of list, length
    List(Box<Type<'a>>, usize),
    /// length
    MixedList(usize),
    Object(HashMap<&'a str, Type<'a>>),
}

impl Component {
    pub fn to_type<'a>(&'a self) -> Type<'a> {
        use Component::*;
        match self {
            Float(_) => Type::Float,
            Int(_) => Type::Int,
            Long(_) => Type::Long,
            String(_) => Type::String,
            Bool(_) => Type::Bool,
            Trigger => Type::Trigger,
            Uuid(_) => Type::Uuid,
            Binary(_) => Type::Binary,
            Date(_) => Type::Date,
            DateTime(_) => Type::DateTime,
            Time(_) => Type::Time,
            Duration(_) => Type::Duration,
            Color(_) => Type::Color,
            Object(map) => {
                let mut res = HashMap::new();
                for (k, v) in map {
                    res.insert(k.as_str(), v.to_type());
                }
                Type::Object(res)
            }
            IntList(l) => Type::List(Box::new(Type::Int), l.len()),
            FloatList(l) => Type::List(Box::new(Type::Float), l.len()),
            LongList(l) => Type::List(Box::new(Type::Long), l.len()),
            StringList(l) => Type::List(Box::new(Type::String), l.len()),
            BoolList(l) => Type::List(Box::new(Type::Bool), l.len()),
            UuidList(l) => Type::List(Box::new(Type::Uuid), l.len()),
            BinaryList(l) => Type::List(Box::new(Type::Binary), l.len()),
            DateList(l) => Type::List(Box::new(Type::Date), l.len()),
            TimeList(l) => Type::List(Box::new(Type::Time), l.len()),
            DateTimeList(l) => Type::List(Box::new(Type::DateTime), l.len()),
            DurationList(l) => Type::List(Box::new(Type::Duration), l.len()),
            ColorList(l) => Type::List(Box::new(Type::Color), l.len()),
            MixedList(l) => Type::MixedList(l.len()),
        }
    }
}
