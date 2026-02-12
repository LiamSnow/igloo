//! Persistent generational arena for Device and Group storage
//!
//! Solves the ABA problem by using generational indices.
//! Generation counter is persisted to prevent ID reuse across restarts.
//!
//! Adapted from [generational-arena](https://crates.io/crates/generational-arena)
//!
//! Tries to be mostly panic free, with autofixing behavior.

use igloo_interface::id::GenerationalID;
use serde::{Deserialize, Deserializer, Serialize, ser::SerializeStruct};
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

pub trait ArenaItem<IDMarker> {
    fn id(&self) -> GenerationalID<IDMarker>;
}

pub struct Arena<IDMarker, T> {
    entries: Vec<Entry<T>>,
    generation: u32,
    free_list_head: Option<usize>,
    len: usize,
    _phantom: PhantomData<IDMarker>,
}

impl<IDMarker, T> Arena<IDMarker, T> {
    pub fn new(generation: u32) -> Self {
        Self {
            entries: Vec::new(),
            generation,
            free_list_head: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    /// Creates an arena with pre-allocated slots up to max_index
    /// All slots up to max_index become free entries
    pub fn with_max_index(max_index: usize, generation: u32) -> Self {
        let mut items = Vec::with_capacity(max_index + 1);

        // build free list
        for i in 0..=max_index {
            let next_free = if i < max_index { Some(i + 1) } else { None };
            items.push(Entry::Free { next_free });
        }

        Self {
            entries: items,
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
            match self.entries[index] {
                Entry::Free { next_free } => {
                    self.free_list_head = next_free;
                    self.len += 1;
                    self.entries[index] = Entry::Occupied {
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
        let len = self.entries.len().max(1);
        self.reserve(len);

        // retry with newly allocated space
        let index = self
            .free_list_head
            .expect("free list must exist after reserve");
        match self.entries[index] {
            Entry::Free { next_free } => {
                self.free_list_head = next_free;
                self.len += 1;
                self.entries[index] = Entry::Occupied {
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

        for i in (0..self.entries.len()).rev() {
            if matches!(self.entries[i], Entry::Free { .. }) {
                self.entries[i] = Entry::Free {
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
        if index >= self.entries.len() {
            let start = self.entries.len();
            let old_head = self.free_list_head;

            self.entries.reserve_exact(index - start + 1);
            self.entries.extend((start..=index).map(|i| {
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

        match &self.entries[index] {
            Entry::Free { .. } => {
                self.remove_from_free_list(index);
                self.entries[index] = Entry::Occupied { generation, value };
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
            if let Entry::Free { next_free } = self.entries[target] {
                self.free_list_head = next_free;
            }
            return;
        }

        // search through list
        let mut current = self.free_list_head;
        while let Some(idx) = current {
            if let Entry::Free { next_free } = &self.entries[idx] {
                if *next_free == Some(target) {
                    let target_next = if let Entry::Free { next_free } = self.entries[target] {
                        next_free
                    } else {
                        // broken free listen
                        // TODO probably need to handle this somehow?
                        return;
                    };

                    if let Entry::Free { next_free } = &mut self.entries[idx] {
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

        if index >= self.entries.len() {
            return None;
        }

        match self.entries[index] {
            Entry::Occupied { generation, .. } if generation == id.generation() => {
                let entry = std::mem::replace(
                    &mut self.entries[index],
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

        match self.entries.get(index) {
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

        match self.entries.get_mut(index) {
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
        self.entries.len()
    }

    /// Reserve space for additional elements
    pub fn reserve(&mut self, additional: usize) {
        let start = self.entries.len();
        let end = start + additional;
        let old_head = self.free_list_head;

        self.entries.reserve_exact(additional);
        self.entries.extend((start..end).map(|i| {
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
        self.entries.iter().filter_map(|entry| match entry {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => None,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut().filter_map(|entry| match entry {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => None,
        })
    }

    pub fn items(&self) -> &Vec<Entry<T>> {
        &self.entries
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

impl<IDMarker, T: Serialize> Serialize for Arena<IDMarker, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Arena", 2)?;
        state.serialize_field("generation", &self.generation)?;

        let occupied: Vec<&T> = self.iter().collect();
        state.serialize_field("entry", &occupied)?;

        state.end()
    }
}

impl<'de, IDMarker, T> Deserialize<'de> for Arena<IDMarker, T>
where
    T: Deserialize<'de> + ArenaItem<IDMarker>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ArenaData<T> {
            generation: u32,
            entry: Vec<T>,
        }

        let data: ArenaData<T> = ArenaData::deserialize(deserializer)?;
        let mut arena = Arena::new(data.generation);

        for entry in data.entry {
            let id = entry.id();
            arena.insert_at(id, entry).map_err(de::Error::custom)?;
        }

        Ok(arena)
    }
}
