use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::types::{IglooType, IglooValue};

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum ComparisonOp {
    #[display("==")]
    Eq,
    #[display("!=")]
    Neq,
    #[display(">")]
    Gt,
    #[display(">=")]
    Gte,
    #[display("<")]
    Lt,
    #[display("<=")]
    Lte,
    #[display("contains")]
    Contains,
}

impl ComparisonOp {
    pub fn can_eval(&self, a: &IglooType, b: &IglooType) -> bool {
        self.can_eval_without_cast(a, b) || choose_cast_direction(a, b).is_some()
    }

    pub fn can_eval_without_cast(&self, a: &IglooType, b: &IglooType) -> bool {
        use ComparisonOp::*;
        use IglooType::*;

        match self {
            Eq | Neq => a == b,
            Gt | Gte | Lt | Lte => a == b && matches!(a, Integer | Real | Text | Date | Time),
            Contains => {
                matches!(
                    (a, b),
                    (IntegerList, Integer)
                        | (RealList, Real)
                        | (TextList, Text)
                        | (BooleanList, Boolean)
                        | (ColorList, Color)
                        | (DateList, Date)
                        | (TimeList, Time)
                        | (FloeIDList, FloeID)
                        | (DeviceIDList, DeviceID)
                        | (GroupIDList, GroupID)
                        | (FloeSnapshotList, FloeSnapshot)
                        | (DeviceSnapshotList, DeviceSnapshot)
                        | (GroupSnapshotList, GroupSnapshot)
                        | (EntitySnapshotList, EntitySnapshot)
                        | (Text, Text)
                )
            }
        }
    }
}

impl ComparisonOp {
    pub fn eval(&self, a: &IglooValue, b: &IglooValue) -> Option<bool> {
        if self.can_eval_without_cast(&a.r#type(), &b.r#type()) {
            self.eval_without_cast(a, b)
        } else if let Some(direction) = choose_cast_direction(&a.r#type(), &b.r#type()) {
            match direction {
                CastDirection::AToB => {
                    let a_casted = a.clone().cast(b.r#type())?;
                    self.eval_without_cast(&a_casted, b)
                }
                CastDirection::BToA => {
                    let b_casted = b.clone().cast(a.r#type())?;
                    self.eval_without_cast(a, &b_casted)
                }
            }
        } else {
            None
        }
    }

    pub fn eval_without_cast(&self, a: &IglooValue, b: &IglooValue) -> Option<bool> {
        use ComparisonOp::*;
        use IglooValue::*;

        Some(match self {
            Eq => a == b,
            Neq => a != b,

            Gt => match (a, b) {
                (Integer(a), Integer(b)) => a > b,
                (Real(a), Real(b)) => a > b,
                (Text(a), Text(b)) => a > b,
                (Date(a), Date(b)) => a.days_since_epoch() > b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() > b.to_seconds(),
                _ => return None,
            },

            Gte => match (a, b) {
                (Integer(a), Integer(b)) => a >= b,
                (Real(a), Real(b)) => a >= b,
                (Text(a), Text(b)) => a >= b,
                (Date(a), Date(b)) => a.days_since_epoch() >= b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() >= b.to_seconds(),
                _ => return None,
            },

            Lt => match (a, b) {
                (Integer(a), Integer(b)) => a < b,
                (Real(a), Real(b)) => a < b,
                (Text(a), Text(b)) => a < b,
                (Date(a), Date(b)) => a.days_since_epoch() < b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() < b.to_seconds(),
                _ => return None,
            },

            Lte => match (a, b) {
                (Integer(a), Integer(b)) => a <= b,
                (Real(a), Real(b)) => a <= b,
                (Text(a), Text(b)) => a <= b,
                (Date(a), Date(b)) => a.days_since_epoch() <= b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() <= b.to_seconds(),
                _ => return None,
            },

            Contains => match (a, b) {
                (IntegerList(list), Integer(val)) => list.contains(val),
                (RealList(list), Real(val)) => list.contains(val),
                (TextList(list), Text(val)) => list.contains(val),
                (BooleanList(list), Boolean(val)) => list.contains(val),
                (ColorList(list), Color(val)) => list.contains(val),
                (DateList(list), Date(val)) => list.contains(val),
                (TimeList(list), Time(val)) => list.contains(val),
                (FloeIDList(list), FloeID(val)) => list.contains(val),
                (DeviceIDList(list), DeviceID(val)) => list.contains(val),
                (GroupIDList(list), GroupID(val)) => list.contains(val),
                (FloeSnapshotList(list), FloeSnapshot(val)) => list.contains(val),
                (DeviceSnapshotList(list), DeviceSnapshot(val)) => list.contains(val),
                (GroupSnapshotList(list), GroupSnapshot(val)) => list.contains(val),
                (EntitySnapshotList(list), EntitySnapshot(val)) => list.contains(val),
                (Text(haystack), Text(needle)) => haystack.contains(needle.as_str()),
                _ => return None,
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum CastDirection {
    AToB,
    BToA,
}

fn choose_cast_direction(a: &IglooType, b: &IglooType) -> Option<CastDirection> {
    use CastDirection::*;
    Some(match (a.can_lossless_cast(*b), b.can_lossless_cast(*a)) {
        (true, false) => AToB,
        (false, true) => BToA,
        (true, true) => {
            if a.type_width() > b.type_width() {
                BToA
            } else {
                AToB
            }
        }
        _ => match (a.can_lossy_cast(*b), b.can_lossy_cast(*a)) {
            (true, false) => AToB,
            (false, true) => BToA,
            (true, true) => {
                if a.type_width() > b.type_width() {
                    BToA
                } else {
                    AToB
                }
            }
            _ => return None,
        },
    })
}

impl IglooType {
    fn type_width(&self) -> u8 {
        use IglooType::*;
        match self {
            Real => 3,
            Integer => 2,
            Boolean => 1,

            RealList => 3,
            IntegerList => 2,
            BooleanList => 1,

            _ => 0,
        }
    }
}
