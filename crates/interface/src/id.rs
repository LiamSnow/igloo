use std::fmt::Display;

use crate::generated::shared::igloo::lib::id::{
    DeviceId, EntityId, EntityIndex, ExtensionId, ExtensionIndex, GroupId,
};

pub const MAX_ENTITY_ID_LENGTH: usize = 100;

impl GenerationalId for DeviceId {
    #[inline]
    fn from_raw(packed: u64) -> Self {
        Self { packed }
    }

    #[inline]
    fn inner(&self) -> u64 {
        self.packed
    }
}

impl GenerationalId for GroupId {
    #[inline]
    fn from_raw(packed: u64) -> Self {
        Self { packed }
    }

    #[inline]
    fn inner(&self) -> u64 {
        self.packed
    }
}

pub trait GenerationalId: Sized {
    const HIGH_BIT: u64 = 1 << 63;
    const INDEX_BITS: u64 = 31;
    const INDEX_MASK: u64 = (1 << Self::INDEX_BITS) - 1;

    fn from_raw(packed: u64) -> Self;
    fn inner(&self) -> u64;

    #[inline]
    fn new(packed: u64) -> Self {
        let me = Self::from_raw(packed);
        debug_assert!(me.valid(), "invalid ID");
        me
    }

    #[inline]
    fn from_parts(index: u32, generation: u32) -> Self {
        debug_assert!(index as u64 <= Self::INDEX_MASK, "index exceeds 31 bits");
        let packed = Self::HIGH_BIT
            | ((generation as u64) << Self::INDEX_BITS)
            | (index as u64 & Self::INDEX_MASK);
        Self::from_raw(packed)
    }

    #[inline]
    fn index(&self) -> u32 {
        debug_assert!(self.valid(), "invalid ID");
        (self.inner() & Self::INDEX_MASK) as u32
    }

    #[inline]
    fn generation(&self) -> u32 {
        debug_assert!(self.valid(), "invalid ID");
        ((self.inner() >> Self::INDEX_BITS) & 0xFFFF_FFFF) as u32
    }

    fn valid(&self) -> bool {
        self.inner() & Self::HIGH_BIT == 1
    }
}

impl Default for ExtensionId {
    fn default() -> Self {
        Self {
            inner: "undefined".to_string(),
        }
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self {
            inner: "undefined".to_string(),
        }
    }
}

impl Default for ExtensionIndex {
    fn default() -> Self {
        Self { inner: u64::MAX }
    }
}

impl Default for EntityIndex {
    fn default() -> Self {
        Self { inner: u64::MAX }
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

impl Display for ExtensionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Extension(\"{}\")", self.inner)
    }
}

impl Display for ExtensionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Extension(&{})", self.inner)
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity(\"{}\")", self.inner)
    }
}

impl Display for EntityIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity(&{})", self.inner)
    }
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Device({})", self.packed)
    }
}

impl Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Group({})", self.packed)
    }
}
