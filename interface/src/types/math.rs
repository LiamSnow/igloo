use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::types::{IglooColor, IglooInteger, IglooReal, IglooType, IglooValue};

/// Math & logic operations for IglooValues
/// The result type is always the type of the LHS (input value)
/// For cases like Add(Real+Int) the RHS is auto-casted
#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
pub enum MathOp {
    /// Int+Int,
    /// Real+Real, Real+Int,
    /// Color+Color, Color+Real,
    /// Text+Text (concat)
    #[display("+ {_0}")]
    Add(IglooValue),
    /// Int-Int,
    /// Real-Real, Real-Int,
    /// Color-Color, Color-Real
    #[display("- {_0}")]
    Subtract(IglooValue),
    /// Int\*Int,
    /// Real\*Real, Real\*Int,
    /// Color\*Color, Color\*Real,
    /// Text\*Int (repeat)
    #[display("* {_0}")]
    Multiply(IglooValue),
    /// Int/Int,
    /// Real/Real, Real/Int,
    /// Color/Color, Color/Real,
    /// Color/Real
    #[display("/ {_0}")]
    Divide(IglooValue),
    /// Int%Int,
    /// Real%Real, Real%Int,
    #[display("modulo {_0}")]
    Modulo(IglooValue),
    /// Int^Int,
    /// Real^Real, Real^Int,
    #[display("^ {_0}")]
    Power(IglooValue),

    /// Int, Real, Bool (equiv. to ::Not)
    #[display("negate")]
    Negate,
    /// Int, Real
    #[display("absolute")]
    Absolute,
    /// Real
    #[display("floor")]
    Floor,
    /// Real
    #[display("ceiling")]
    Ceiling,
    /// Real
    #[display("round")]
    Round,

    /// Int, Real, Date, Time
    #[display("min")]
    Min(IglooValue),
    /// Int, Real, Date, Time
    #[display("max")]
    Max(IglooValue),

    /// Int, Bool
    #[display("and")]
    And(IglooValue),
    /// Int, Bool
    #[display("or")]
    Or(IglooValue),
    /// Int, Bool
    #[display("xor")]
    Xor(IglooValue),
    /// Bool
    #[display("not")]
    Not,

    /// Integer
    #[display("shift left")]
    ShiftLeft(IglooInteger),
    /// Integer
    #[display("shift right")]
    ShiftRight(IglooInteger),

    /// Date
    #[display("add days")]
    AddDays(IglooInteger),
    /// Date
    #[display("add weeks")]
    AddWeeks(IglooInteger),
    /// Date
    #[display("add months")]
    AddMonths(IglooInteger),
    /// Date
    #[display("add years")]
    AddYears(IglooInteger),

    /// Time
    #[display("add seconds")]
    AddSeconds(IglooInteger),
    /// Time
    #[display("add minutes")]
    AddMinutes(IglooInteger),
    /// Time
    #[display("add hours")]
    AddHours(IglooInteger),

    // Color
    #[display("mix")]
    Mix(IglooColor),
    // Color
    #[display("saturate")]
    Saturate(IglooReal),
    // Color
    #[display("desaturate")]
    Desaturate(IglooReal),
    // Color
    #[display("grayscale")]
    Grayscale,
    // Color
    #[display("hue shift")]
    HueShift(IglooReal),

    // Text
    #[display("to upper")]
    ToUpper,
    // Text
    #[display("to lower")]
    ToLower,
    // Text
    #[display("trim")]
    Trim,
}

impl MathOp {
    pub fn can_eval(&self, lhs: &IglooType) -> bool {
        use IglooType::*;
        use MathOp::*;

        match self {
            Add(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer) | (Real, Real | Integer) | (Color, Color | Real) | (Text, Text)
            ),

            Subtract(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer) | (Real, Real | Integer) | (Color, Color | Real)
            ),

