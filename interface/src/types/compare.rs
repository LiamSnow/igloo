use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::types::{IglooType, IglooValue, cast::CastDirection};

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
    pub fn can_eval(&self, lhs: &IglooType, rhs: &IglooType) -> bool {
        self.can_eval_without_cast(lhs, rhs) || choose_cast_direction(lhs, rhs).is_some()
    }

    pub fn can_eval_without_cast(&self, lhs: &IglooType, rhs: &IglooType) -> bool {
        use ComparisonOp::*;
        use IglooType::*;

        match self {
            Eq | Neq => lhs == rhs,
            Gt | Gte | Lt | Lte => lhs == rhs && matches!(lhs, Integer | Real | Text | Date | Time),
            Contains => {
                matches!(
                    (lhs, rhs),
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
    pub fn eval(&self, lhs: &IglooValue, rhs: &IglooValue) -> Option<bool> {
        if self.can_eval_without_cast(&lhs.r#type(), &rhs.r#type()) {
            self.eval_without_cast(lhs, rhs)
        } else if let Some(direction) = choose_cast_direction(&lhs.r#type(), &rhs.r#type()) {
            match direction {
                CastDirection::LTR => {
                    let a_casted = lhs.clone().cast(rhs.r#type())?;
                    self.eval_without_cast(&a_casted, rhs)
                }
                CastDirection::RTL => {
                    let b_casted = rhs.clone().cast(lhs.r#type())?;
                    self.eval_without_cast(lhs, &b_casted)
                }
            }
        } else {
            None
        }
    }

    pub fn eval_without_cast(&self, lhs: &IglooValue, rhs: &IglooValue) -> Option<bool> {
        use ComparisonOp::*;
        use IglooValue::*;

        Some(match self {
            Eq => lhs == rhs,
            Neq => lhs != rhs,

            Gt => match (lhs, rhs) {
                (Integer(a), Integer(b)) => a > b,
                (Real(a), Real(b)) => a > b,
                (Text(a), Text(b)) => a > b,
                (Date(a), Date(b)) => a.days_since_epoch() > b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() > b.to_seconds(),
                _ => return None,
            },

            Gte => match (lhs, rhs) {
                (Integer(a), Integer(b)) => a >= b,
                (Real(a), Real(b)) => a >= b,
                (Text(a), Text(b)) => a >= b,
                (Date(a), Date(b)) => a.days_since_epoch() >= b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() >= b.to_seconds(),
                _ => return None,
            },

            Lt => match (lhs, rhs) {
                (Integer(a), Integer(b)) => a < b,
                (Real(a), Real(b)) => a < b,
                (Text(a), Text(b)) => a < b,
                (Date(a), Date(b)) => a.days_since_epoch() < b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() < b.to_seconds(),
                _ => return None,
            },

            Lte => match (lhs, rhs) {
                (Integer(a), Integer(b)) => a <= b,
                (Real(a), Real(b)) => a <= b,
                (Text(a), Text(b)) => a <= b,
                (Date(a), Date(b)) => a.days_since_epoch() <= b.days_since_epoch(),
                (Time(a), Time(b)) => a.to_seconds() <= b.to_seconds(),
                _ => return None,
            },

            Contains => match (lhs, rhs) {
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

impl ComparisonOp {
    #[inline]
    pub fn eval_usize(&self, lhs: usize, rhs: usize) -> bool {
        match self {
            ComparisonOp::Eq => lhs == rhs,
            ComparisonOp::Neq => lhs != rhs,
            ComparisonOp::Lt => lhs < rhs,
            ComparisonOp::Lte => lhs <= rhs,
            ComparisonOp::Gt => lhs > rhs,
            ComparisonOp::Gte => lhs >= rhs,
            _ => false,
        }
    }
}

fn choose_cast_direction(a: &IglooType, b: &IglooType) -> Option<CastDirection> {
    use CastDirection::*;
    Some(match (a.can_lossless_cast(*b), b.can_lossless_cast(*a)) {
        (true, false) => LTR,
        (false, true) => RTL,
        (true, true) => {
            if a.type_width() > b.type_width() {
                RTL
            } else {
                LTR
            }
        }
        _ => match (a.can_lossy_cast(*b), b.can_lossy_cast(*a)) {
            (true, false) => LTR,
            (false, true) => RTL,
            (true, true) => {
                if a.type_width() > b.type_width() {
                    RTL
                } else {
                    LTR
                }
            }
            _ => return None,
        },
    })
}
