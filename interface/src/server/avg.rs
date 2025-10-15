use std::ops::{Add, Sub};

pub trait Averageable {
    type Sum: Add<Output = Self::Sum> + Sub<Output = Self::Sum> + Clone + Default;

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
