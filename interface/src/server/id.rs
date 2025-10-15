use std::fmt::Display;

use borsh::{BorshDeserialize, BorshSerialize};

/// persistent
#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct FloeID(pub String);

/// ephemeral
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloeRef(pub usize);

/// persistent
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, BorshSerialize, BorshDeserialize)]
pub struct DeviceID(u64);

/// persistent
#[derive(Debug, PartialEq, Eq, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct GroupID(u64);

impl GroupID {
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        GroupID(packed)
    }

    pub fn idx(&self) -> u32 {
        self.0 as u32
    }

    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }
}

impl Display for GroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.idx(), self.generation())
    }
}

impl DeviceID {
    pub fn from_parts(idx: u32, generation: u32) -> Self {
        let packed = (idx as u64) | ((generation as u64) << 32);
        DeviceID(packed)
    }

    pub fn from_comb(c: u64) -> Self {
        DeviceID(c)
    }

    pub fn idx(&self) -> u32 {
        self.0 as u32
    }

    pub fn generation(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn take(self) -> u64 {
        self.0
    }
}

impl Display for DeviceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.idx(), self.generation())
    }
}
