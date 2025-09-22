// This file is transpiled into components.rs via build.rs
// Which generates necessary methods, add #[derive(..)] and pub prefix
// Note: Make sure to make a fields public!!

/// signed 32-bit integer
struct Int(pub i32);

/// unsigned 32-bit integer
struct Uint(pub u32);

/// signed 64-bit integer
struct Long(pub i64);

/// 64-bit floating point
struct Float(pub f64);

struct Bool(pub bool);

struct Text(pub String);

// TODO should we have this?
struct Object(pub std::collections::HashMap<String, Component>);

// TODO should we have this?
struct List(pub Vec<Component>);

struct IntList(pub Vec<i32>);

struct UintList(pub Vec<u32>);
struct LongList(pub Vec<i64>);
struct FloatList(pub Vec<f64>);
struct TextList(pub Vec<String>);

struct Date(pub jiff::civil::Date);
struct Time(pub jiff::civil::Time);
struct DateTime(pub jiff::civil::DateTime);
// Weekday(Weekday),
struct Duration(pub jiff::SignedDuration);

struct Switch(pub bool);

struct Dimmer(pub u8);

struct Color {
    // IMPLEMENT AVERAGEABLE
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

enum Unit {
    Seconds,

    Minutes,
    Hours,
    Days,

    Celsius,
    Fahrenheit,
    Kelvin,

    Meters,
    Centimeters,
    Feet,
    Inches,

    Percent,

    Watts,
    Kilowatts,

    Pascal,
    Bar,
    Psi,

    Liters,
    Gallons,

    Decibels,

    Custom(String),
}

/// usually used in combination with a Float to set a range
struct FloatMin(pub f64);
/// usually used in combination with a Float to set a range
struct FloatMax(pub f64);
/// usually used in combination with a Float to set a range
struct FloatStep(pub f64);

/// usually used in combination with an Int to set a range
struct IntMin(pub i32);
/// usually used in combination with an Int to set a range
struct IntMax(pub i32);
/// usually used in combination with an Int to set a range
struct IntStep(pub i32);

/// usually used in combination with an Long to set a range
struct LongMin(pub i64);
/// usually used in combination with an Long to set a range
struct LongMax(pub i64);
/// usually used in combination with an Long to set a range
struct LongStep(pub i64);

/// usually used in combination with an Uint to set a range
struct UintMin(pub u32);
/// usually used in combination with an Uint to set a range
struct UintMax(pub u32);
/// usually used in combination with an Uint to set a range
struct UintStep(pub u32);

/// usually used in combination with a Switch and sometimes a Dimmer, Color, and/or ColorTemperature
struct Light;

/// usually used in combination with a Switch and sometimes a Dimmer, Color, and/or ColorTemperature
struct LightBulb;

/// this is just a marker meant to be comined with a Uint
/// and optional UintMin and UintMax
/// for clarity you can also add Unit::Kelvin
struct ColorTemperature;

/// used in combination with Text to set a max length
struct TextMaxLength(pub usize);
/// used in combination with Text to set a min length
struct TextMinLength(pub usize);
/// used in combination with Text to require a regex expr
struct TextPattern(pub String);

/// A marker for this to be a text selection used with:
///   Text: the currently selected option
///   TextList: the options available
struct TextSelect;

enum FanOscillation {
    Off,
    Vertical,
    Horizontal,
    Both,
}

enum FanDirection {
    Forward,
    Reverse,
}

enum FanSpeed {
    Auto,
    Percent(f64),
}

enum ClimateMode {
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

enum LockState {
    Unknown,
    Locked,
    Unlocked,
    Jammed,
    Locking,
    Unlocking,
    Custom(String),
}

enum MediaState {
    Unknown,
    Idle,
    Playing,
    Paused,
    Custom(String),
}

enum CoverState {
    Opening,
    Closing,
    Stopped,
    Open,
    Closed,
    Custom(String),
}

enum BinarySensorType {
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

enum AlarmState {
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
