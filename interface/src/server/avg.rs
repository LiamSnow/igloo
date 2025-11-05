use std::ops;

use crate::types::*;
use derive_more::{Add, Sub};

pub trait Averageable {
    type Sum: ops::Add<Output = Self::Sum> + ops::Sub<Output = Self::Sum> + Clone + Default;

    /// convert a single item to its sum repr
    fn to_sum_repr(self) -> Self::Sum;

    /// calc sum from multiple items
    fn to_sum(items: Vec<Self>) -> Self::Sum
    where
        Self: Sized,
    {
        items
            .into_iter()
            .map(|item| item.to_sum_repr())
            .fold(Self::Sum::default(), |acc, x| acc + x)
    }

    /// calc avg from parts
    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized;
}

/// An efficient average handler for averaging components
/// Start out with from_vec() then update based on diffs
#[derive(Clone, Debug, Default)]
pub struct Average<T: Averageable> {
    sum: T::Sum,
    count: usize,
}

impl<T: Averageable> Average<T> {
    pub fn new() -> Self {
        Self {
            sum: T::Sum::default(),
            count: 0,
        }
    }

    pub fn from_vec(items: Vec<T>) -> Self {
        let count = items.len();
        let sum = T::to_sum(items);
        Self { sum, count }
    }

    pub fn add(&mut self, item: T) {
        self.sum = self.sum.clone() + item.to_sum_repr();
        self.count += 1;
    }

    pub fn remove(&mut self, item: T) {
        if self.count > 0 {
            self.sum = self.sum.clone() - item.to_sum_repr();
            self.count -= 1;
        }
    }

    pub fn update(&mut self, old_item: T, new_item: T) {
        self.sum = self.sum.clone() - old_item.to_sum_repr() + new_item.to_sum_repr();
    }

    pub fn current_average(&self) -> Option<T> {
        match self.count == 0 {
            true => None,
            false => Some(T::from_sum(self.sum.clone(), self.count)),
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[derive(Clone, Debug, Default, Add, Sub)]
pub struct IglooColorSum {
    pub r: u64,
    pub g: u64,
    pub b: u64,
}

impl Averageable for IglooColor {
    type Sum = IglooColorSum;
    fn to_sum_repr(self) -> Self::Sum {
        IglooColorSum {
            r: self.r as u64,
            g: self.g as u64,
            b: self.b as u64,
        }
    }
    fn from_sum(sum: Self::Sum, len: usize) -> Self {
        IglooColor {
            r: (sum.r / len as u64) as u8,
            g: (sum.g / len as u64) as u8,
            b: (sum.b / len as u64) as u8,
        }
    }
}

impl Averageable for IglooDate {
    type Sum = u64;

    fn to_sum_repr(self) -> Self::Sum {
        // FIXME? approx, maybe we shouldn't implement avgable?
        self.year as u64 * 365 + self.month as u64 * 30 + self.day as u64
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self {
        let avg_days = sum / len as u64;
        let year = (avg_days / 365) as u16;
        let remaining = avg_days % 365;
        let month = (remaining / 30).max(1).min(12) as u8;
        let day = (remaining % 30).max(1).min(31) as u8;

        IglooDate { year, month, day }
    }
}

impl Averageable for IglooTime {
    type Sum = u64;

    fn to_sum_repr(self) -> Self::Sum {
        self.hour as u64 * 3600 + self.minute as u64 * 60 + self.second as u64
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self {
        let avg_seconds = sum / len as u64;
        let hour = ((avg_seconds / 3600) % 24) as u8;
        let minute = ((avg_seconds % 3600) / 60) as u8;
        let second = (avg_seconds % 60) as u8;

        IglooTime {
            hour,
            minute,
            second,
        }
    }
}

impl Averageable for IglooInteger {
    type Sum = Self;

    fn to_sum_repr(self) -> Self::Sum {
        self
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        sum / len as i64
    }
}

impl Averageable for IglooReal {
    type Sum = Self;

    fn to_sum_repr(self) -> Self::Sum {
        self
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        sum / len as f64
    }
}

impl Averageable for IglooBoolean {
    type Sum = u32;

    fn to_sum_repr(self) -> Self::Sum {
        self as u32
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        (sum / len as u32) != 0
    }
}
