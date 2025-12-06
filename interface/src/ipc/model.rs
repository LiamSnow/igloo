use crate::Component;
use bincode::{Decode, Encode};
use rustc_hash::FxHashMap;

pub const MSIM: u8 = 5;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[repr(u8)]
pub enum IglooMessage {
    // WARN: Append-only. Never change order or it will break backwards-compatibility
    //.
    //.
    /// Floe/Script must send to Igloo on boot
    /// Igloo will then never send >MSIC or >MSIM
    WhatsUpIgloo {
        // max supported igloo component
        msic: u16,
        // max supported igloo message
        msim: u8,
    } = 0,

    /// Name
    CreateDevice(String) = 1,
    /// Name, DeviceID
    DeviceCreated(String, u64) = 2,

    RegisterEntity {
        device: u64,
        entity_id: String,
        entity_index: usize,
    } = 3,

    WriteComponents {
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    } = 4,

    /// Custom command from Igloo -> Client
    /// As specified in Igloo.toml
    Custom {
        name: String,
        params: FxHashMap<String, String>,
    } = 5,
}
