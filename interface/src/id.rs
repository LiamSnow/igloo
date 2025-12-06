use bincode::{Decode, Encode};
use derive_more::Display;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

/// persistent
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Floe(\"{_0}\")")]
#[repr(transparent)]
pub struct ExtensionID(pub String);

/// ephemeral
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Floe(#{_0})")]
#[repr(transparent)]
pub struct ExtensionIndex(pub usize);

/// persistent
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Entity(\"{_0}\")")]
#[repr(transparent)]
pub struct EntityID(pub String);

/// ephemeral
// TODO actually use this
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Entity(#{_0})")]
#[repr(transparent)]
pub struct EntityIndex(pub usize);

/// persistent
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    bincode::Encode,
    bincode::Decode,
)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Device({}:{})", self.index(), self.generation())]
#[repr(transparent)]
pub struct DeviceID(u64);

/// persistent
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    bincode::Encode,
    bincode::Decode,
)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Group({}:{})", self.index(), self.generation())]
#[repr(transparent)]
pub struct GroupID(u64);

pub trait GenerationalID: Copy + Eq + Hash + fmt::Display + fmt::Debug {
    fn from_parts(index: u32, generation: u32) -> Self;
    fn from_comb(c: u64) -> Self;
    fn index(&self) -> u32;
    fn generation(&self) -> u32;
    fn take(self) -> u64;
}

impl GenerationalID for GroupID {
    #[inline]
    fn from_parts(index: u32, generation: u32) -> Self {
        let packed = (index as u64) | ((generation as u64) << 32);
        GroupID(packed)
    }

    #[inline]
    fn from_comb(c: u64) -> Self {
        GroupID(c)
    }

    #[inline]
    fn index(&self) -> u32 {
        self.0 as u32
    }

    #[inline]
    fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[inline]
    fn take(self) -> u64 {
        self.0
    }
}

impl GenerationalID for DeviceID {
    #[inline]
    fn from_parts(index: u32, generation: u32) -> Self {
        let packed = (index as u64) | ((generation as u64) << 32);
        DeviceID(packed)
    }

    #[inline]
    fn from_comb(c: u64) -> Self {
        DeviceID(c)
    }

    #[inline]
    fn index(&self) -> u32 {
        self.0 as u32
    }

    #[inline]
    fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[inline]
    fn take(self) -> u64 {
        self.0
    }
}

impl Default for ExtensionID {
    fn default() -> Self {
        Self("undefined".to_string())
    }
}

impl Default for EntityID {
    fn default() -> Self {
        Self("undefined".to_string())
    }
}

impl Default for ExtensionIndex {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl Default for DeviceID {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

impl Default for GroupID {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

impl Default for EntityIndex {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl FromStr for DeviceID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Expected DeviceID format 'index:generation', got '{}'",
                s
            ));
        }

        let index = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid DeviceID index: {}", e))?;
        let generation = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid DeviceID generation: {}", e))?;

        Ok(DeviceID::from_parts(index, generation))
    }
}

impl FromStr for GroupID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Expected GroupID format 'index:generation', got '{}'",
                s
            ));
        }

        let index = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid GroupID index: {}", e))?;
        let generation = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid GroupID generation: {}", e))?;

        Ok(GroupID::from_parts(index, generation))
    }
}
