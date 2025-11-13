use crate::types::{IglooType, IglooValue};

impl IglooType {
    pub fn can_cast(self, to: Self) -> bool {
        self.can_lossless_cast(to) || self.can_lossy_cast(to)
    }

    pub fn can_lossless_cast(self, to: Self) -> bool {
        use IglooType::*;
        matches!(
            (self, to),
            (_, Text)
                | (Integer, Real)
                | (Boolean, Integer)
                | (Boolean, Real)

                //.
                | (IntegerList, RealList)
                | (BooleanList, IntegerList)
                | (BooleanList, RealList)
                | (IntegerList, TextList)
                | (RealList, TextList)
                | (BooleanList, TextList)
                | (ColorList, TextList)
                | (DateList, TextList)
                | (TimeList, TextList)
        )
    }

    pub fn can_lossy_cast(self, to: Self) -> bool {
        use IglooType::*;
        matches!(
            (self, to),
            (Real, Integer)
                | (Integer, Boolean)
                | (Real, Boolean)
                | (RealList, IntegerList)
                | (IntegerList, BooleanList)
                | (RealList, BooleanList)
        )
    }

    pub fn cast_node_name(self, to: Self) -> Option<String> {
        if !self.can_cast(to) {
            return None;
        }
        Some(format!("Cast {self} to {to}"))
    }
}

impl IglooValue {
    pub fn cast(self, to: IglooType) -> Option<IglooValue> {
        let r#type = self.r#type();
        if r#type.can_lossless_cast(to) {
            self.lossless_cast(to)
        } else if r#type.can_lossy_cast(to) {
            self.lossy_cast(to)
        } else {
            None
        }
    }

    pub fn lossless_cast(self, to: IglooType) -> Option<IglooValue> {
        use IglooType as T;
        use IglooValue::*;

        match (self, to) {
            (v, T::Text) => Some(Text(v.to_string())),

            (Integer(v), T::Real) => Some(Real(v as f64)),

            (Boolean(v), T::Integer) => Some(Integer(if v { 1 } else { 0 })),

            (Boolean(v), T::Real) => Some(Real(if v { 1.0 } else { 0.0 })),

            (IntegerList(list), T::RealList) => {
                Some(RealList(list.into_iter().map(|v| v as f64).collect()))
            }

            (BooleanList(list), T::IntegerList) => Some(IntegerList(
                list.into_iter().map(|v| if v { 1 } else { 0 }).collect(),
            )),

            (BooleanList(list), T::RealList) => Some(RealList(
                list.into_iter()
                    .map(|v| if v { 1.0 } else { 0.0 })
                    .collect(),
            )),

            (IntegerList(list), T::TextList) => {
                Some(TextList(list.into_iter().map(|v| v.to_string()).collect()))
            }

            (RealList(list), T::TextList) => {
                Some(TextList(list.into_iter().map(|v| v.to_string()).collect()))
            }

            (BooleanList(list), T::TextList) => {
                Some(TextList(list.into_iter().map(|v| v.to_string()).collect()))
            }

            (ColorList(list), T::TextList) => {
                Some(TextList(list.into_iter().map(|v| v.to_string()).collect()))
            }

            (DateList(list), T::TextList) => {
                Some(TextList(list.into_iter().map(|v| v.to_string()).collect()))
            }

            (TimeList(list), T::TextList) => {
                Some(TextList(list.into_iter().map(|v| v.to_string()).collect()))
            }

            _ => None,
        }
    }

    pub fn lossy_cast(self, to: IglooType) -> Option<IglooValue> {
        use IglooType as T;
        use IglooValue::*;

        match (self, to) {
            (Real(v), T::Integer) => Some(Integer(v as i64)),

            (Integer(v), T::Boolean) => Some(Boolean(v != 0)),

            (Real(v), T::Boolean) => Some(Boolean(v != 0.0)),

            (RealList(list), T::IntegerList) => {
                Some(IntegerList(list.into_iter().map(|v| v as i64).collect()))
            }

            (IntegerList(list), T::BooleanList) => {
                Some(BooleanList(list.into_iter().map(|v| v != 0).collect()))
            }

            (RealList(list), T::BooleanList) => {
                Some(BooleanList(list.into_iter().map(|v| v != 0.0).collect()))
            }

            _ => None,
        }
    }
}
