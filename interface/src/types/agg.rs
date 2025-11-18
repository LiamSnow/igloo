use super::*;
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, BorshSerialize, BorshDeserialize)]
pub enum AggregationOp {
    #[display("mean")]
    Mean,
    #[display("median")]
    Median,
    #[display("max")]
    Max,
    #[display("min")]
    Min,
    #[display("sum")]
    Sum,
    #[display("any")]
    Any,
    #[display("all")]
    All,
}

pub const AGGREGATION_OPS: [AggregationOp; 7] = [
    AggregationOp::Mean,
    AggregationOp::Median,
    AggregationOp::Max,
    AggregationOp::Min,
    AggregationOp::Sum,
    AggregationOp::Any,
    AggregationOp::All,
];

impl AggregationOp {
    pub fn can_apply(&self, r#type: &IglooType) -> bool {
        use AggregationOp::*;
        use IglooType::*;
        match r#type {
            Integer | Real => matches!(self, Mean | Median | Max | Min | Sum),
            Boolean => matches!(self, Mean | Any | All),
            Date | Time => matches!(self, Mean | Median | Max | Min),
            Color => matches!(self, Mean | Median | Max | Min),
            Enum(_) => matches!(self, Mean),
            _ => false,
        }
    }
}

pub trait Aggregatable {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue>;
}

impl Aggregatable for IglooValue {
    fn aggregate(items: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match items.get(0) {
            Some(IglooValue::Integer(_)) => IglooInteger::aggregate(items, op),
            Some(IglooValue::Real(_)) => IglooReal::aggregate(items, op),
            Some(IglooValue::Boolean(_)) => IglooBoolean::aggregate(items, op),
            Some(IglooValue::Color(_)) => IglooColor::aggregate(items, op),
            Some(IglooValue::Date(_)) => IglooDate::aggregate(items, op),
            Some(IglooValue::Time(_)) => IglooTime::aggregate(items, op),
            Some(IglooValue::Enum(_)) => IglooEnumValue::aggregate(items, op),
            _ => None,
        }
    }
}

