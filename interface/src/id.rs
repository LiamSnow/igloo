use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

/// persistent
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, BorshSerialize, BorshDeserialize,
)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Floe(\"{_0}\")")]
pub struct FloeID(pub String);

/// ephemeral
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Floe(#{_0})")]
pub struct FloeRef(pub usize);

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
    BorshSerialize,
    BorshDeserialize,
)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Device({}:{})", self.index(), self.generation())]
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
    BorshSerialize,
    BorshDeserialize,
)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Group({}:{})", self.index(), self.generation())]
pub struct GroupID(u64);

impl GroupID {
    #[inline]
    pub fn from_parts(index: u32, generation: u32) -> Self {
        let packed = (index as u64) | ((generation as u64) << 32);
        GroupID(packed)
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.0 as u32
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }
}

impl DeviceID {
    #[inline]
    pub fn from_parts(index: u32, generation: u32) -> Self {
        let packed = (index as u64) | ((generation as u64) << 32);
        DeviceID(packed)
    }

    #[inline]
    pub fn from_comb(c: u64) -> Self {
        DeviceID(c)
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.0 as u32
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn take(self) -> u64 {
        self.0
    }
}

impl Default for FloeID {
    fn default() -> Self {
        Self("undefined".to_string())
    }
}

impl Default for FloeRef {
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
