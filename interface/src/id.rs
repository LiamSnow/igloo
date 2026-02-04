use bincode::{Decode, Encode};
use derive_more::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;

/// persistent
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[display("Extension(\"{_0}\")")]
#[repr(transparent)]
pub struct ExtensionID(pub String);

/// ephemeral
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[display("Extension(#{_0})")]
#[repr(transparent)]
pub struct ExtensionIndex(pub usize);

/// persistent
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[display("Entity(\"{_0}\")")]
#[repr(transparent)]
pub struct EntityID(pub String);

/// ephemeral
// TODO actually use this
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[display("Entity(#{_0})")]
#[repr(transparent)]
pub struct EntityIndex(pub usize);

// Persistent Packed ID
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
#[display("{}", bs58::encode(packed.to_be_bytes()).into_string())]
#[repr(transparent)]
pub struct GenerationalID<T> {
    packed: u64,
    marker: PhantomData<T>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode,
)]
pub struct DeviceIDMarker;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode,
)]
pub struct GroupIDMarker;

// Persistent Packed ID
pub type DeviceID = GenerationalID<DeviceIDMarker>;

// Persistent Packed ID
pub type GroupID = GenerationalID<GroupIDMarker>;

impl<T> GenerationalID<T> {
    #[inline]
    pub fn new(packed: u64) -> Self {
        Self {
            packed,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn from_parts(index: u32, generation: u32) -> Self {
        let packed = (index as u64) | ((generation as u64) << 32);
        Self::new(packed)
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.packed as u32
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        (self.packed >> 32) as u32
    }

    // Decode from Base58
    pub fn decode_bs58(s: &str) -> bs58::decode::Result<Self> {
        let mut bytes = [0u8; 8];
        bs58::decode(s).onto(&mut bytes)?;
        Ok(Self::new(u64::from_be_bytes(bytes)))
    }

    /// Encode to Base58
    pub fn encode_bs58(&self) -> String {
        bs58::encode(self.packed.to_be_bytes()).into_string()
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

impl<T> Default for GenerationalID<T> {
    fn default() -> Self {
        Self::new(u64::MAX)
    }
}

impl Default for EntityIndex {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl<T> FromStr for GenerationalID<T> {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode_bs58(s)
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for GenerationalID<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            // toml|json -> base58 string
            let encoded = self.encode_bs58();
            serializer.serialize_str(&encoded)
        } else {
            // bincode -> raw u64
            serializer.serialize_u64(self.packed)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for GenerationalID<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            // toml|json -> base58 string
            let s = String::deserialize(deserializer)?;
            Self::decode_bs58(&s).map_err(serde::de::Error::custom)
        } else {
            // bincode -> raw u64
            let value = u64::deserialize(deserializer)?;
            Ok(Self::new(value))
        }
    }
}
