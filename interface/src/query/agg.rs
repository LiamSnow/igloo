use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::compound::*;
use crate::types::*;

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

impl IglooType {
    pub fn is_aggregatable(&self) -> bool {
        use IglooType::*;
        matches!(
            self,
            Integer | Real | Boolean | Date | Time | Color | Enum(_)
        )
    }
}

pub trait Aggregatable {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self>
    where
        Self: Sized;
}

impl Aggregatable for IglooInteger {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum: Self = 0;
                let mut count = 0;
                for item in items {
                    sum += item;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                Some(sum / count as Self)
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort();
                let mid = collected.len() / 2;
                if collected.len() % 2 == 0 {
                    Some((collected[mid - 1] + collected[mid]) / 2)
                } else {
                    Some(collected[mid])
                }
            }
            AggregationOp::Max => items.into_iter().max(),
            AggregationOp::Min => items.into_iter().min(),
            AggregationOp::Sum => {
                let mut sum: Self = 0;
                for item in items {
                    sum += item;
                }
                Some(sum)
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooReal {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum: f64 = 0.0;
                let mut count = 0;
                for item in items {
                    sum += item;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                Some(sum / count as f64)
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = collected.len() / 2;
                if collected.len() % 2 == 0 {
                    Some((collected[mid - 1] + collected[mid]) / 2.0)
                } else {
                    Some(collected[mid])
                }
            }
            AggregationOp::Max => items
                .into_iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            AggregationOp::Min => items
                .into_iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            AggregationOp::Sum => {
                let mut sum: f64 = 0.0;
                for item in items {
                    sum += item;
                }
                Some(sum)
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooBoolean {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut true_count = 0;
                let mut total_count = 0;
                for item in items {
                    if item {
                        true_count += 1;
                    }
                    total_count += 1;
                }
                if total_count == 0 {
                    return None;
                }
                Some(true_count * 2 >= total_count)
            }
            AggregationOp::Any => {
                for item in items {
                    if item {
                        return Some(true);
                    }
                }
                None
            }
            AggregationOp::All => {
                let mut seen_any = false;
                for item in items {
                    seen_any = true;
                    if !item {
                        return Some(false);
                    }
                }
                if seen_any { Some(true) } else { None }
            }
            _ => None,
        }
    }
}

impl Aggregatable for IglooDate {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum: i64 = 0;
                let mut count = 0;
                for item in items {
                    sum += item.days_since_epoch() as i64;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                let avg_days = (sum / count) as i32;
                Some(IglooDate::from_days_since_epoch(avg_days))
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort_by_key(|d| d.days_since_epoch());
                let mid = collected.len() / 2;
                Some(collected[mid].clone())
            }
            AggregationOp::Max => items.into_iter().max_by_key(|d| d.days_since_epoch()),
            AggregationOp::Min => items.into_iter().min_by_key(|d| d.days_since_epoch()),
            _ => None,
        }
    }
}

impl Aggregatable for IglooTime {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum: i64 = 0;
                let mut count = 0;
                for item in items {
                    sum += item.to_seconds() as i64;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                let avg_seconds = (sum / count) as i32;
                Some(IglooTime::from_seconds(avg_seconds))
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort_by_key(|t| t.to_seconds());
                let mid = collected.len() / 2;
                Some(collected[mid].clone())
            }
            AggregationOp::Max => items.into_iter().max_by_key(|t| t.to_seconds()),
            AggregationOp::Min => items.into_iter().min_by_key(|t| t.to_seconds()),
            _ => None,
        }
    }
}

impl Aggregatable for IglooColor {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum = IglooColor::default();
                let mut count = 0;
                for item in items {
                    sum = sum + item;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                Some(sum / count as f64)
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort_by(|a, b| {
                    (a.r, a.g, a.b)
                        .partial_cmp(&(b.r, b.g, b.b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let mid = collected.len() / 2;
                Some(collected[mid].clone())
            }
            AggregationOp::Max => items.into_iter().max_by(|a, b| {
                (a.r, a.g, a.b)
                    .partial_cmp(&(b.r, b.g, b.b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            AggregationOp::Min => items.into_iter().min_by(|a, b| {
                (a.r, a.g, a.b)
                    .partial_cmp(&(b.r, b.g, b.b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => None,
        }
    }
}