            Multiply(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer)
                    | (Real, Real | Integer)
                    | (Color, Color | Real)
                    | (Text, Integer)
            ),

            Divide(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer) | (Real, Real | Integer) | (Color, Color | Real)
            ),

            Modulo(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer) | (Real, Real | Integer)
            ),

            Power(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer) | (Real, Real | Integer)
            ),

            Min(rhs) | Max(rhs) => matches!(
                (lhs, rhs.r#type()),
                (Integer, Integer) | (Real, Real) | (Date, Date) | (Time, Time)
            ),

            Negate => matches!(lhs, Integer | Real | Boolean),
            Absolute => matches!(lhs, Integer | Real),
            Floor | Ceiling | Round => matches!(lhs, Real),

            And(rhs) | Or(rhs) | Xor(rhs) => {
                matches!((lhs, rhs.r#type()), (Integer, Integer) | (Boolean, Boolean))
            }
            Not => matches!(lhs, Boolean),

            ShiftLeft(_) | ShiftRight(_) => matches!(lhs, Integer),

            AddDays(_) | AddWeeks(_) | AddMonths(_) | AddYears(_) => {
                matches!(lhs, Date)
            }

            AddSeconds(_) | AddMinutes(_) | AddHours(_) => {
                matches!(lhs, Time)
            }

            Mix(_) | Saturate(_) | Desaturate(_) | Grayscale | HueShift(_) => {
                matches!(lhs, Color)
            }

            ToUpper | ToLower | Trim => matches!(lhs, Text),
        }
    }

    pub fn eval(&self, lhs: &IglooValue) -> Option<IglooValue> {
        use IglooValue::*;
        use MathOp::*;

        Some(match (lhs, self) {
            (Integer(a), Add(Integer(b))) => Integer(a.wrapping_add(*b)),
            (Integer(a), Subtract(Integer(b))) => Integer(a.wrapping_sub(*b)),
            (Integer(a), Multiply(Integer(b))) => Integer(a.wrapping_mul(*b)),
            (Integer(a), Divide(Integer(b))) => {
                if *b == 0 {
                    return None;
                }
                Integer(a / b)
            }
            (Integer(a), Modulo(Integer(b))) => {
                if *b == 0 {
                    return None;
                }
                Integer(a % b)
            }
            (Integer(a), Power(Integer(b))) => {
                if *b < 0 {
                    return None;
                }
                Integer(a.checked_pow(*b as u32)?)
            }

            (Real(a), Add(Real(b))) => Real(a + b),
            (Real(a), Add(Integer(b))) => Real(a + *b as f64),
            (Real(a), Subtract(Real(b))) => Real(a - b),
            (Real(a), Subtract(Integer(b))) => Real(a - *b as f64),
            (Real(a), Multiply(Real(b))) => Real(a * b),
            (Real(a), Multiply(Integer(b))) => Real(a * *b as f64),
            (Real(a), Divide(Real(b))) => {
                if *b == 0.0 {
                    return None;
                }
                Real(a / b)
            }
            (Real(a), Divide(Integer(b))) => {
                if *b == 0 {
                    return None;
                }
                Real(a / *b as f64)
            }
            (Real(a), Modulo(Real(b))) => {
                if *b == 0.0 {
                    return None;
                }
                Real(a % b)
            }
            (Real(a), Modulo(Integer(b))) => {
                if *b == 0 {
                    return None;
                }
                Real(a % *b as f64)
            }
            (Real(a), Power(Real(b))) => Real(a.powf(*b)),
            (Real(a), Power(Integer(b))) => Real(a.powi(*b as i32)),

            (Color(a), Add(Color(b))) => Color(*a + *b),
            (Color(a), Add(Real(b))) => {
                let gray = IglooColor::from_rgb(*b, *b, *b);
                Color(*a + gray)
            }
            (Color(a), Subtract(Color(b))) => Color(*a - *b),
            (Color(a), Subtract(Real(b))) => {
                let gray = IglooColor::from_rgb(*b, *b, *b);
                Color(*a - gray)
            }
            (Color(a), Multiply(Color(b))) => Color(*a * *b),
            (Color(a), Multiply(Real(b))) => Color(*a * *b),
            (Color(a), Divide(Color(b))) => Color(*a / *b),
            (Color(a), Divide(Real(b))) => {
                if *b == 0.0 {
                    return None;
                }
                Color(*a / *b)
            }

            (Text(a), Add(Text(b))) => Text(format!("{}{}", a, b)),
            (Text(a), Multiply(Integer(b))) => {
                if *b < 0 {
                    return None;
                }
                Text(a.repeat(*b as usize))
            }

            (Integer(a), Min(Integer(b))) => Integer(*a.min(b)),
            (Integer(a), Max(Integer(b))) => Integer(*a.max(b)),
            (Real(a), Min(Real(b))) => Real(a.min(*b)),
            (Real(a), Max(Real(b))) => Real(a.max(*b)),
            (Date(a), Min(Date(b))) => Date(if a.days_since_epoch() < b.days_since_epoch() {
                *a
            } else {
                *b
            }),
            (Date(a), Max(Date(b))) => Date(if a.days_since_epoch() > b.days_since_epoch() {
                *a
            } else {
                *b
            }),
            (Time(a), Min(Time(b))) => Time(if a.to_seconds() < b.to_seconds() {
                *a
            } else {
                *b
            }),
            (Time(a), Max(Time(b))) => Time(if a.to_seconds() > b.to_seconds() {
                *a
            } else {
                *b
            }),

            (Integer(a), Negate) => Integer(-a),
            (Real(a), Negate) => Real(-a),
            (Boolean(a), Negate) => Boolean(!a),
            (Integer(a), Absolute) => Integer(a.abs()),
            (Real(a), Absolute) => Real(a.abs()),
            (Real(a), Floor) => Real(a.floor()),
            (Real(a), Ceiling) => Real(a.ceil()),
            (Real(a), Round) => Real(a.round()),

            (Integer(a), And(Integer(b))) => Integer(a & b),
            (Integer(a), Or(Integer(b))) => Integer(a | b),
            (Integer(a), Xor(Integer(b))) => Integer(a ^ b),
            (Boolean(a), And(Boolean(b))) => Boolean(*a && *b),
            (Boolean(a), Or(Boolean(b))) => Boolean(*a || *b),
            (Boolean(a), Xor(Boolean(b))) => Boolean(*a ^ *b),
            (Boolean(a), Not) => Boolean(!a),

            (Integer(a), ShiftLeft(n)) => {
                if *n < 0 || *n > 63 {
                    return None;
                }
                Integer(a << n)
            }
            (Integer(a), ShiftRight(n)) => {
                if *n < 0 || *n > 63 {
                    return None;
                }
                Integer(a >> n)
            }

            (Date(a), AddDays(n)) => Date(a.add_days(*n as i32)),
            (Date(a), AddWeeks(n)) => Date(a.add_weeks(*n as i32)),
            (Date(a), AddMonths(n)) => Date(a.add_months(*n as i32)),
            (Date(a), AddYears(n)) => Date(a.add_years(*n as i16)),

            (Time(a), AddSeconds(n)) => Time(a.add_seconds(*n as i32)),
            (Time(a), AddMinutes(n)) => Time(a.add_minutes(*n as i32)),
            (Time(a), AddHours(n)) => Time(a.add_hours(*n as i32)),

            (Color(a), Mix(b)) => Color(a.blend(b, 0.5)),
            (Color(a), Saturate(amount)) => Color(a.saturate(*amount)),
            (Color(a), Desaturate(amount)) => Color(a.desaturate(*amount)),
            (Color(a), Grayscale) => Color(a.grayscale()),
            (Color(a), HueShift(degrees)) => Color(a.hue_shift(*degrees)),

            (Text(a), ToUpper) => Text(a.to_uppercase()),
            (Text(a), ToLower) => Text(a.to_lowercase()),
            (Text(a), Trim) => Text(a.trim().to_string()),

            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_arithmetic() {
        let a = IglooValue::Integer(10);
        assert_eq!(
            MathOp::Add(IglooValue::Integer(5)).eval(&a),
            Some(IglooValue::Integer(15))
        );
        assert_eq!(
            MathOp::Subtract(IglooValue::Integer(3)).eval(&a),
            Some(IglooValue::Integer(7))
        );
        assert_eq!(
            MathOp::Multiply(IglooValue::Integer(4)).eval(&a),
            Some(IglooValue::Integer(40))
        );
        assert_eq!(
            MathOp::Divide(IglooValue::Integer(2)).eval(&a),
            Some(IglooValue::Integer(5))
        );
        assert_eq!(
            MathOp::Modulo(IglooValue::Integer(3)).eval(&a),
            Some(IglooValue::Integer(1))
        );
        assert_eq!(
            MathOp::Power(IglooValue::Integer(2)).eval(&a),
            Some(IglooValue::Integer(100))
        );
    }

    #[test]
    fn test_division_by_zero() {
        let a = IglooValue::Integer(10);
        assert_eq!(MathOp::Divide(IglooValue::Integer(0)).eval(&a), None);
        assert_eq!(MathOp::Modulo(IglooValue::Integer(0)).eval(&a), None);
    }

    #[test]
    fn test_real_arithmetic() {
        let a = IglooValue::Real(10.5);
        assert_eq!(
            MathOp::Add(IglooValue::Real(2.5)).eval(&a),
            Some(IglooValue::Real(13.0))
        );
        assert_eq!(
            MathOp::Add(IglooValue::Integer(5)).eval(&a),
            Some(IglooValue::Real(15.5))
        );
    }

    #[test]
    fn test_text_operations() {
        let text = IglooValue::Text("hello".to_string());
        assert_eq!(
            MathOp::Add(IglooValue::Text(" world".to_string())).eval(&text),
            Some(IglooValue::Text("hello world".to_string()))
        );
        assert_eq!(
            MathOp::Multiply(IglooValue::Integer(3)).eval(&text),
            Some(IglooValue::Text("hellohellohello".to_string()))
        );
        assert_eq!(
            MathOp::ToUpper.eval(&text),
            Some(IglooValue::Text("HELLO".to_string()))
        );
    }

    #[test]
    fn test_color_operations() {
        let color = IglooValue::Color(IglooColor::from_rgb(0.5, 0.5, 0.5));
        assert!(
            MathOp::Multiply(IglooValue::Real(2.0))
                .eval(&color)
                .is_some()
        );
        assert!(MathOp::Grayscale.eval(&color).is_some());
    }

    #[test]
    fn test_unary_operations() {
        assert_eq!(
            MathOp::Negate.eval(&IglooValue::Integer(5)),
            Some(IglooValue::Integer(-5))
        );
        assert_eq!(
            MathOp::Absolute.eval(&IglooValue::Integer(-10)),
            Some(IglooValue::Integer(10))
        );
        assert_eq!(
            MathOp::Floor.eval(&IglooValue::Real(3.7)),
            Some(IglooValue::Real(3.0))
        );
    }

    #[test]
    fn test_can_eval() {
        assert!(MathOp::Add(IglooValue::Integer(5)).can_eval(&IglooType::Integer));
        assert!(MathOp::Add(IglooValue::Integer(5)).can_eval(&IglooType::Real));
        assert!(!MathOp::Add(IglooValue::Integer(5)).can_eval(&IglooType::Text));
        assert!(MathOp::Floor.can_eval(&IglooType::Real));
        assert!(!MathOp::Floor.can_eval(&IglooType::Integer));
    }
}