impl Aggregatable for IglooInteger {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match op {
            AggregationOp::Mean => {
                let mut sum: Self = 0;
                let mut count = 0;
                for val in iter {
                    if let IglooValue::Integer(v) = val {
                        sum += v;
                        count += 1;
                    }
                }
                if count == 0 {
                    return None;
                }
                Some(IglooValue::Integer(sum / count as Self))
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = Vec::new();
                for v in iter {
                    if let IglooValue::Integer(i) = v {
                        collected.push(i);
                    }
                }
                if collected.is_empty() {
                    return None;
                }
                collected.sort_unstable();
                let mid = collected.len() / 2;
                let result = if collected.len() % 2 == 0 {
                    (collected[mid - 1] + collected[mid]) / 2
                } else {
                    collected[mid]
                };
                Some(IglooValue::Integer(result))
            }
            AggregationOp::Max => {
                let mut max_val: Option<Self> = None;
                for val in iter {
                    if let IglooValue::Integer(i) = val {
                        max_val = Some(match max_val {
                            None => i,
                            Some(m) => {
                                if i > m {
                                    i
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                max_val.map(IglooValue::Integer)
            }
            AggregationOp::Min => {
                let mut min_val: Option<Self> = None;
                for val in iter {
                    if let IglooValue::Integer(i) = val {
                        min_val = Some(match min_val {
                            None => i,
                            Some(m) => {
                                if i < m {
                                    i
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                min_val.map(IglooValue::Integer)
            }
            AggregationOp::Sum => {
                let mut sum: Self = 0;
                for val in iter {
                    if let IglooValue::Integer(v) = val {
                        sum += v;
                    }
                }
                Some(IglooValue::Integer(sum))
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooReal {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match op {
            AggregationOp::Mean => {
                let mut sum: f64 = 0.0;
                let mut count = 0;
                for val in iter {
                    if let IglooValue::Real(v) = val {
                        sum += v;
                        count += 1;
                    }
                }
                if count == 0 {
                    return None;
                }
                Some(IglooValue::Real(sum / count as f64))
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = Vec::new();
                for v in iter {
                    if let IglooValue::Real(r) = v {
                        collected.push(r);
                    }
                }
                if collected.is_empty() {
                    return None;
                }
                collected
                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = collected.len() / 2;
                let result = if collected.len() % 2 == 0 {
                    (collected[mid - 1] + collected[mid]) / 2.0
                } else {
                    collected[mid]
                };
                Some(IglooValue::Real(result))
            }
            AggregationOp::Max => {
                let mut max_val: Option<Self> = None;
                for val in iter {
                    if let IglooValue::Real(r) = val {
                        max_val = Some(match max_val {
                            None => r,
                            Some(m) => {
                                match r.partial_cmp(&m).unwrap_or(std::cmp::Ordering::Equal) {
                                    std::cmp::Ordering::Greater => r,
                                    _ => m,
                                }
                            }
                        });
                    }
                }
                max_val.map(IglooValue::Real)
            }
            AggregationOp::Min => {
                let mut min_val: Option<Self> = None;
                for val in iter {
                    if let IglooValue::Real(r) = val {
                        min_val = Some(match min_val {
                            None => r,
                            Some(m) => {
                                match r.partial_cmp(&m).unwrap_or(std::cmp::Ordering::Equal) {
                                    std::cmp::Ordering::Less => r,
                                    _ => m,
                                }
                            }
                        });
                    }
                }
                min_val.map(IglooValue::Real)
            }
            AggregationOp::Sum => {
                let mut sum: f64 = 0.0;
                for val in iter {
                    if let IglooValue::Real(v) = val {
                        sum += v;
                    }
                }
                Some(IglooValue::Real(sum))
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooBoolean {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match op {
            AggregationOp::Mean => {
                let mut true_count = 0;
                let mut total_count = 0;
                for val in iter {
                    if let IglooValue::Boolean(v) = val {
                        if v {
                            true_count += 1;
                        }
                        total_count += 1;
                    }
                }
                if total_count == 0 {
                    return None;
                }
                Some(IglooValue::Boolean(true_count * 2 >= total_count))
            }
            AggregationOp::Any => {
                for val in iter {
                    if let IglooValue::Boolean(v) = val {
                        if v {
                            return Some(IglooValue::Boolean(true));
                        }
                    }
                }
                None
            }
            AggregationOp::All => {
                let mut seen_any = false;
                for val in iter {
                    if let IglooValue::Boolean(v) = val {
                        seen_any = true;
                        if !v {
                            return Some(IglooValue::Boolean(false));
                        }
                    }
                }
                if seen_any {
                    Some(IglooValue::Boolean(true))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooDate {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match op {
            AggregationOp::Mean => {
                let mut sum: i64 = 0;
                let mut count = 0;
                for val in iter {
                    if let IglooValue::Date(d) = val {
                        sum += d.days_since_epoch() as i64;
                        count += 1;
                    }
                }
                if count == 0 {
                    return None;
                }
                let avg_days = (sum / count) as i32;
                Some(IglooValue::Date(IglooDate::from_days_since_epoch(avg_days)))
            }
            AggregationOp::Median => {
                let mut collected: Vec<IglooDate> = Vec::new();
                for v in iter {
                    if let IglooValue::Date(d) = v {
                        collected.push(d);
                    }
                }
                if collected.is_empty() {
                    return None;
                }
                collected.sort_unstable_by_key(|d| d.days_since_epoch());
                let mid = collected.len() / 2;
                Some(IglooValue::Date(collected.swap_remove(mid)))
            }
            AggregationOp::Max => {
                let mut max_val: Option<IglooDate> = None;
                for val in iter {
                    if let IglooValue::Date(d) = val {
                        max_val = Some(match max_val {
                            None => d,
                            Some(m) => {
                                if d.days_since_epoch() > m.days_since_epoch() {
                                    d
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                max_val.map(IglooValue::Date)
            }
            AggregationOp::Min => {
                let mut min_val: Option<IglooDate> = None;
                for val in iter {
                    if let IglooValue::Date(d) = val {
                        min_val = Some(match min_val {
                            None => d,
                            Some(m) => {
                                if d.days_since_epoch() < m.days_since_epoch() {
                                    d
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                min_val.map(IglooValue::Date)
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooTime {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match op {
            AggregationOp::Mean => {
                let mut sum: i64 = 0;
                let mut count = 0;
                for val in iter {
                    if let IglooValue::Time(t) = val {
                        sum += t.to_seconds() as i64;
                        count += 1;
                    }
                }
                if count == 0 {
                    return None;
                }
                let avg_seconds = (sum / count) as i32;
                Some(IglooValue::Time(IglooTime::from_seconds(avg_seconds)))
            }
            AggregationOp::Median => {
                let mut collected: Vec<IglooTime> = Vec::new();
                for v in iter {
                    if let IglooValue::Time(t) = v {
                        collected.push(t);
                    }
                }
                if collected.is_empty() {
                    return None;
                }
                collected.sort_unstable_by_key(|t| t.to_seconds());
                let mid = collected.len() / 2;
                Some(IglooValue::Time(collected.swap_remove(mid)))
            }
            AggregationOp::Max => {
                let mut max_val: Option<IglooTime> = None;
                for val in iter {
                    if let IglooValue::Time(t) = val {
                        max_val = Some(match max_val {
                            None => t,
                            Some(m) => {
                                if t.to_seconds() > m.to_seconds() {
                                    t
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                max_val.map(IglooValue::Time)
            }
            AggregationOp::Min => {
                let mut min_val: Option<IglooTime> = None;
                for val in iter {
                    if let IglooValue::Time(t) = val {
                        min_val = Some(match min_val {
                            None => t,
                            Some(m) => {
                                if t.to_seconds() < m.to_seconds() {
                                    t
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                min_val.map(IglooValue::Time)
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooColor {
    fn aggregate(iter: Vec<IglooValue>, op: AggregationOp) -> Option<IglooValue> {
        match op {
            AggregationOp::Mean => {
                let mut sum = IglooColor::default();
                let mut count = 0;
                for val in iter {
                    if let IglooValue::Color(c) = val {
                        sum = sum + c;
                        count += 1;
                    }
                }
                if count == 0 {
                    return None;
                }
                Some(IglooValue::Color(sum / count as f64))
            }
            AggregationOp::Median => {
                let mut collected: Vec<IglooColor> = Vec::new();
                for v in iter {
                    if let IglooValue::Color(c) = v {
                        collected.push(c);
                    }
                }
                if collected.is_empty() {
                    return None;
                }
                collected.sort_unstable_by(|a, b| {
                    (a.r, a.g, a.b)
                        .partial_cmp(&(b.r, b.g, b.b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let mid = collected.len() / 2;
                Some(IglooValue::Color(collected.swap_remove(mid)))
            }
            AggregationOp::Max => {
                let mut max_val: Option<IglooColor> = None;
                for val in iter {
                    if let IglooValue::Color(c) = val {
                        max_val = Some(match max_val {
                            None => c,
                            Some(m) => {
                                let cmp = (c.r, c.g, c.b)
                                    .partial_cmp(&(m.r, m.g, m.b))
                                    .unwrap_or(std::cmp::Ordering::Equal);
                                if cmp == std::cmp::Ordering::Greater {
                                    c
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                max_val.map(IglooValue::Color)
            }
            AggregationOp::Min => {
                let mut min_val: Option<IglooColor> = None;
                for val in iter {
                    if let IglooValue::Color(c) = val {
                        min_val = Some(match min_val {
                            None => c,
                            Some(m) => {
                                let cmp = (c.r, c.g, c.b)
                                    .partial_cmp(&(m.r, m.g, m.b))
                                    .unwrap_or(std::cmp::Ordering::Equal);
                                if cmp == std::cmp::Ordering::Less {
                                    c
                                } else {
                                    m
                                }
                            }
                        });
                    }
                }
                min_val.map(IglooValue::Color)
            }
            _ => None,
        }
    }
}
