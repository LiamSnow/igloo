//! Persistent generational arena for Device and Group storage
//!
//! Solves the ABA problem by using generational indices.
//! Generation counter is persisted to prevent ID reuse across restarts.
//!
//! Adapted from [generational-arena](https://crates.io/crates/generational-arena)
//!
//! Tries to be mostly panic free, with autofixing behavior.

use igloo_interface::id::GenerationalID;
use std::marker::PhantomData;

#[derive(thiserror::Error, Debug)]
#[error("Cannot insert `{tried}` because `{there}` has that slot!")]
pub struct SlotOccupied<IDMarker> {
    pub tried: GenerationalID<IDMarker>,
    pub there: GenerationalID<IDMarker>,
}

#[derive(Debug)]
pub enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied { generation: u32, value: T },
}

pub struct Arena<IDMarker, T> {
    items: Vec<Entry<T>>,
    generation: u32,
    free_list_head: Option<usize>,
    len: usize,
    _phantom: PhantomData<IDMarker>,
}

impl<IDMarker, T> Arena<IDMarker, T> {
    /// Creates an arena with pre-allocated slots up to max_index
    /// All slots up to max_index become free entries
    pub fn new(max_index: usize, generation: u32) -> Self {
        let mut items = Vec::with_capacity(max_index + 1);

        // build free list
        for i in 0..=max_index {
            let next_free = if i < max_index { Some(i + 1) } else { None };
            items.push(Entry::Free { next_free });
        }

        Self {
            items,
            generation,
            free_list_head: if max_index > 0 { Some(0) } else { None },
            len: 0,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Insert a value, allocating a new slot if needed
    pub fn insert(&mut self, value: T) -> GenerationalID<IDMarker> {
        // try to use free list first
        if let Some(index) = self.free_list_head {
            match self.items[index] {
                Entry::Free { next_free } => {
                    self.free_list_head = next_free;
                    self.len += 1;
                    self.items[index] = Entry::Occupied {
                        generation: self.generation,
                        value,
                    };
                    return GenerationalID::from_parts(index as u32, self.generation);
                }
                Entry::Occupied { .. } => {
                    eprintln!(
                        "Corrupt free list detected at index {}, rebuilding...",
                        index
                    );
                    self.rebuild_free_list();
                    return self.insert(value);
                }
            }
        }

        // no free slots -> allocate more
        let len = self.items.len().max(1);
        self.reserve(len);

        // retry with newly allocated space
        let index = self
            .free_list_head
            .expect("free list must exist after reserve");
        match self.items[index] {
            Entry::Free { next_free } => {
                self.free_list_head = next_free;
                self.len += 1;
                self.items[index] = Entry::Occupied {
                    generation: self.generation,
                    value,
                };
                GenerationalID::from_parts(index as u32, self.generation)
            }
            // really should not happen
            Entry::Occupied { .. } => unreachable!("corrupt free list after reserve"),
        }
    }

    fn rebuild_free_list(&mut self) {
        let mut new_head = None;

        for i in (0..self.items.len()).rev() {
            if matches!(self.items[i], Entry::Free { .. }) {
                self.items[i] = Entry::Free {
                    next_free: new_head,
                };
                new_head = Some(i);
            }
        }

        self.free_list_head = new_head;
    }

    /// Insert at a specific index with a specific generation
    /// Used during load to restore persisted state
    /// Will automatically expand the arena if needed
    pub fn insert_at(
        &mut self,
        id: GenerationalID<IDMarker>,
        value: T,
    ) -> Result<(), SlotOccupied<IDMarker>> {
        let index = id.index() as usize;
        let generation = id.generation();

        // auto-expand
        if index >= self.items.len() {
            let start = self.items.len();
            let old_head = self.free_list_head;

            self.items.reserve_exact(index - start + 1);
            self.items.extend((start..=index).map(|i| {
                if i == index {
                    Entry::Free {
                        next_free: old_head,
                    }
                } else {
                    Entry::Free {
                        next_free: Some(i + 1),
                    }
                }
            }));

            self.free_list_head = Some(start);
        }

        match &self.items[index] {
            Entry::Free { .. } => {
                self.remove_from_free_list(index);
                self.items[index] = Entry::Occupied { generation, value };
                self.len += 1;
                Ok(())
            }
            Entry::Occupied { generation, .. } => Err(SlotOccupied {
                tried: id,
                there: GenerationalID::from_parts(index as u32, *generation),
            }),
        }
    }

    fn remove_from_free_list(&mut self, target: usize) {
        if self.free_list_head == Some(target) {
            // head of list
            if let Entry::Free { next_free } = self.items[target] {
                self.free_list_head = next_free;
            }
            return;
        }

        // search through list
        let mut current = self.free_list_head;
        while let Some(idx) = current {
            if let Entry::Free { next_free } = &self.items[idx] {
                if *next_free == Some(target) {
                    let target_next = if let Entry::Free { next_free } = self.items[target] {
                        next_free
                    } else {
                        // broken free listen
                        // TODO probably need to handle this somehow?
                        return;
                    };

                    if let Entry::Free { next_free } = &mut self.items[idx] {
                        *next_free = target_next;
                    }
                    return;
                }
                current = *next_free;
            }
        }
    }

    /// Remove an element by ID
    /// Returns Some(value) if removed, None if not found or stale
    pub fn remove(&mut self, id: GenerationalID<IDMarker>) -> Option<T> {
        let index = id.index() as usize;

        if index >= self.items.len() {
            return None;
        }

        match self.items[index] {
            Entry::Occupied { generation, .. } if generation == id.generation() => {
                let entry = std::mem::replace(
                    &mut self.items[index],
                    Entry::Free {
                        next_free: self.free_list_head,
                    },
                );

                self.generation += 1;
                self.free_list_head = Some(index);
                self.len -= 1;

                match entry {
                    Entry::Occupied { value, .. } => Some(value),
                    _ => unreachable!(),
                }
            }
            _ => None,
        }
    }

    /// Get a reference to an element
    /// Returns None if not found or stale
    #[inline]
    pub fn get(&self, id: &GenerationalID<IDMarker>) -> Option<&T> {
        let index = id.index() as usize;

        match self.items.get(index) {
            Some(Entry::Occupied { generation, value }) if *generation == id.generation() => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Get a mutable reference to an element
    /// Returns None if not found or stale
    #[inline]
    pub fn get_mut(&mut self, id: &GenerationalID<IDMarker>) -> Option<&mut T> {
        let index = id.index() as usize;

        match self.items.get_mut(index) {
            Some(Entry::Occupied { generation, value }) if *generation == id.generation() => {
                Some(value)
            }
            _ => None,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn contains(&self, id: &GenerationalID<IDMarker>) -> bool {
        self.get(id).is_some()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[allow(dead_code)]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.items.len()
    }

    /// Reserve space for additional elements
    pub fn reserve(&mut self, additional: usize) {
        let start = self.items.len();
        let end = start + additional;
        let old_head = self.free_list_head;

        self.items.reserve_exact(additional);
        self.items.extend((start..end).map(|i| {
            if i == end - 1 {
                Entry::Free {
                    next_free: old_head,
                }
            } else {
                Entry::Free {
                    next_free: Some(i + 1),
                }
            }
        }));

        self.free_list_head = Some(start);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().filter_map(|entry| match entry {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => None,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().filter_map(|entry| match entry {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => None,
        })
    }

    pub fn items(&self) -> &Vec<Entry<T>> {
        &self.items
    }
}

impl<T> Entry<T> {
    #[allow(dead_code)]
    pub fn is_occupied(&self) -> bool {
        matches!(self, Entry::Occupied { .. })
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => None,
        }
    }

    #[allow(dead_code)]
    pub fn generation(&self) -> Option<u32> {
        match self {
            Entry::Occupied { generation, .. } => Some(*generation),
            Entry::Free { .. } => None,
        }
    }
}
