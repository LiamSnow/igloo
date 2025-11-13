use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

/// persistent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Floe(\"{_0}\")")]
pub struct FloeID(pub String);

/// ephemeral
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Floe(#{_0})")]
pub struct FloeRef(pub usize);

/// persistent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Device({}:{})", self.idx(), self.generation())]
pub struct DeviceID(u64);

/// persistent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Group({}:{})", self.idx(), self.generation())]
pub struct GroupID(u64);

impl GroupID {
    #[inline]
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        GroupID(packed)
    }

    // TODO rename to index
    #[inline]
    pub fn idx(&self) -> u32 {
        self.0 as u32
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }
}

impl DeviceID {
    #[inline]
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        DeviceID(packed)
    }

    #[inline]
    pub fn from_comb(c: u64) -> Self {
        DeviceID(c)
    }

    // TODO rename to index
    #[inline]
    pub fn idx(&self) -> u32 {
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
                "Expected DeviceID format 'idx:generation', got '{}'",
                s
            ));
        }

        let idx = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid DeviceID idx: {}", e))?;
        let generation = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid DeviceID generation: {}", e))?;

        Ok(DeviceID::from_parts(idx, generation))
    }
}

impl FromStr for GroupID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Expected GroupID format 'idx:generation', got '{}'",
                s
            ));
        }

        let idx = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid GroupID idx: {}", e))?;
        let generation = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid GroupID generation: {}", e))?;

        Ok(GroupID::from_parts(idx, generation))
    }
}
