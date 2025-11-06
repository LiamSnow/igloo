use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationOp {
    Mean,
    Median,
    Max,
    Min,
    Sum,
    Any,
    All,
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
                let mut sum: u64 = 0;
                let mut count = 0;
                for item in items {
                    sum += item.year as u64 * 365 + item.month as u64 * 30 + item.day as u64;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                let avg_days = sum / count as u64;
                let year = (avg_days / 365) as u16;
                let remaining = avg_days % 365;
                let month = (remaining / 30).clamp(1, 12) as u8;
                let day = (remaining % 30).clamp(1, 31) as u8;
                Some(IglooDate { year, month, day })
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort_by_key(|d| d.year as u32 * 372 + d.month as u32 * 31 + d.day as u32);
                let mid = collected.len() / 2;
                Some(collected[mid].clone())
            }
            AggregationOp::Max => items
                .into_iter()
                .max_by_key(|d| d.year as u32 * 372 + d.month as u32 * 31 + d.day as u32),
            AggregationOp::Min => items
                .into_iter()
                .min_by_key(|d| d.year as u32 * 372 + d.month as u32 * 31 + d.day as u32),
            _ => None,
        }
    }
}

impl Aggregatable for IglooTime {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum: u64 = 0;
                let mut count = 0;
                for item in items {
                    sum += item.hour as u64 * 3600 + item.minute as u64 * 60 + item.second as u64;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                let avg_seconds = sum / count as u64;
                let hour = ((avg_seconds / 3600) % 24) as u8;
                let minute = ((avg_seconds % 3600) / 60) as u8;
                let second = (avg_seconds % 60) as u8;
                Some(IglooTime {
                    hour,
                    minute,
                    second,
                })
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected
                    .sort_by_key(|t| t.hour as u32 * 3600 + t.minute as u32 * 60 + t.second as u32);
                let mid = collected.len() / 2;
                Some(collected[mid].clone())
            }
            AggregationOp::Max => items
                .into_iter()
                .max_by_key(|t| t.hour as u32 * 3600 + t.minute as u32 * 60 + t.second as u32),
            AggregationOp::Min => items
                .into_iter()
                .min_by_key(|t| t.hour as u32 * 3600 + t.minute as u32 * 60 + t.second as u32),
            _ => None,
        }
    }
}

impl Aggregatable for IglooColor {
    fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
        match op {
            AggregationOp::Mean => {
                let mut sum_r: u64 = 0;
                let mut sum_g: u64 = 0;
                let mut sum_b: u64 = 0;
                let mut count = 0;
                for item in items {
                    sum_r += item.r as u64;
                    sum_g += item.g as u64;
                    sum_b += item.b as u64;
                    count += 1;
                }
                if count == 0 {
                    return None;
                }
                Some(IglooColor {
                    r: (sum_r / count) as u8,
                    g: (sum_g / count) as u8,
                    b: (sum_b / count) as u8,
                })
            }
            AggregationOp::Median => {
                let mut collected: Vec<Self> = items.into_iter().collect();
                if collected.is_empty() {
                    return None;
                }
                collected.sort_by_key(|c| (c.r, c.g, c.b));
                let mid = collected.len() / 2;
                Some(collected[mid].clone())
            }
            AggregationOp::Max => items.into_iter().max_by_key(|c| (c.r, c.g, c.b)),
            AggregationOp::Min => items.into_iter().min_by_key(|c| (c.r, c.g, c.b)),
            _ => None,
        }
    }
}
