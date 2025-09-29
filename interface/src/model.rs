use bytes::Bytes;
use smallvec::SmallVec;
use uuid::Uuid;

use crate::Component;

const VERSION: u8 = 0;

/// An ephemeral device identifier. Changes every session
pub type DeviceRef = u16;
/// An ephemeral entity identifier. Changes every session
pub type EntityRef = u16;

pub const FLOE_YEAH_BRO_IM_UP: u8 = 0x80;
pub const FLOE_REGISTER_DEVICE: u8 = 0x81;
pub const FLOE_UPDATES: u8 = 0x82;
pub const FLOE_CUSTOM_COMMAND_ERROR: u8 = 0x83;
pub const FLOE_LOG: u8 = 0x84;

pub const IGLOO_HEY_BUDDY_YOU_AWAKE: u8 = 0x00;
pub const IGLOO_DEVICE_REGISTERED: u8 = 0x01;
pub const IGLOO_REQUEST_UPDATES: u8 = 0x02;
pub const IGLOO_EXECUTE_CUSTOM_COMMAND: u8 = 0x03;

/// MISO Floe sending command -> Igloo
/// Wire format: `[length: u32-le][rest..]`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FloeCommand {
    /// Version handshake response
    /// Wire format: `[0x80][version: u8]`
    YeahBroImUp(u8),

    /// Register a device with Igloo
    /// Wire format: `[0x81][rest..]`
    #[serde(skip)] // FIXME remove
    RegisterDevice(RegisterDevicePayload),

    /// Tell Igloo that components have changed state
    /// Wire format: `[0x82][rest..]`
    Updates(ComponentUpdate),

    /// Custom command execution error
    /// Wire format: `[0x83][cmd_id: u8][msg_len: u16-le][msg: String]`
    CustomCommandError(u8, String),

    /// Log message from device
    /// Wire format: `[0x84][msg_len: u16-le][msg: String]`
    Log(String),
}

/// MOSI Igloo sending command -> Floe
/// Wire format: `[length: u32-le][rest..]`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IglooCommand {
    /// Version handshake request
    /// Wire format: `[0x00][version: u8]`
    HeyBuddyYouAwake(u8),

    /// Device registration acknowledgment
    /// After a successful RegisterDevice, Igloo will give you back
    /// the Device ID and its ephemeral identifier (only for this session)
    /// Wire format: `[0x01][rest..]`
    DeviceRegistered(DeviceRegisteredPayload),

    /// Request component updates from device
    /// This may be invalid, in which case do nothing (or log err).
    /// Updates are NOT confirmed on the device tree until you
    /// acknowledge them by sending back ::Updates
    /// Wire format: `[0x02][rest..]`
    RequestUpdates(ComponentUpdate),

    /// Execute custom command on device
    /// Wire format: `[0x03][cmd_id: u8][payload: bytes]`
    ExecuteCustomCommand(u8, Bytes),
}

/// Component update message
#[derive(Debug, Clone, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
pub struct ComponentUpdate {
    /// Wire format: `[device: u16-le]`
    pub device: DeviceRef,
    /// Wire format: `[entity: u16-le]`
    pub entity: EntityRef,
    /// Wire format: `[value_count: u8][components: [id: u16][data]...]`
    ///  - See individual components for their respective Wire formats
    pub values: SmallVec<[Component; 4]>,
}

/// Device registration payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterDevicePayload {
    /// Persistent device ID
    /// You should register the device under the same UUID every boot
    /// Wire format: `[id: 16 bytes]`
    #[serde(skip)] // FIXME remove
    pub id: Uuid,
    /// Name for the device for first register
    /// Can be modified by the user later on
    /// Wire format: `[len: u16-le][name: String]`
    pub initial_name: String,
    /// The name of every entity you will ever have
    /// Cannot change this after registration!
    /// Wire format: `[count: u16-le][entities: [len: u16-le][name: String]...]`
    pub entity_names: Vec<String>,
}

/// Device registration response payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceRegisteredPayload {
    /// The ID of the device that you registered
    /// Wire format: `[id: 16 bytes]`
    pub id: Uuid,
    /// The ephemeral device identifier (IE only for this session)
    /// Wire format: `[device_ref: u16-le]`
    pub device_ref: DeviceRef,
    /// Entity name (you gave) and the ephemeral entity identifier
    /// Wire format: `[count: u16-le][entities: [name_len: u16-le][name: String][entity_ref: u16-le]...]`
    pub entity_refs: Vec<(String, EntityRef)>,
}
