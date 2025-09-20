use crate::traits::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    #[serde(flatten)]
    pub switchable: Switchable,
    #[serde(flatten)]
    pub colorable: Colorable,
    #[serde(flatten)]
    pub dimmable: Dimmable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fan {
    #[serde(flatten)]
    pub switchable: Switchable,
    pub speed: Option<FanSpeed>,
    pub direction: Option<FanDirection>,
    pub oscillation: Option<FanOscillation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FanOscillation {
    Off,
    Vertical,
    Horizontal,
    Both,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FanDirection {
    Forward,
    Reverse,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FanSpeed {
    Auto,
    Percent(f64),
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Switchable {
    pub on: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dimmable {
    pub brightness: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Colorable {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Temperature {
    pub temperature: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClimateMode {
    Off,
    Auto,
    Heat,
    Cool,
    HeatCool,
    FanOnly,
    Dry,
    Eco,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thermostat {
    pub current_temperature: f64,
    pub target_temperature: f64,
    pub climate_mode: ClimateMode,
    pub target_temperature_high: Option<f64>,
    pub target_temperature_low: Option<f64>,
    pub current_humidity: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lock {
    pub lock_state: LockState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LockState {
    Unknown,
    Locked,
    Unlocked,
    Jammed,
    Locking,
    Unlocking,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaPlayer {
    pub media_state: MediaState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MediaState {
    Unknown,
    Idle,
    Playing,
    Paused,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cover {
    pub position: f64,
    pub state: Option<CoverState>,
    pub tilt: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CoverState {
    Opening,
    Closing,
    Stopped,
    Open,
    Closed,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinarySensor {
    pub state: bool,
    pub sensor_type: Option<SensorType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SensorType {
    Motion,
    Door,
    Window,
    Smoke,
    Gas,
    Moisture,
    Occupancy,
    Light,
    Sound,
    Vibration,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Button {
    pub action: Option<ButtonAction>,
    pub pressed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ButtonAction {
    Single,
    Double,
    Long,
    Release,
    Pressed,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Number {
    pub value: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    #[serde(flatten)]
    pub unitable: Option<Unitable>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alarm {
    pub arm_state: ArmState,
    pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ArmState {
    Disarmed,
    ArmedHome,
    ArmedAway,
    ArmedNight,
    ArmedVacation,
    ArmedCustom,
    Pending,
    Triggered,
    Arming,
    Disarming,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    pub streaming: bool,
    pub recording: Option<bool>,
    pub motion_detected: Option<bool>,
    pub snapshot_url: Option<String>,
    pub stream_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vacuum {
    pub state: VacuumState,
    pub battery_level: Option<f64>,
    pub fan_speed_percent: Option<f64>,
    pub fan_speed_mode: Option<VacuumFanSpeed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VacuumState {
    Docked,
    Idle,
    Cleaning,
    Returning,
    Paused,
    Error,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VacuumFanSpeed {
    Auto,
    Silent,
    Standard,
    Medium,
    High,
    Max,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Select {
    pub selected: String,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Text {
    pub text: String,
    pub max_length: Option<i32>,
    pub pattern: Option<String>,
}
