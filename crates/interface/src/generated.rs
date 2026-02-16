use crate::types::*;
use crate::types::agg::AggregationOp;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
/// Total number of Components in Igloo
/// (length of `Components` enum)
pub const NUM_COMPONENTS: usize = 48usize;
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum ComponentType {
    Integer,
    Real,
    Text,
    Boolean,
    Color,
    Date,
    Time,
    IntegerList,
    RealList,
    TextList,
    BooleanList,
    ColorList,
    DateList,
    TimeList,
    Trigger,
    Timestamp,
    Duration,
    Weekday,
    Light,
    Switch,
    Dimmer,
    ColorMode,
    ColorTemperature,
    Volume,
    Muted,
    Config,
    Diagnostic,
    TextSelect,
    Siren,
    Sensor,
    Icon,
    AccuracyDecimals,
    DeviceClass,
    SensorStateClass,
    Unit,
    FanOscillation,
    FanDirection,
    FanSpeed,
    ClimateMode,
    LockState,
    MediaState,
    Cover,
    CoverState,
    Position,
    Tilt,
    Valve,
    ValveState,
    AlarmState,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IglooEnumType {
    Weekday,
    ColorMode,
    SensorStateClass,
    Unit,
    FanOscillation,
    FanDirection,
    FanSpeed,
    ClimateMode,
    LockState,
    MediaState,
    CoverState,
    ValveState,
    AlarmState,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IglooEnumValue {
    Weekday(Weekday),
    ColorMode(ColorMode),
    SensorStateClass(SensorStateClass),
    Unit(Unit),
    FanOscillation(FanOscillation),
    FanDirection(FanDirection),
    FanSpeed(FanSpeed),
    ClimateMode(ClimateMode),
    LockState(LockState),
    MediaState(MediaState),
    CoverState(CoverState),
    ValveState(ValveState),
    AlarmState(AlarmState),
}
impl IglooEnumValue {
    pub fn from_string(enum_type: &IglooEnumType, s: String) -> Option<Self> {
        match enum_type {
            IglooEnumType::Weekday => {
                Weekday::try_from(s).ok().map(IglooEnumValue::Weekday)
            }
            IglooEnumType::ColorMode => {
                ColorMode::try_from(s).ok().map(IglooEnumValue::ColorMode)
            }
            IglooEnumType::SensorStateClass => {
                SensorStateClass::try_from(s).ok().map(IglooEnumValue::SensorStateClass)
            }
            IglooEnumType::Unit => Unit::try_from(s).ok().map(IglooEnumValue::Unit),
            IglooEnumType::FanOscillation => {
                FanOscillation::try_from(s).ok().map(IglooEnumValue::FanOscillation)
            }
            IglooEnumType::FanDirection => {
                FanDirection::try_from(s).ok().map(IglooEnumValue::FanDirection)
            }
            IglooEnumType::FanSpeed => {
                FanSpeed::try_from(s).ok().map(IglooEnumValue::FanSpeed)
            }
            IglooEnumType::ClimateMode => {
                ClimateMode::try_from(s).ok().map(IglooEnumValue::ClimateMode)
            }
            IglooEnumType::LockState => {
                LockState::try_from(s).ok().map(IglooEnumValue::LockState)
            }
            IglooEnumType::MediaState => {
                MediaState::try_from(s).ok().map(IglooEnumValue::MediaState)
            }
            IglooEnumType::CoverState => {
                CoverState::try_from(s).ok().map(IglooEnumValue::CoverState)
            }
            IglooEnumType::ValveState => {
                ValveState::try_from(s).ok().map(IglooEnumValue::ValveState)
            }
            IglooEnumType::AlarmState => {
                AlarmState::try_from(s).ok().map(IglooEnumValue::AlarmState)
            }
        }
    }
    pub fn default(enum_type: &IglooEnumType) -> Self {
        match enum_type {
            IglooEnumType::Weekday => IglooEnumValue::Weekday(Weekday::Sunday),
            IglooEnumType::ColorMode => IglooEnumValue::ColorMode(ColorMode::RGB),
            IglooEnumType::SensorStateClass => {
                IglooEnumValue::SensorStateClass(SensorStateClass::Measurement)
            }
            IglooEnumType::Unit => IglooEnumValue::Unit(Unit::VoltAmperes),
            IglooEnumType::FanOscillation => {
                IglooEnumValue::FanOscillation(FanOscillation::Off)
            }
            IglooEnumType::FanDirection => {
                IglooEnumValue::FanDirection(FanDirection::Forward)
            }
            IglooEnumType::FanSpeed => IglooEnumValue::FanSpeed(FanSpeed::On),
            IglooEnumType::ClimateMode => IglooEnumValue::ClimateMode(ClimateMode::Off),
            IglooEnumType::LockState => IglooEnumValue::LockState(LockState::Unknown),
            IglooEnumType::MediaState => IglooEnumValue::MediaState(MediaState::Unknown),
            IglooEnumType::CoverState => IglooEnumValue::CoverState(CoverState::Idle),
            IglooEnumType::ValveState => IglooEnumValue::ValveState(ValveState::Idle),
            IglooEnumType::AlarmState => IglooEnumValue::AlarmState(AlarmState::Disarmed),
        }
    }
    pub fn get_type(&self) -> IglooEnumType {
        match self {
            IglooEnumValue::Weekday(_) => IglooEnumType::Weekday,
            IglooEnumValue::ColorMode(_) => IglooEnumType::ColorMode,
            IglooEnumValue::SensorStateClass(_) => IglooEnumType::SensorStateClass,
            IglooEnumValue::Unit(_) => IglooEnumType::Unit,
            IglooEnumValue::FanOscillation(_) => IglooEnumType::FanOscillation,
            IglooEnumValue::FanDirection(_) => IglooEnumType::FanDirection,
            IglooEnumValue::FanSpeed(_) => IglooEnumType::FanSpeed,
            IglooEnumValue::ClimateMode(_) => IglooEnumType::ClimateMode,
            IglooEnumValue::LockState(_) => IglooEnumType::LockState,
            IglooEnumValue::MediaState(_) => IglooEnumType::MediaState,
            IglooEnumValue::CoverState(_) => IglooEnumType::CoverState,
            IglooEnumValue::ValveState(_) => IglooEnumType::ValveState,
            IglooEnumValue::AlarmState(_) => IglooEnumType::AlarmState,
        }
    }
}
impl std::fmt::Display for IglooEnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IglooEnumValue::Weekday(v) => write!(f, "{}", v),
            IglooEnumValue::ColorMode(v) => write!(f, "{}", v),
            IglooEnumValue::SensorStateClass(v) => write!(f, "{}", v),
            IglooEnumValue::Unit(v) => write!(f, "{}", v),
            IglooEnumValue::FanOscillation(v) => write!(f, "{}", v),
            IglooEnumValue::FanDirection(v) => write!(f, "{}", v),
            IglooEnumValue::FanSpeed(v) => write!(f, "{}", v),
            IglooEnumValue::ClimateMode(v) => write!(f, "{}", v),
            IglooEnumValue::LockState(v) => write!(f, "{}", v),
            IglooEnumValue::MediaState(v) => write!(f, "{}", v),
            IglooEnumValue::CoverState(v) => write!(f, "{}", v),
            IglooEnumValue::ValveState(v) => write!(f, "{}", v),
            IglooEnumValue::AlarmState(v) => write!(f, "{}", v),
        }
    }
}
pub static IGLOO_ENUMS: [IglooEnumType; 13usize] = [
    IglooEnumType::Weekday,
    IglooEnumType::ColorMode,
    IglooEnumType::SensorStateClass,
    IglooEnumType::Unit,
    IglooEnumType::FanOscillation,
    IglooEnumType::FanDirection,
    IglooEnumType::FanSpeed,
    IglooEnumType::ClimateMode,
    IglooEnumType::LockState,
    IglooEnumType::MediaState,
    IglooEnumType::CoverState,
    IglooEnumType::ValveState,
    IglooEnumType::AlarmState,
];
impl ComponentType {
    pub fn igloo_type(&self) -> Option<IglooType> {
        match self {
            ComponentType::Integer => Some(IglooType::Integer),
            ComponentType::Real => Some(IglooType::Real),
            ComponentType::Text => Some(IglooType::Text),
            ComponentType::Boolean => Some(IglooType::Boolean),
            ComponentType::Color => Some(IglooType::Color),
            ComponentType::Date => Some(IglooType::Date),
            ComponentType::Time => Some(IglooType::Time),
            ComponentType::IntegerList => Some(IglooType::IntegerList),
            ComponentType::RealList => Some(IglooType::RealList),
            ComponentType::TextList => Some(IglooType::TextList),
            ComponentType::BooleanList => Some(IglooType::BooleanList),
            ComponentType::ColorList => Some(IglooType::ColorList),
            ComponentType::DateList => Some(IglooType::DateList),
            ComponentType::TimeList => Some(IglooType::TimeList),
            ComponentType::Trigger => None,
            ComponentType::Timestamp => Some(IglooType::Integer),
            ComponentType::Duration => Some(IglooType::Integer),
            ComponentType::Weekday => Some(IglooType::Enum(IglooEnumType::Weekday)),
            ComponentType::Light => None,
            ComponentType::Switch => Some(IglooType::Boolean),
            ComponentType::Dimmer => Some(IglooType::Real),
            ComponentType::ColorMode => Some(IglooType::Enum(IglooEnumType::ColorMode)),
            ComponentType::ColorTemperature => Some(IglooType::Integer),
            ComponentType::Volume => Some(IglooType::Real),
            ComponentType::Muted => Some(IglooType::Boolean),
            ComponentType::Config => None,
            ComponentType::Diagnostic => None,
            ComponentType::TextSelect => None,
            ComponentType::Siren => None,
            ComponentType::Sensor => None,
            ComponentType::Icon => Some(IglooType::Text),
            ComponentType::AccuracyDecimals => Some(IglooType::Integer),
            ComponentType::DeviceClass => Some(IglooType::Text),
            ComponentType::SensorStateClass => {
                Some(IglooType::Enum(IglooEnumType::SensorStateClass))
            }
            ComponentType::Unit => Some(IglooType::Enum(IglooEnumType::Unit)),
            ComponentType::FanOscillation => {
                Some(IglooType::Enum(IglooEnumType::FanOscillation))
            }
            ComponentType::FanDirection => {
                Some(IglooType::Enum(IglooEnumType::FanDirection))
            }
            ComponentType::FanSpeed => Some(IglooType::Enum(IglooEnumType::FanSpeed)),
            ComponentType::ClimateMode => {
                Some(IglooType::Enum(IglooEnumType::ClimateMode))
            }
            ComponentType::LockState => Some(IglooType::Enum(IglooEnumType::LockState)),
            ComponentType::MediaState => Some(IglooType::Enum(IglooEnumType::MediaState)),
            ComponentType::Cover => None,
            ComponentType::CoverState => Some(IglooType::Enum(IglooEnumType::CoverState)),
            ComponentType::Position => Some(IglooType::Real),
            ComponentType::Tilt => Some(IglooType::Real),
            ComponentType::Valve => None,
            ComponentType::ValveState => Some(IglooType::Enum(IglooEnumType::ValveState)),
            ComponentType::AlarmState => Some(IglooType::Enum(IglooEnumType::AlarmState)),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}
impl TryFrom<String> for Weekday {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Sunday" | "sunday" | "sun" => Weekday::Sunday,
                "Monday" | "monday" | "mon" | "m" => Weekday::Monday,
                "Tuesday" | "tuesday" | "tue" | "tues" | "t" => Weekday::Tuesday,
                "Wednesday" | "wednesday" | "wed" | "w" => Weekday::Wednesday,
                "Thursday" | "thursday" | "thu" | "r" => Weekday::Thursday,
                "Friday" | "friday" | "fri" | "f" => Weekday::Friday,
                "Saturday" | "saturday" | "sat" => Weekday::Saturday,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for Weekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Weekday::Sunday => write!(f, "{}", "Sunday"),
            Weekday::Monday => write!(f, "{}", "Monday"),
            Weekday::Tuesday => write!(f, "{}", "Tuesday"),
            Weekday::Wednesday => write!(f, "{}", "Wednesday"),
            Weekday::Thursday => write!(f, "{}", "Thursday"),
            Weekday::Friday => write!(f, "{}", "Friday"),
            Weekday::Saturday => write!(f, "{}", "Saturday"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ColorMode {
    RGB,
    Temperature,
}
impl TryFrom<String> for ColorMode {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "RGB" => ColorMode::RGB,
                "Temperature" => ColorMode::Temperature,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::RGB => write!(f, "{}", "RGB"),
            ColorMode::Temperature => write!(f, "{}", "Temperature"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SensorStateClass {
    Measurement,
    TotalIncreasing,
    Total,
}
impl TryFrom<String> for SensorStateClass {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Measurement" => SensorStateClass::Measurement,
                "TotalIncreasing" => SensorStateClass::TotalIncreasing,
                "Total" => SensorStateClass::Total,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for SensorStateClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorStateClass::Measurement => write!(f, "{}", "Measurement"),
            SensorStateClass::TotalIncreasing => write!(f, "{}", "TotalIncreasing"),
            SensorStateClass::Total => write!(f, "{}", "Total"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Unit {
    VoltAmperes,
    Watts,
    KiloWatts,
    BtusPerHour,
    VoltAmpereReactive,
    WattHours,
    KiloWattHours,
    MegaWattHours,
    Milliamperes,
    Amperes,
    Millivolts,
    Volts,
    Degrees,
    Euros,
    Dollars,
    Cents,
    Celsius,
    Fahrenheit,
    Kelvin,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
    Millimeters,
    Centimeters,
    Meters,
    Kilometers,
    Inches,
    Feet,
    Yard,
    Miles,
    Hertz,
    Kilohertz,
    Megahertz,
    Gigahertz,
    Pascal,
    Hectopascal,
    Kilopascal,
    Bar,
    Centibar,
    Millibar,
    MillimeterMercury,
    InchMercury,
    Psi,
    Decibel,
    DecibelAWeighted,
    DecibelsMilliwatt,
    Liters,
    Milliliters,
    CubicMeters,
    CubicFeet,
    Gallons,
    FluidOunce,
    CubicMetersPerHour,
    CubicFeetPerMinute,
    SquareMeters,
    Grams,
    Kilograms,
    Milligrams,
    Micrograms,
    Ounces,
    Pounds,
    MicrosiemensPerCentimeter,
    Lux,
    UvIndex,
    Percentage,
    WattsPerSquareMeter,
    BtusPerHourSquareFoot,
    MicrogramsPerCubicMeter,
    MilligramsPerCubicMeter,
    MicrogramsPerCubicFoot,
    PartsPerCubicMeter,
    PartsPerMillion,
    PartsPerBillion,
    MillimetersPerDay,
    MillimetersPerHour,
    FeetPerSecond,
    InchesPerDay,
    MetersPerSecond,
    InchesPerHour,
    KilometersPerHour,
    Knots,
    MilesPerHour,
    Bits,
    Kilobits,
    Megabits,
    Gigabits,
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Petabytes,
    Exabytes,
    Zettabytes,
    Yottabytes,
    Kibibytes,
    Mebibytes,
    Gibibytes,
    Tebibytes,
    Pebibytes,
    Exbibytes,
    Zebibytes,
    Yobibytes,
    BitsPerSecond,
    KilobitsPerSecond,
    MegabitsPerSecond,
    GigabitsPerSecond,
    BytesPerSecond,
    KilobytesPerSecond,
    MegabytesPerSecond,
    GigabytesPerSecond,
    KibibytesPerSecond,
    MebibytesPerSecond,
    GibibytesPerSecond,
}
impl TryFrom<String> for Unit {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "VoltAmperes" | "VA" => Unit::VoltAmperes,
                "Watts" | "W" | "watt" | "watts" => Unit::Watts,
                "KiloWatts" | "kW" | "kw" | "kilowatt" | "kilowatts" => Unit::KiloWatts,
                "BtusPerHour" | "BTU/h" => Unit::BtusPerHour,
                "VoltAmpereReactive" | "var" => Unit::VoltAmpereReactive,
                "WattHours" | "Wh" => Unit::WattHours,
                "KiloWattHours" | "kWh" => Unit::KiloWattHours,
                "MegaWattHours" | "MWh" => Unit::MegaWattHours,
                "Milliamperes" | "mA" | "milliampere" | "milliamperes" | "milliamps"
                | "milliamp" => Unit::Milliamperes,
                "Amperes" | "A" | "amps" | "amp" | "ampere" | "amperes" => Unit::Amperes,
                "Millivolts" | "mV" | "millivolts" => Unit::Millivolts,
                "Volts" | "V" | "volt" | "volts" => Unit::Volts,
                "Degrees" | "°" | "degree" | "degrees" => Unit::Degrees,
                "Euros" | "€" | "euro" | "euros" => Unit::Euros,
                "Dollars" | "$" | "dollar" | "dollars" => Unit::Dollars,
                "Cents" | "¢" | "cent" | "cents" => Unit::Cents,
                "Celsius" | "°C" | "celcius" => Unit::Celsius,
                "Fahrenheit" | "°F" | "fahrenheit" => Unit::Fahrenheit,
                "Kelvin" | "K" | "kelvin" => Unit::Kelvin,
                "Microseconds" | "μs" | "us" | "microseconds" => Unit::Microseconds,
                "Milliseconds" | "ms" | "milliseconds" => Unit::Milliseconds,
                "Seconds" | "s" | "seconds" => Unit::Seconds,
                "Minutes" | "min" | "mins" | "minute" | "minutes" => Unit::Minutes,
                "Hours" | "h" | "hour" | "hours" => Unit::Hours,
                "Days" | "d" | "day" | "days" => Unit::Days,
                "Weeks" | "w" | "wk" | "wks" | "week" | "weeks" => Unit::Weeks,
                "Months" | "month" | "months" => Unit::Months,
                "Years" | "y" | "year" | "years" => Unit::Years,
                "Millimeters" | "mm" => Unit::Millimeters,
                "Centimeters" | "cm" => Unit::Centimeters,
                "Meters" | "m" => Unit::Meters,
                "Kilometers" | "km" => Unit::Kilometers,
                "Inches" | "in" => Unit::Inches,
                "Feet" | "ft" => Unit::Feet,
                "Yard" | "yd" => Unit::Yard,
                "Miles" | "mi" => Unit::Miles,
                "Hertz" | "Hz" => Unit::Hertz,
                "Kilohertz" | "kHz" => Unit::Kilohertz,
                "Megahertz" | "MHz" => Unit::Megahertz,
                "Gigahertz" | "GHz" => Unit::Gigahertz,
                "Pascal" | "Pa" => Unit::Pascal,
                "Hectopascal" | "hPa" => Unit::Hectopascal,
                "Kilopascal" | "kPa" => Unit::Kilopascal,
                "Bar" | "bar" => Unit::Bar,
                "Centibar" | "cbar" => Unit::Centibar,
                "Millibar" | "mbar" => Unit::Millibar,
                "MillimeterMercury" | "mmHg" => Unit::MillimeterMercury,
                "InchMercury" | "inHg" => Unit::InchMercury,
                "Psi" | "psi" => Unit::Psi,
                "Decibel" | "dB" | "decibel" | "decibels" => Unit::Decibel,
                "DecibelAWeighted" | "dBa" => Unit::DecibelAWeighted,
                "DecibelsMilliwatt" | "dBm" => Unit::DecibelsMilliwatt,
                "Liters" | "L" => Unit::Liters,
                "Milliliters" | "mL" => Unit::Milliliters,
                "CubicMeters" | "m³" => Unit::CubicMeters,
                "CubicFeet" | "ft³" => Unit::CubicFeet,
                "Gallons" | "gal" => Unit::Gallons,
                "FluidOunce" | "fl. oz." => Unit::FluidOunce,
                "CubicMetersPerHour" | "m³/h" => Unit::CubicMetersPerHour,
                "CubicFeetPerMinute" | "ft³/m" => Unit::CubicFeetPerMinute,
                "SquareMeters" | "m²" => Unit::SquareMeters,
                "Grams" | "g" => Unit::Grams,
                "Kilograms" | "kg" => Unit::Kilograms,
                "Milligrams" | "mg" => Unit::Milligrams,
                "Micrograms" | "µg" => Unit::Micrograms,
                "Ounces" | "oz" => Unit::Ounces,
                "Pounds" | "lb" => Unit::Pounds,
                "MicrosiemensPerCentimeter" | "µS/cm" => Unit::MicrosiemensPerCentimeter,
                "Lux" | "lx" => Unit::Lux,
                "UvIndex" | "UV index" => Unit::UvIndex,
                "Percentage" | "%" => Unit::Percentage,
                "WattsPerSquareMeter" | "W/m²" => Unit::WattsPerSquareMeter,
                "BtusPerHourSquareFoot" | "BTU/(h×ft²)" => Unit::BtusPerHourSquareFoot,
                "MicrogramsPerCubicMeter" | "µg/m³" => Unit::MicrogramsPerCubicMeter,
                "MilligramsPerCubicMeter" | "mg/m³" => Unit::MilligramsPerCubicMeter,
                "MicrogramsPerCubicFoot" | "μg/ft³" => Unit::MicrogramsPerCubicFoot,
                "PartsPerCubicMeter" | "p/m³" => Unit::PartsPerCubicMeter,
                "PartsPerMillion" | "ppm" => Unit::PartsPerMillion,
                "PartsPerBillion" | "ppb" => Unit::PartsPerBillion,
                "MillimetersPerDay" | "mm/d" => Unit::MillimetersPerDay,
                "MillimetersPerHour" | "mm/h" => Unit::MillimetersPerHour,
                "FeetPerSecond" | "ft/s" => Unit::FeetPerSecond,
                "InchesPerDay" | "in/d" => Unit::InchesPerDay,
                "MetersPerSecond" | "m/s" => Unit::MetersPerSecond,
                "InchesPerHour" | "in/h" => Unit::InchesPerHour,
                "KilometersPerHour" | "km/h" => Unit::KilometersPerHour,
                "Knots" | "kn" => Unit::Knots,
                "MilesPerHour" | "mph" => Unit::MilesPerHour,
                "Bits" | "bit" => Unit::Bits,
                "Kilobits" | "kbit" => Unit::Kilobits,
                "Megabits" | "Mbit" => Unit::Megabits,
                "Gigabits" | "Gbit" => Unit::Gigabits,
                "Bytes" | "B" => Unit::Bytes,
                "Kilobytes" | "kB" => Unit::Kilobytes,
                "Megabytes" | "MB" => Unit::Megabytes,
                "Gigabytes" | "GB" => Unit::Gigabytes,
                "Terabytes" | "TB" => Unit::Terabytes,
                "Petabytes" | "PB" => Unit::Petabytes,
                "Exabytes" | "EB" => Unit::Exabytes,
                "Zettabytes" | "ZB" => Unit::Zettabytes,
                "Yottabytes" | "YB" => Unit::Yottabytes,
                "Kibibytes" | "KiB" => Unit::Kibibytes,
                "Mebibytes" | "MiB" => Unit::Mebibytes,
                "Gibibytes" | "GiB" => Unit::Gibibytes,
                "Tebibytes" | "TiB" => Unit::Tebibytes,
                "Pebibytes" | "PiB" => Unit::Pebibytes,
                "Exbibytes" | "EiB" => Unit::Exbibytes,
                "Zebibytes" | "ZiB" => Unit::Zebibytes,
                "Yobibytes" | "YiB" => Unit::Yobibytes,
                "BitsPerSecond" | "bit/s" => Unit::BitsPerSecond,
                "KilobitsPerSecond" | "kbit/s" => Unit::KilobitsPerSecond,
                "MegabitsPerSecond" | "Mbit/s" => Unit::MegabitsPerSecond,
                "GigabitsPerSecond" | "Gbit/s" => Unit::GigabitsPerSecond,
                "BytesPerSecond" | "B/s" => Unit::BytesPerSecond,
                "KilobytesPerSecond" | "kB/s" => Unit::KilobytesPerSecond,
                "MegabytesPerSecond" | "MB/s" => Unit::MegabytesPerSecond,
                "GigabytesPerSecond" | "GB/s" => Unit::GigabytesPerSecond,
                "KibibytesPerSecond" | "KiB/s" => Unit::KibibytesPerSecond,
                "MebibytesPerSecond" | "MiB/s" => Unit::MebibytesPerSecond,
                "GibibytesPerSecond" | "GiB/s" => Unit::GibibytesPerSecond,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::VoltAmperes => write!(f, "{}", "VoltAmperes"),
            Unit::Watts => write!(f, "{}", "Watts"),
            Unit::KiloWatts => write!(f, "{}", "KiloWatts"),
            Unit::BtusPerHour => write!(f, "{}", "BtusPerHour"),
            Unit::VoltAmpereReactive => write!(f, "{}", "VoltAmpereReactive"),
            Unit::WattHours => write!(f, "{}", "WattHours"),
            Unit::KiloWattHours => write!(f, "{}", "KiloWattHours"),
            Unit::MegaWattHours => write!(f, "{}", "MegaWattHours"),
            Unit::Milliamperes => write!(f, "{}", "Milliamperes"),
            Unit::Amperes => write!(f, "{}", "Amperes"),
            Unit::Millivolts => write!(f, "{}", "Millivolts"),
            Unit::Volts => write!(f, "{}", "Volts"),
            Unit::Degrees => write!(f, "{}", "Degrees"),
            Unit::Euros => write!(f, "{}", "Euros"),
            Unit::Dollars => write!(f, "{}", "Dollars"),
            Unit::Cents => write!(f, "{}", "Cents"),
            Unit::Celsius => write!(f, "{}", "Celsius"),
            Unit::Fahrenheit => write!(f, "{}", "Fahrenheit"),
            Unit::Kelvin => write!(f, "{}", "Kelvin"),
            Unit::Microseconds => write!(f, "{}", "Microseconds"),
            Unit::Milliseconds => write!(f, "{}", "Milliseconds"),
            Unit::Seconds => write!(f, "{}", "Seconds"),
            Unit::Minutes => write!(f, "{}", "Minutes"),
            Unit::Hours => write!(f, "{}", "Hours"),
            Unit::Days => write!(f, "{}", "Days"),
            Unit::Weeks => write!(f, "{}", "Weeks"),
            Unit::Months => write!(f, "{}", "Months"),
            Unit::Years => write!(f, "{}", "Years"),
            Unit::Millimeters => write!(f, "{}", "Millimeters"),
            Unit::Centimeters => write!(f, "{}", "Centimeters"),
            Unit::Meters => write!(f, "{}", "Meters"),
            Unit::Kilometers => write!(f, "{}", "Kilometers"),
            Unit::Inches => write!(f, "{}", "Inches"),
            Unit::Feet => write!(f, "{}", "Feet"),
            Unit::Yard => write!(f, "{}", "Yard"),
            Unit::Miles => write!(f, "{}", "Miles"),
            Unit::Hertz => write!(f, "{}", "Hertz"),
            Unit::Kilohertz => write!(f, "{}", "Kilohertz"),
            Unit::Megahertz => write!(f, "{}", "Megahertz"),
            Unit::Gigahertz => write!(f, "{}", "Gigahertz"),
            Unit::Pascal => write!(f, "{}", "Pascal"),
            Unit::Hectopascal => write!(f, "{}", "Hectopascal"),
            Unit::Kilopascal => write!(f, "{}", "Kilopascal"),
            Unit::Bar => write!(f, "{}", "Bar"),
            Unit::Centibar => write!(f, "{}", "Centibar"),
            Unit::Millibar => write!(f, "{}", "Millibar"),
            Unit::MillimeterMercury => write!(f, "{}", "MillimeterMercury"),
            Unit::InchMercury => write!(f, "{}", "InchMercury"),
            Unit::Psi => write!(f, "{}", "Psi"),
            Unit::Decibel => write!(f, "{}", "Decibel"),
            Unit::DecibelAWeighted => write!(f, "{}", "DecibelAWeighted"),
            Unit::DecibelsMilliwatt => write!(f, "{}", "DecibelsMilliwatt"),
            Unit::Liters => write!(f, "{}", "Liters"),
            Unit::Milliliters => write!(f, "{}", "Milliliters"),
            Unit::CubicMeters => write!(f, "{}", "CubicMeters"),
            Unit::CubicFeet => write!(f, "{}", "CubicFeet"),
            Unit::Gallons => write!(f, "{}", "Gallons"),
            Unit::FluidOunce => write!(f, "{}", "FluidOunce"),
            Unit::CubicMetersPerHour => write!(f, "{}", "CubicMetersPerHour"),
            Unit::CubicFeetPerMinute => write!(f, "{}", "CubicFeetPerMinute"),
            Unit::SquareMeters => write!(f, "{}", "SquareMeters"),
            Unit::Grams => write!(f, "{}", "Grams"),
            Unit::Kilograms => write!(f, "{}", "Kilograms"),
            Unit::Milligrams => write!(f, "{}", "Milligrams"),
            Unit::Micrograms => write!(f, "{}", "Micrograms"),
            Unit::Ounces => write!(f, "{}", "Ounces"),
            Unit::Pounds => write!(f, "{}", "Pounds"),
            Unit::MicrosiemensPerCentimeter => {
                write!(f, "{}", "MicrosiemensPerCentimeter")
            }
            Unit::Lux => write!(f, "{}", "Lux"),
            Unit::UvIndex => write!(f, "{}", "UvIndex"),
            Unit::Percentage => write!(f, "{}", "Percentage"),
            Unit::WattsPerSquareMeter => write!(f, "{}", "WattsPerSquareMeter"),
            Unit::BtusPerHourSquareFoot => write!(f, "{}", "BtusPerHourSquareFoot"),
            Unit::MicrogramsPerCubicMeter => write!(f, "{}", "MicrogramsPerCubicMeter"),
            Unit::MilligramsPerCubicMeter => write!(f, "{}", "MilligramsPerCubicMeter"),
            Unit::MicrogramsPerCubicFoot => write!(f, "{}", "MicrogramsPerCubicFoot"),
            Unit::PartsPerCubicMeter => write!(f, "{}", "PartsPerCubicMeter"),
            Unit::PartsPerMillion => write!(f, "{}", "PartsPerMillion"),
            Unit::PartsPerBillion => write!(f, "{}", "PartsPerBillion"),
            Unit::MillimetersPerDay => write!(f, "{}", "MillimetersPerDay"),
            Unit::MillimetersPerHour => write!(f, "{}", "MillimetersPerHour"),
            Unit::FeetPerSecond => write!(f, "{}", "FeetPerSecond"),
            Unit::InchesPerDay => write!(f, "{}", "InchesPerDay"),
            Unit::MetersPerSecond => write!(f, "{}", "MetersPerSecond"),
            Unit::InchesPerHour => write!(f, "{}", "InchesPerHour"),
            Unit::KilometersPerHour => write!(f, "{}", "KilometersPerHour"),
            Unit::Knots => write!(f, "{}", "Knots"),
            Unit::MilesPerHour => write!(f, "{}", "MilesPerHour"),
            Unit::Bits => write!(f, "{}", "Bits"),
            Unit::Kilobits => write!(f, "{}", "Kilobits"),
            Unit::Megabits => write!(f, "{}", "Megabits"),
            Unit::Gigabits => write!(f, "{}", "Gigabits"),
            Unit::Bytes => write!(f, "{}", "Bytes"),
            Unit::Kilobytes => write!(f, "{}", "Kilobytes"),
            Unit::Megabytes => write!(f, "{}", "Megabytes"),
            Unit::Gigabytes => write!(f, "{}", "Gigabytes"),
            Unit::Terabytes => write!(f, "{}", "Terabytes"),
            Unit::Petabytes => write!(f, "{}", "Petabytes"),
            Unit::Exabytes => write!(f, "{}", "Exabytes"),
            Unit::Zettabytes => write!(f, "{}", "Zettabytes"),
            Unit::Yottabytes => write!(f, "{}", "Yottabytes"),
            Unit::Kibibytes => write!(f, "{}", "Kibibytes"),
            Unit::Mebibytes => write!(f, "{}", "Mebibytes"),
            Unit::Gibibytes => write!(f, "{}", "Gibibytes"),
            Unit::Tebibytes => write!(f, "{}", "Tebibytes"),
            Unit::Pebibytes => write!(f, "{}", "Pebibytes"),
            Unit::Exbibytes => write!(f, "{}", "Exbibytes"),
            Unit::Zebibytes => write!(f, "{}", "Zebibytes"),
            Unit::Yobibytes => write!(f, "{}", "Yobibytes"),
            Unit::BitsPerSecond => write!(f, "{}", "BitsPerSecond"),
            Unit::KilobitsPerSecond => write!(f, "{}", "KilobitsPerSecond"),
            Unit::MegabitsPerSecond => write!(f, "{}", "MegabitsPerSecond"),
            Unit::GigabitsPerSecond => write!(f, "{}", "GigabitsPerSecond"),
            Unit::BytesPerSecond => write!(f, "{}", "BytesPerSecond"),
            Unit::KilobytesPerSecond => write!(f, "{}", "KilobytesPerSecond"),
            Unit::MegabytesPerSecond => write!(f, "{}", "MegabytesPerSecond"),
            Unit::GigabytesPerSecond => write!(f, "{}", "GigabytesPerSecond"),
            Unit::KibibytesPerSecond => write!(f, "{}", "KibibytesPerSecond"),
            Unit::MebibytesPerSecond => write!(f, "{}", "MebibytesPerSecond"),
            Unit::GibibytesPerSecond => write!(f, "{}", "GibibytesPerSecond"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FanOscillation {
    Off,
    On,
    Vertical,
    Horizontal,
    Both,
}
impl TryFrom<String> for FanOscillation {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Off" | "off" => FanOscillation::Off,
                "On" | "on" => FanOscillation::On,
                "Vertical" | "vertical" => FanOscillation::Vertical,
                "Horizontal" | "horizontal" => FanOscillation::Horizontal,
                "Both" | "both" => FanOscillation::Both,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for FanOscillation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FanOscillation::Off => write!(f, "{}", "Off"),
            FanOscillation::On => write!(f, "{}", "On"),
            FanOscillation::Vertical => write!(f, "{}", "Vertical"),
            FanOscillation::Horizontal => write!(f, "{}", "Horizontal"),
            FanOscillation::Both => write!(f, "{}", "Both"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FanDirection {
    Forward,
    Reverse,
}
impl TryFrom<String> for FanDirection {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Forward" | "forward" => FanDirection::Forward,
                "Reverse" | "reverse" => FanDirection::Reverse,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for FanDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FanDirection::Forward => write!(f, "{}", "Forward"),
            FanDirection::Reverse => write!(f, "{}", "Reverse"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FanSpeed {
    On,
    Off,
    Auto,
    Low,
    Medium,
    High,
    Middle,
    Focus,
    Diffuse,
    Quiet,
}
impl TryFrom<String> for FanSpeed {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "On" | "on" => FanSpeed::On,
                "Off" | "off" => FanSpeed::Off,
                "Auto" | "auto" => FanSpeed::Auto,
                "Low" | "low" => FanSpeed::Low,
                "Medium" | "medium" => FanSpeed::Medium,
                "High" | "high" => FanSpeed::High,
                "Middle" | "middle" => FanSpeed::Middle,
                "Focus" | "focus" => FanSpeed::Focus,
                "Diffuse" | "diffuse" => FanSpeed::Diffuse,
                "Quiet" | "quiet" => FanSpeed::Quiet,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FanSpeed::On => write!(f, "{}", "On"),
            FanSpeed::Off => write!(f, "{}", "Off"),
            FanSpeed::Auto => write!(f, "{}", "Auto"),
            FanSpeed::Low => write!(f, "{}", "Low"),
            FanSpeed::Medium => write!(f, "{}", "Medium"),
            FanSpeed::High => write!(f, "{}", "High"),
            FanSpeed::Middle => write!(f, "{}", "Middle"),
            FanSpeed::Focus => write!(f, "{}", "Focus"),
            FanSpeed::Diffuse => write!(f, "{}", "Diffuse"),
            FanSpeed::Quiet => write!(f, "{}", "Quiet"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ClimateMode {
    Off,
    Auto,
    Heat,
    Cool,
    HeatCool,
    FanOnly,
    Dry,
    Eco,
}
impl TryFrom<String> for ClimateMode {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Off" | "off" => ClimateMode::Off,
                "Auto" | "auto" => ClimateMode::Auto,
                "Heat" | "heat" => ClimateMode::Heat,
                "Cool" | "cool" => ClimateMode::Cool,
                "HeatCool" | "heat_cool" => ClimateMode::HeatCool,
                "FanOnly" | "fan_only" => ClimateMode::FanOnly,
                "Dry" | "dry" => ClimateMode::Dry,
                "Eco" | "eco" => ClimateMode::Eco,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for ClimateMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClimateMode::Off => write!(f, "{}", "Off"),
            ClimateMode::Auto => write!(f, "{}", "Auto"),
            ClimateMode::Heat => write!(f, "{}", "Heat"),
            ClimateMode::Cool => write!(f, "{}", "Cool"),
            ClimateMode::HeatCool => write!(f, "{}", "HeatCool"),
            ClimateMode::FanOnly => write!(f, "{}", "FanOnly"),
            ClimateMode::Dry => write!(f, "{}", "Dry"),
            ClimateMode::Eco => write!(f, "{}", "Eco"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LockState {
    Unknown,
    Locked,
    Unlocked,
    Jammed,
    Locking,
    Unlocking,
}
impl TryFrom<String> for LockState {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Unknown" | "unknown" => LockState::Unknown,
                "Locked" | "locked" => LockState::Locked,
                "Unlocked" | "unlocked" => LockState::Unlocked,
                "Jammed" | "jammed" => LockState::Jammed,
                "Locking" | "locking" => LockState::Locking,
                "Unlocking" | "unlocking" => LockState::Unlocking,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for LockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockState::Unknown => write!(f, "{}", "Unknown"),
            LockState::Locked => write!(f, "{}", "Locked"),
            LockState::Unlocked => write!(f, "{}", "Unlocked"),
            LockState::Jammed => write!(f, "{}", "Jammed"),
            LockState::Locking => write!(f, "{}", "Locking"),
            LockState::Unlocking => write!(f, "{}", "Unlocking"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MediaState {
    Unknown,
    Idle,
    Playing,
    Paused,
}
impl TryFrom<String> for MediaState {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Unknown" | "unknown" => MediaState::Unknown,
                "Idle" | "idle" => MediaState::Idle,
                "Playing" | "playing" => MediaState::Playing,
                "Paused" | "paused" => MediaState::Paused,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for MediaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaState::Unknown => write!(f, "{}", "Unknown"),
            MediaState::Idle => write!(f, "{}", "Idle"),
            MediaState::Playing => write!(f, "{}", "Playing"),
            MediaState::Paused => write!(f, "{}", "Paused"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CoverState {
    Idle,
    Opening,
    Closing,
    Stopped,
    Open,
    Closed,
}
impl TryFrom<String> for CoverState {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Idle" | "idle" => CoverState::Idle,
                "Opening" | "opening" => CoverState::Opening,
                "Closing" | "closing" => CoverState::Closing,
                "Stopped" | "stopped" => CoverState::Stopped,
                "Open" | "open" => CoverState::Open,
                "Closed" | "closed" => CoverState::Closed,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for CoverState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverState::Idle => write!(f, "{}", "Idle"),
            CoverState::Opening => write!(f, "{}", "Opening"),
            CoverState::Closing => write!(f, "{}", "Closing"),
            CoverState::Stopped => write!(f, "{}", "Stopped"),
            CoverState::Open => write!(f, "{}", "Open"),
            CoverState::Closed => write!(f, "{}", "Closed"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ValveState {
    Idle,
    Opening,
    Closing,
}
impl TryFrom<String> for ValveState {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Idle" | "idle" => ValveState::Idle,
                "Opening" | "opening" | "is_opening" => ValveState::Opening,
                "Closing" | "closing" | "is_closing" => ValveState::Closing,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for ValveState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValveState::Idle => write!(f, "{}", "Idle"),
            ValveState::Opening => write!(f, "{}", "Opening"),
            ValveState::Closing => write!(f, "{}", "Closing"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AlarmState {
    Disarmed,
    ArmedHome,
    ArmedAway,
    ArmedNight,
    ArmedVacation,
    ArmedUnknown,
    Pending,
    Triggered,
    Arming,
    Disarming,
}
impl TryFrom<String> for AlarmState {
    type Error = ();
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(
            match s.as_str() {
                "Disarmed" | "disarmed" => AlarmState::Disarmed,
                "ArmedHome" | "armed_home" => AlarmState::ArmedHome,
                "ArmedAway" | "armed_away" => AlarmState::ArmedAway,
                "ArmedNight" | "armed_night" => AlarmState::ArmedNight,
                "ArmedVacation" | "armed_vacation" => AlarmState::ArmedVacation,
                "ArmedUnknown" | "armed_unknown" | "armed" => AlarmState::ArmedUnknown,
                "Pending" | "pending" => AlarmState::Pending,
                "Triggered" | "triggered" => AlarmState::Triggered,
                "Arming" | "arming" => AlarmState::Arming,
                "Disarming" | "disarming" => AlarmState::Disarming,
                _ => return Err(()),
            },
        )
    }
}
impl std::fmt::Display for AlarmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlarmState::Disarmed => write!(f, "{}", "Disarmed"),
            AlarmState::ArmedHome => write!(f, "{}", "ArmedHome"),
            AlarmState::ArmedAway => write!(f, "{}", "ArmedAway"),
            AlarmState::ArmedNight => write!(f, "{}", "ArmedNight"),
            AlarmState::ArmedVacation => write!(f, "{}", "ArmedVacation"),
            AlarmState::ArmedUnknown => write!(f, "{}", "ArmedUnknown"),
            AlarmState::Pending => write!(f, "{}", "Pending"),
            AlarmState::Triggered => write!(f, "{}", "Triggered"),
            AlarmState::Arming => write!(f, "{}", "Arming"),
            AlarmState::Disarming => write!(f, "{}", "Disarming"),
        }
    }
}
impl ComponentType {
    pub fn snake_name(&self) -> &'static str {
        match self {
            ComponentType::Integer => "integer",
            ComponentType::Real => "real",
            ComponentType::Text => "text",
            ComponentType::Boolean => "boolean",
            ComponentType::Color => "color",
            ComponentType::Date => "date",
            ComponentType::Time => "time",
            ComponentType::IntegerList => "integer_list",
            ComponentType::RealList => "real_list",
            ComponentType::TextList => "text_list",
            ComponentType::BooleanList => "boolean_list",
            ComponentType::ColorList => "color_list",
            ComponentType::DateList => "date_list",
            ComponentType::TimeList => "time_list",
            ComponentType::Trigger => "trigger",
            ComponentType::Timestamp => "timestamp",
            ComponentType::Duration => "duration",
            ComponentType::Weekday => "weekday",
            ComponentType::Light => "light",
            ComponentType::Switch => "switch",
            ComponentType::Dimmer => "dimmer",
            ComponentType::ColorMode => "color_mode",
            ComponentType::ColorTemperature => "color_temperature",
            ComponentType::Volume => "volume",
            ComponentType::Muted => "muted",
            ComponentType::Config => "config",
            ComponentType::Diagnostic => "diagnostic",
            ComponentType::TextSelect => "text_select",
            ComponentType::Siren => "siren",
            ComponentType::Sensor => "sensor",
            ComponentType::Icon => "icon",
            ComponentType::AccuracyDecimals => "accuracy_decimals",
            ComponentType::DeviceClass => "device_class",
            ComponentType::SensorStateClass => "sensor_state_class",
            ComponentType::Unit => "unit",
            ComponentType::FanOscillation => "fan_oscillation",
            ComponentType::FanDirection => "fan_direction",
            ComponentType::FanSpeed => "fan_speed",
            ComponentType::ClimateMode => "climate_mode",
            ComponentType::LockState => "lock_state",
            ComponentType::MediaState => "media_state",
            ComponentType::Cover => "cover",
            ComponentType::CoverState => "cover_state",
            ComponentType::Position => "position",
            ComponentType::Tilt => "tilt",
            ComponentType::Valve => "valve",
            ComponentType::ValveState => "valve_state",
            ComponentType::AlarmState => "alarm_state",
        }
    }
    pub fn kebab_name(&self) -> &'static str {
        match self {
            ComponentType::Integer => "integer",
            ComponentType::Real => "real",
            ComponentType::Text => "text",
            ComponentType::Boolean => "boolean",
            ComponentType::Color => "color",
            ComponentType::Date => "date",
            ComponentType::Time => "time",
            ComponentType::IntegerList => "integer-list",
            ComponentType::RealList => "real-list",
            ComponentType::TextList => "text-list",
            ComponentType::BooleanList => "boolean-list",
            ComponentType::ColorList => "color-list",
            ComponentType::DateList => "date-list",
            ComponentType::TimeList => "time-list",
            ComponentType::Trigger => "trigger",
            ComponentType::Timestamp => "timestamp",
            ComponentType::Duration => "duration",
            ComponentType::Weekday => "weekday",
            ComponentType::Light => "light",
            ComponentType::Switch => "switch",
            ComponentType::Dimmer => "dimmer",
            ComponentType::ColorMode => "color-mode",
            ComponentType::ColorTemperature => "color-temperature",
            ComponentType::Volume => "volume",
            ComponentType::Muted => "muted",
            ComponentType::Config => "config",
            ComponentType::Diagnostic => "diagnostic",
            ComponentType::TextSelect => "text-select",
            ComponentType::Siren => "siren",
            ComponentType::Sensor => "sensor",
            ComponentType::Icon => "icon",
            ComponentType::AccuracyDecimals => "accuracy-decimals",
            ComponentType::DeviceClass => "device-class",
            ComponentType::SensorStateClass => "sensor-state-class",
            ComponentType::Unit => "unit",
            ComponentType::FanOscillation => "fan-oscillation",
            ComponentType::FanDirection => "fan-direction",
            ComponentType::FanSpeed => "fan-speed",
            ComponentType::ClimateMode => "climate-mode",
            ComponentType::LockState => "lock-state",
            ComponentType::MediaState => "media-state",
            ComponentType::Cover => "cover",
            ComponentType::CoverState => "cover-state",
            ComponentType::Position => "position",
            ComponentType::Tilt => "tilt",
            ComponentType::Valve => "valve",
            ComponentType::ValveState => "valve-state",
            ComponentType::AlarmState => "alarm-state",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u16)]
pub enum Component {
    Integer(IglooInteger),
    Real(IglooReal),
    Text(IglooText),
    Boolean(IglooBoolean),
    Color(IglooColor),
    Date(IglooDate),
    Time(IglooTime),
    IntegerList(IglooIntegerList),
    RealList(IglooRealList),
    TextList(IglooTextList),
    BooleanList(IglooBooleanList),
    ColorList(IglooColorList),
    DateList(IglooDateList),
    TimeList(IglooTimeList),
    Trigger,
    Timestamp(IglooInteger),
    Duration(IglooInteger),
    Weekday(Weekday),
    Light,
    Switch(IglooBoolean),
    Dimmer(IglooReal),
    ColorMode(ColorMode),
    ColorTemperature(IglooInteger),
    Volume(IglooReal),
    Muted(IglooBoolean),
    Config,
    Diagnostic,
    TextSelect,
    Siren,
    Sensor,
    Icon(IglooText),
    AccuracyDecimals(IglooInteger),
    DeviceClass(IglooText),
    SensorStateClass(SensorStateClass),
    Unit(Unit),
    FanOscillation(FanOscillation),
    FanDirection(FanDirection),
    FanSpeed(FanSpeed),
    ClimateMode(ClimateMode),
    LockState(LockState),
    MediaState(MediaState),
    Cover,
    CoverState(CoverState),
    Position(IglooReal),
    Tilt(IglooReal),
    Valve,
    ValveState(ValveState),
    AlarmState(AlarmState),
}
impl Component {
    pub fn get_type(&self) -> ComponentType {
        match self {
            Component::Integer(_) => ComponentType::Integer,
            Component::Real(_) => ComponentType::Real,
            Component::Text(_) => ComponentType::Text,
            Component::Boolean(_) => ComponentType::Boolean,
            Component::Color(_) => ComponentType::Color,
            Component::Date(_) => ComponentType::Date,
            Component::Time(_) => ComponentType::Time,
            Component::IntegerList(_) => ComponentType::IntegerList,
            Component::RealList(_) => ComponentType::RealList,
            Component::TextList(_) => ComponentType::TextList,
            Component::BooleanList(_) => ComponentType::BooleanList,
            Component::ColorList(_) => ComponentType::ColorList,
            Component::DateList(_) => ComponentType::DateList,
            Component::TimeList(_) => ComponentType::TimeList,
            Component::Trigger => ComponentType::Trigger,
            Component::Timestamp(_) => ComponentType::Timestamp,
            Component::Duration(_) => ComponentType::Duration,
            Component::Weekday(_) => ComponentType::Weekday,
            Component::Light => ComponentType::Light,
            Component::Switch(_) => ComponentType::Switch,
            Component::Dimmer(_) => ComponentType::Dimmer,
            Component::ColorMode(_) => ComponentType::ColorMode,
            Component::ColorTemperature(_) => ComponentType::ColorTemperature,
            Component::Volume(_) => ComponentType::Volume,
            Component::Muted(_) => ComponentType::Muted,
            Component::Config => ComponentType::Config,
            Component::Diagnostic => ComponentType::Diagnostic,
            Component::TextSelect => ComponentType::TextSelect,
            Component::Siren => ComponentType::Siren,
            Component::Sensor => ComponentType::Sensor,
            Component::Icon(_) => ComponentType::Icon,
            Component::AccuracyDecimals(_) => ComponentType::AccuracyDecimals,
            Component::DeviceClass(_) => ComponentType::DeviceClass,
            Component::SensorStateClass(_) => ComponentType::SensorStateClass,
            Component::Unit(_) => ComponentType::Unit,
            Component::FanOscillation(_) => ComponentType::FanOscillation,
            Component::FanDirection(_) => ComponentType::FanDirection,
            Component::FanSpeed(_) => ComponentType::FanSpeed,
            Component::ClimateMode(_) => ComponentType::ClimateMode,
            Component::LockState(_) => ComponentType::LockState,
            Component::MediaState(_) => ComponentType::MediaState,
            Component::Cover => ComponentType::Cover,
            Component::CoverState(_) => ComponentType::CoverState,
            Component::Position(_) => ComponentType::Position,
            Component::Tilt(_) => ComponentType::Tilt,
            Component::Valve => ComponentType::Valve,
            Component::ValveState(_) => ComponentType::ValveState,
            Component::AlarmState(_) => ComponentType::AlarmState,
        }
    }
}
impl Component {
    pub fn inner_string(&self) -> Option<String> {
        match self {
            Component::Integer(payload) => Some(format!("{payload:?}")),
            Component::Real(payload) => Some(format!("{payload:?}")),
            Component::Text(payload) => Some(format!("{payload:?}")),
            Component::Boolean(payload) => Some(format!("{payload:?}")),
            Component::Color(payload) => Some(format!("{payload:?}")),
            Component::Date(payload) => Some(format!("{payload:?}")),
            Component::Time(payload) => Some(format!("{payload:?}")),
            Component::IntegerList(payload) => Some(format!("{payload:?}")),
            Component::RealList(payload) => Some(format!("{payload:?}")),
            Component::TextList(payload) => Some(format!("{payload:?}")),
            Component::BooleanList(payload) => Some(format!("{payload:?}")),
            Component::ColorList(payload) => Some(format!("{payload:?}")),
            Component::DateList(payload) => Some(format!("{payload:?}")),
            Component::TimeList(payload) => Some(format!("{payload:?}")),
            Component::Timestamp(payload) => Some(format!("{payload:?}")),
            Component::Duration(payload) => Some(format!("{payload:?}")),
            Component::Weekday(payload) => Some(format!("{payload:?}")),
            Component::Switch(payload) => Some(format!("{payload:?}")),
            Component::Dimmer(payload) => Some(format!("{payload:?}")),
            Component::ColorMode(payload) => Some(format!("{payload:?}")),
            Component::ColorTemperature(payload) => Some(format!("{payload:?}")),
            Component::Volume(payload) => Some(format!("{payload:?}")),
            Component::Muted(payload) => Some(format!("{payload:?}")),
            Component::Icon(payload) => Some(format!("{payload:?}")),
            Component::AccuracyDecimals(payload) => Some(format!("{payload:?}")),
            Component::DeviceClass(payload) => Some(format!("{payload:?}")),
            Component::SensorStateClass(payload) => Some(format!("{payload:?}")),
            Component::Unit(payload) => Some(format!("{payload:?}")),
            Component::FanOscillation(payload) => Some(format!("{payload:?}")),
            Component::FanDirection(payload) => Some(format!("{payload:?}")),
            Component::FanSpeed(payload) => Some(format!("{payload:?}")),
            Component::ClimateMode(payload) => Some(format!("{payload:?}")),
            Component::LockState(payload) => Some(format!("{payload:?}")),
            Component::MediaState(payload) => Some(format!("{payload:?}")),
            Component::CoverState(payload) => Some(format!("{payload:?}")),
            Component::Position(payload) => Some(format!("{payload:?}")),
            Component::Tilt(payload) => Some(format!("{payload:?}")),
            Component::ValveState(payload) => Some(format!("{payload:?}")),
            Component::AlarmState(payload) => Some(format!("{payload:?}")),
            _ => None,
        }
    }
}
impl Component {
    pub fn from_string(comp_type: ComponentType, s: String) -> Option<Component> {
        match comp_type {
            ComponentType::Integer => s.parse().ok().map(Component::Integer),
            ComponentType::Real => s.parse().ok().map(Component::Real),
            ComponentType::Text => Some(Component::Text(s)),
            ComponentType::Boolean => s.parse().ok().map(Component::Boolean),
            ComponentType::Color => s.parse().ok().map(Component::Color),
            ComponentType::Date => s.parse().ok().map(Component::Date),
            ComponentType::Time => s.parse().ok().map(Component::Time),
            ComponentType::IntegerList => {
                parse_list(&s)?
                    .into_iter()
                    .map(|item| item.parse().ok())
                    .collect::<Option<Vec<_>>>()
                    .map(Component::IntegerList)
            }
            ComponentType::RealList => {
                parse_list(&s)?
                    .into_iter()
                    .map(|item| item.parse().ok())
                    .collect::<Option<Vec<_>>>()
                    .map(Component::RealList)
            }
            ComponentType::TextList => parse_list(&s).map(Component::TextList),
            ComponentType::BooleanList => {
                parse_list(&s)?
                    .into_iter()
                    .map(|item| item.parse().ok())
                    .collect::<Option<Vec<_>>>()
                    .map(Component::BooleanList)
            }
            ComponentType::ColorList => {
                parse_list(&s)?
                    .into_iter()
                    .map(|item| item.parse().ok())
                    .collect::<Option<Vec<_>>>()
                    .map(Component::ColorList)
            }
            ComponentType::DateList => {
                parse_list(&s)?
                    .into_iter()
                    .map(|item| item.parse().ok())
                    .collect::<Option<Vec<_>>>()
                    .map(Component::DateList)
            }
            ComponentType::TimeList => {
                parse_list(&s)?
                    .into_iter()
                    .map(|item| item.parse().ok())
                    .collect::<Option<Vec<_>>>()
                    .map(Component::TimeList)
            }
            ComponentType::Timestamp => s.parse().ok().map(Component::Timestamp),
            ComponentType::Duration => s.parse().ok().map(Component::Duration),
            ComponentType::Weekday => s.try_into().ok().map(Component::Weekday),
            ComponentType::Switch => s.parse().ok().map(Component::Switch),
            ComponentType::Dimmer => s.parse().ok().map(Component::Dimmer),
            ComponentType::ColorMode => s.try_into().ok().map(Component::ColorMode),
            ComponentType::ColorTemperature => {
                s.parse().ok().map(Component::ColorTemperature)
            }
            ComponentType::Volume => s.parse().ok().map(Component::Volume),
            ComponentType::Muted => s.parse().ok().map(Component::Muted),
            ComponentType::Icon => Some(Component::Icon(s)),
            ComponentType::AccuracyDecimals => {
                s.parse().ok().map(Component::AccuracyDecimals)
            }
            ComponentType::DeviceClass => Some(Component::DeviceClass(s)),
            ComponentType::SensorStateClass => {
                s.try_into().ok().map(Component::SensorStateClass)
            }
            ComponentType::Unit => s.try_into().ok().map(Component::Unit),
            ComponentType::FanOscillation => {
                s.try_into().ok().map(Component::FanOscillation)
            }
            ComponentType::FanDirection => s.try_into().ok().map(Component::FanDirection),
            ComponentType::FanSpeed => s.try_into().ok().map(Component::FanSpeed),
            ComponentType::ClimateMode => s.try_into().ok().map(Component::ClimateMode),
            ComponentType::LockState => s.try_into().ok().map(Component::LockState),
            ComponentType::MediaState => s.try_into().ok().map(Component::MediaState),
            ComponentType::CoverState => s.try_into().ok().map(Component::CoverState),
            ComponentType::Position => s.parse().ok().map(Component::Position),
            ComponentType::Tilt => s.parse().ok().map(Component::Tilt),
            ComponentType::ValveState => s.try_into().ok().map(Component::ValveState),
            ComponentType::AlarmState => s.try_into().ok().map(Component::AlarmState),
            _ => None,
        }
    }
}
impl Component {
    pub fn to_igloo_value(&self) -> Option<IglooValue> {
        match self {
            Component::Integer(v) => Some(IglooValue::Integer(v.clone())),
            Component::Real(v) => Some(IglooValue::Real(v.clone())),
            Component::Text(v) => Some(IglooValue::Text(v.clone())),
            Component::Boolean(v) => Some(IglooValue::Boolean(v.clone())),
            Component::Color(v) => Some(IglooValue::Color(v.clone())),
            Component::Date(v) => Some(IglooValue::Date(v.clone())),
            Component::Time(v) => Some(IglooValue::Time(v.clone())),
            Component::IntegerList(v) => Some(IglooValue::IntegerList(v.clone())),
            Component::RealList(v) => Some(IglooValue::RealList(v.clone())),
            Component::TextList(v) => Some(IglooValue::TextList(v.clone())),
            Component::BooleanList(v) => Some(IglooValue::BooleanList(v.clone())),
            Component::ColorList(v) => Some(IglooValue::ColorList(v.clone())),
            Component::DateList(v) => Some(IglooValue::DateList(v.clone())),
            Component::TimeList(v) => Some(IglooValue::TimeList(v.clone())),
            Component::Trigger => None,
            Component::Timestamp(v) => Some(IglooValue::Integer(v.clone())),
            Component::Duration(v) => Some(IglooValue::Integer(v.clone())),
            Component::Weekday(v) => {
                Some(IglooValue::Enum(IglooEnumValue::Weekday(v.clone())))
            }
            Component::Light => None,
            Component::Switch(v) => Some(IglooValue::Boolean(v.clone())),
            Component::Dimmer(v) => Some(IglooValue::Real(v.clone())),
            Component::ColorMode(v) => {
                Some(IglooValue::Enum(IglooEnumValue::ColorMode(v.clone())))
            }
            Component::ColorTemperature(v) => Some(IglooValue::Integer(v.clone())),
            Component::Volume(v) => Some(IglooValue::Real(v.clone())),
            Component::Muted(v) => Some(IglooValue::Boolean(v.clone())),
            Component::Config => None,
            Component::Diagnostic => None,
            Component::TextSelect => None,
            Component::Siren => None,
            Component::Sensor => None,
            Component::Icon(v) => Some(IglooValue::Text(v.clone())),
            Component::AccuracyDecimals(v) => Some(IglooValue::Integer(v.clone())),
            Component::DeviceClass(v) => Some(IglooValue::Text(v.clone())),
            Component::SensorStateClass(v) => {
                Some(IglooValue::Enum(IglooEnumValue::SensorStateClass(v.clone())))
            }
            Component::Unit(v) => Some(IglooValue::Enum(IglooEnumValue::Unit(v.clone()))),
            Component::FanOscillation(v) => {
                Some(IglooValue::Enum(IglooEnumValue::FanOscillation(v.clone())))
            }
            Component::FanDirection(v) => {
                Some(IglooValue::Enum(IglooEnumValue::FanDirection(v.clone())))
            }
            Component::FanSpeed(v) => {
                Some(IglooValue::Enum(IglooEnumValue::FanSpeed(v.clone())))
            }
            Component::ClimateMode(v) => {
                Some(IglooValue::Enum(IglooEnumValue::ClimateMode(v.clone())))
            }
            Component::LockState(v) => {
                Some(IglooValue::Enum(IglooEnumValue::LockState(v.clone())))
            }
            Component::MediaState(v) => {
                Some(IglooValue::Enum(IglooEnumValue::MediaState(v.clone())))
            }
            Component::Cover => None,
            Component::CoverState(v) => {
                Some(IglooValue::Enum(IglooEnumValue::CoverState(v.clone())))
            }
            Component::Position(v) => Some(IglooValue::Real(v.clone())),
            Component::Tilt(v) => Some(IglooValue::Real(v.clone())),
            Component::Valve => None,
            Component::ValveState(v) => {
                Some(IglooValue::Enum(IglooEnumValue::ValveState(v.clone())))
            }
            Component::AlarmState(v) => {
                Some(IglooValue::Enum(IglooEnumValue::AlarmState(v.clone())))
            }
        }
    }
}
impl Component {
    pub fn from_igloo_value(r#type: ComponentType, value: IglooValue) -> Option<Self> {
        match r#type {
            ComponentType::Integer => {
                if let IglooValue::Integer(v) = value {
                    Some(Component::Integer(v))
                } else {
                    None
                }
            }
            ComponentType::Real => {
                if let IglooValue::Real(v) = value {
                    Some(Component::Real(v))
                } else {
                    None
                }
            }
            ComponentType::Text => {
                if let IglooValue::Text(v) = value {
                    Some(Component::Text(v))
                } else {
                    None
                }
            }
            ComponentType::Boolean => {
                if let IglooValue::Boolean(v) = value {
                    Some(Component::Boolean(v))
                } else {
                    None
                }
            }
            ComponentType::Color => {
                if let IglooValue::Color(v) = value {
                    Some(Component::Color(v))
                } else {
                    None
                }
            }
            ComponentType::Date => {
                if let IglooValue::Date(v) = value {
                    Some(Component::Date(v))
                } else {
                    None
                }
            }
            ComponentType::Time => {
                if let IglooValue::Time(v) = value {
                    Some(Component::Time(v))
                } else {
                    None
                }
            }
            ComponentType::IntegerList => {
                if let IglooValue::IntegerList(v) = value {
                    Some(Component::IntegerList(v))
                } else {
                    None
                }
            }
            ComponentType::RealList => {
                if let IglooValue::RealList(v) = value {
                    Some(Component::RealList(v))
                } else {
                    None
                }
            }
            ComponentType::TextList => {
                if let IglooValue::TextList(v) = value {
                    Some(Component::TextList(v))
                } else {
                    None
                }
            }
            ComponentType::BooleanList => {
                if let IglooValue::BooleanList(v) = value {
                    Some(Component::BooleanList(v))
                } else {
                    None
                }
            }
            ComponentType::ColorList => {
                if let IglooValue::ColorList(v) = value {
                    Some(Component::ColorList(v))
                } else {
                    None
                }
            }
            ComponentType::DateList => {
                if let IglooValue::DateList(v) = value {
                    Some(Component::DateList(v))
                } else {
                    None
                }
            }
            ComponentType::TimeList => {
                if let IglooValue::TimeList(v) = value {
                    Some(Component::TimeList(v))
                } else {
                    None
                }
            }
            ComponentType::Trigger => Some(Component::Trigger),
            ComponentType::Timestamp => {
                if let IglooValue::Integer(v) = value {
                    Some(Component::Timestamp(v))
                } else {
                    None
                }
            }
            ComponentType::Duration => {
                if let IglooValue::Integer(v) = value {
                    Some(Component::Duration(v))
                } else {
                    None
                }
            }
            ComponentType::Weekday => {
                if let IglooValue::Enum(IglooEnumValue::Weekday(v)) = value {
                    Some(Component::Weekday(v))
                } else {
                    None
                }
            }
            ComponentType::Light => Some(Component::Light),
            ComponentType::Switch => {
                if let IglooValue::Boolean(v) = value {
                    Some(Component::Switch(v))
                } else {
                    None
                }
            }
            ComponentType::Dimmer => {
                if let IglooValue::Real(v) = value {
                    Some(Component::Dimmer(v))
                } else {
                    None
                }
            }
            ComponentType::ColorMode => {
                if let IglooValue::Enum(IglooEnumValue::ColorMode(v)) = value {
                    Some(Component::ColorMode(v))
                } else {
                    None
                }
            }
            ComponentType::ColorTemperature => {
                if let IglooValue::Integer(v) = value {
                    Some(Component::ColorTemperature(v))
                } else {
                    None
                }
            }
            ComponentType::Volume => {
                if let IglooValue::Real(v) = value {
                    Some(Component::Volume(v))
                } else {
                    None
                }
            }
            ComponentType::Muted => {
                if let IglooValue::Boolean(v) = value {
                    Some(Component::Muted(v))
                } else {
                    None
                }
            }
            ComponentType::Config => Some(Component::Config),
            ComponentType::Diagnostic => Some(Component::Diagnostic),
            ComponentType::TextSelect => Some(Component::TextSelect),
            ComponentType::Siren => Some(Component::Siren),
            ComponentType::Sensor => Some(Component::Sensor),
            ComponentType::Icon => {
                if let IglooValue::Text(v) = value {
                    Some(Component::Icon(v))
                } else {
                    None
                }
            }
            ComponentType::AccuracyDecimals => {
                if let IglooValue::Integer(v) = value {
                    Some(Component::AccuracyDecimals(v))
                } else {
                    None
                }
            }
            ComponentType::DeviceClass => {
                if let IglooValue::Text(v) = value {
                    Some(Component::DeviceClass(v))
                } else {
                    None
                }
            }
            ComponentType::SensorStateClass => {
                if let IglooValue::Enum(IglooEnumValue::SensorStateClass(v)) = value {
                    Some(Component::SensorStateClass(v))
                } else {
                    None
                }
            }
            ComponentType::Unit => {
                if let IglooValue::Enum(IglooEnumValue::Unit(v)) = value {
                    Some(Component::Unit(v))
                } else {
                    None
                }
            }
            ComponentType::FanOscillation => {
                if let IglooValue::Enum(IglooEnumValue::FanOscillation(v)) = value {
                    Some(Component::FanOscillation(v))
                } else {
                    None
                }
            }
            ComponentType::FanDirection => {
                if let IglooValue::Enum(IglooEnumValue::FanDirection(v)) = value {
                    Some(Component::FanDirection(v))
                } else {
                    None
                }
            }
            ComponentType::FanSpeed => {
                if let IglooValue::Enum(IglooEnumValue::FanSpeed(v)) = value {
                    Some(Component::FanSpeed(v))
                } else {
                    None
                }
            }
            ComponentType::ClimateMode => {
                if let IglooValue::Enum(IglooEnumValue::ClimateMode(v)) = value {
                    Some(Component::ClimateMode(v))
                } else {
                    None
                }
            }
            ComponentType::LockState => {
                if let IglooValue::Enum(IglooEnumValue::LockState(v)) = value {
                    Some(Component::LockState(v))
                } else {
                    None
                }
            }
            ComponentType::MediaState => {
                if let IglooValue::Enum(IglooEnumValue::MediaState(v)) = value {
                    Some(Component::MediaState(v))
                } else {
                    None
                }
            }
            ComponentType::Cover => Some(Component::Cover),
            ComponentType::CoverState => {
                if let IglooValue::Enum(IglooEnumValue::CoverState(v)) = value {
                    Some(Component::CoverState(v))
                } else {
                    None
                }
            }
            ComponentType::Position => {
                if let IglooValue::Real(v) = value {
                    Some(Component::Position(v))
                } else {
                    None
                }
            }
            ComponentType::Tilt => {
                if let IglooValue::Real(v) = value {
                    Some(Component::Tilt(v))
                } else {
                    None
                }
            }
            ComponentType::Valve => Some(Component::Valve),
            ComponentType::ValveState => {
                if let IglooValue::Enum(IglooEnumValue::ValveState(v)) = value {
                    Some(Component::ValveState(v))
                } else {
                    None
                }
            }
            ComponentType::AlarmState => {
                if let IglooValue::Enum(IglooEnumValue::AlarmState(v)) = value {
                    Some(Component::AlarmState(v))
                } else {
                    None
                }
            }
        }
    }
}
use std::ops::ControlFlow;
#[derive(Debug, Clone)]
pub enum Aggregator {
    IntegerSum { sum: i64 },
    IntegerMean { sum: i64, count: usize },
    IntegerMax { val: Option<i64> },
    IntegerMin { val: Option<i64> },
    RealSum { sum: f64 },
    RealMean { sum: f64, count: usize },
    RealMax { val: Option<f64> },
    RealMin { val: Option<f64> },
    BooleanMean { true_count: usize, total: usize },
    BooleanAny { found: bool },
    BooleanAll { all_true: bool },
    ColorMean { sum_r: f64, sum_g: f64, sum_b: f64, count: usize },
    ColorMax { val: Option<IglooColor> },
    ColorMin { val: Option<IglooColor> },
    DateMean { sum: i64, count: usize },
    DateMax { val: Option<IglooDate> },
    DateMin { val: Option<IglooDate> },
    TimeMean { sum: i64, count: usize },
    TimeMax { val: Option<IglooTime> },
    TimeMin { val: Option<IglooTime> },
    TimestampSum { sum: i64 },
    TimestampMean { sum: i64, count: usize },
    TimestampMax { val: Option<i64> },
    TimestampMin { val: Option<i64> },
    DurationSum { sum: i64 },
    DurationMean { sum: i64, count: usize },
    DurationMax { val: Option<i64> },
    DurationMin { val: Option<i64> },
    WeekdayMean { counts: [usize; 7usize], total: usize },
    SwitchMean { true_count: usize, total: usize },
    SwitchAny { found: bool },
    SwitchAll { all_true: bool },
    DimmerSum { sum: f64 },
    DimmerMean { sum: f64, count: usize },
    DimmerMax { val: Option<f64> },
    DimmerMin { val: Option<f64> },
    ColorModeMean { counts: [usize; 2usize], total: usize },
    ColorTemperatureSum { sum: i64 },
    ColorTemperatureMean { sum: i64, count: usize },
    ColorTemperatureMax { val: Option<i64> },
    ColorTemperatureMin { val: Option<i64> },
    VolumeSum { sum: f64 },
    VolumeMean { sum: f64, count: usize },
    VolumeMax { val: Option<f64> },
    VolumeMin { val: Option<f64> },
    MutedMean { true_count: usize, total: usize },
    MutedAny { found: bool },
    MutedAll { all_true: bool },
    AccuracyDecimalsSum { sum: i64 },
    AccuracyDecimalsMean { sum: i64, count: usize },
    AccuracyDecimalsMax { val: Option<i64> },
    AccuracyDecimalsMin { val: Option<i64> },
    SensorStateClassMean { counts: [usize; 3usize], total: usize },
    UnitMean { counts: [usize; 120usize], total: usize },
    FanOscillationMean { counts: [usize; 5usize], total: usize },
    FanDirectionMean { counts: [usize; 2usize], total: usize },
    FanSpeedMean { counts: [usize; 10usize], total: usize },
    ClimateModeMean { counts: [usize; 8usize], total: usize },
    LockStateMean { counts: [usize; 6usize], total: usize },
    MediaStateMean { counts: [usize; 4usize], total: usize },
    CoverStateMean { counts: [usize; 6usize], total: usize },
    PositionSum { sum: f64 },
    PositionMean { sum: f64, count: usize },
    PositionMax { val: Option<f64> },
    PositionMin { val: Option<f64> },
    TiltSum { sum: f64 },
    TiltMean { sum: f64, count: usize },
    TiltMax { val: Option<f64> },
    TiltMin { val: Option<f64> },
    ValveStateMean { counts: [usize; 3usize], total: usize },
    AlarmStateMean { counts: [usize; 10usize], total: usize },
}
impl Aggregator {
    pub fn new(comp_type: ComponentType, op: AggregationOp) -> Option<Self> {
        match (comp_type, op) {
            (ComponentType::Integer, AggregationOp::Sum) => {
                Some(Aggregator::IntegerSum { sum: 0 })
            }
            (ComponentType::Integer, AggregationOp::Mean) => {
                Some(Aggregator::IntegerMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::Integer, AggregationOp::Max) => {
                Some(Aggregator::IntegerMax {
                    val: None,
                })
            }
            (ComponentType::Integer, AggregationOp::Min) => {
                Some(Aggregator::IntegerMin {
                    val: None,
                })
            }
            (ComponentType::Real, AggregationOp::Sum) => {
                Some(Aggregator::RealSum { sum: 0.0 })
            }
            (ComponentType::Real, AggregationOp::Mean) => {
                Some(Aggregator::RealMean {
                    sum: 0.0,
                    count: 0,
                })
            }
            (ComponentType::Real, AggregationOp::Max) => {
                Some(Aggregator::RealMax { val: None })
            }
            (ComponentType::Real, AggregationOp::Min) => {
                Some(Aggregator::RealMin { val: None })
            }
            (ComponentType::Boolean, AggregationOp::Mean) => {
                Some(Aggregator::BooleanMean {
                    true_count: 0,
                    total: 0,
                })
            }
            (ComponentType::Boolean, AggregationOp::Any) => {
                Some(Aggregator::BooleanAny {
                    found: false,
                })
            }
            (ComponentType::Boolean, AggregationOp::All) => {
                Some(Aggregator::BooleanAll {
                    all_true: true,
                })
            }
            (ComponentType::Color, AggregationOp::Mean) => {
                Some(Aggregator::ColorMean {
                    sum_r: 0.0,
                    sum_g: 0.0,
                    sum_b: 0.0,
                    count: 0,
                })
            }
            (ComponentType::Color, AggregationOp::Max) => {
                Some(Aggregator::ColorMax { val: None })
            }
            (ComponentType::Color, AggregationOp::Min) => {
                Some(Aggregator::ColorMin { val: None })
            }
            (ComponentType::Date, AggregationOp::Mean) => {
                Some(Aggregator::DateMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::Date, AggregationOp::Max) => {
                Some(Aggregator::DateMax { val: None })
            }
            (ComponentType::Date, AggregationOp::Min) => {
                Some(Aggregator::DateMin { val: None })
            }
            (ComponentType::Time, AggregationOp::Mean) => {
                Some(Aggregator::TimeMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::Time, AggregationOp::Max) => {
                Some(Aggregator::TimeMax { val: None })
            }
            (ComponentType::Time, AggregationOp::Min) => {
                Some(Aggregator::TimeMin { val: None })
            }
            (ComponentType::Timestamp, AggregationOp::Sum) => {
                Some(Aggregator::TimestampSum { sum: 0 })
            }
            (ComponentType::Timestamp, AggregationOp::Mean) => {
                Some(Aggregator::TimestampMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::Timestamp, AggregationOp::Max) => {
                Some(Aggregator::TimestampMax {
                    val: None,
                })
            }
            (ComponentType::Timestamp, AggregationOp::Min) => {
                Some(Aggregator::TimestampMin {
                    val: None,
                })
            }
            (ComponentType::Duration, AggregationOp::Sum) => {
                Some(Aggregator::DurationSum { sum: 0 })
            }
            (ComponentType::Duration, AggregationOp::Mean) => {
                Some(Aggregator::DurationMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::Duration, AggregationOp::Max) => {
                Some(Aggregator::DurationMax {
                    val: None,
                })
            }
            (ComponentType::Duration, AggregationOp::Min) => {
                Some(Aggregator::DurationMin {
                    val: None,
                })
            }
            (ComponentType::Weekday, AggregationOp::Mean) => {
                Some(Aggregator::WeekdayMean {
                    counts: [0; 7usize],
                    total: 0,
                })
            }
            (ComponentType::Switch, AggregationOp::Mean) => {
                Some(Aggregator::SwitchMean {
                    true_count: 0,
                    total: 0,
                })
            }
            (ComponentType::Switch, AggregationOp::Any) => {
                Some(Aggregator::SwitchAny {
                    found: false,
                })
            }
            (ComponentType::Switch, AggregationOp::All) => {
                Some(Aggregator::SwitchAll {
                    all_true: true,
                })
            }
            (ComponentType::Dimmer, AggregationOp::Sum) => {
                Some(Aggregator::DimmerSum { sum: 0.0 })
            }
            (ComponentType::Dimmer, AggregationOp::Mean) => {
                Some(Aggregator::DimmerMean {
                    sum: 0.0,
                    count: 0,
                })
            }
            (ComponentType::Dimmer, AggregationOp::Max) => {
                Some(Aggregator::DimmerMax { val: None })
            }
            (ComponentType::Dimmer, AggregationOp::Min) => {
                Some(Aggregator::DimmerMin { val: None })
            }
            (ComponentType::ColorMode, AggregationOp::Mean) => {
                Some(Aggregator::ColorModeMean {
                    counts: [0; 2usize],
                    total: 0,
                })
            }
            (ComponentType::ColorTemperature, AggregationOp::Sum) => {
                Some(Aggregator::ColorTemperatureSum {
                    sum: 0,
                })
            }
            (ComponentType::ColorTemperature, AggregationOp::Mean) => {
                Some(Aggregator::ColorTemperatureMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::ColorTemperature, AggregationOp::Max) => {
                Some(Aggregator::ColorTemperatureMax {
                    val: None,
                })
            }
            (ComponentType::ColorTemperature, AggregationOp::Min) => {
                Some(Aggregator::ColorTemperatureMin {
                    val: None,
                })
            }
            (ComponentType::Volume, AggregationOp::Sum) => {
                Some(Aggregator::VolumeSum { sum: 0.0 })
            }
            (ComponentType::Volume, AggregationOp::Mean) => {
                Some(Aggregator::VolumeMean {
                    sum: 0.0,
                    count: 0,
                })
            }
            (ComponentType::Volume, AggregationOp::Max) => {
                Some(Aggregator::VolumeMax { val: None })
            }
            (ComponentType::Volume, AggregationOp::Min) => {
                Some(Aggregator::VolumeMin { val: None })
            }
            (ComponentType::Muted, AggregationOp::Mean) => {
                Some(Aggregator::MutedMean {
                    true_count: 0,
                    total: 0,
                })
            }
            (ComponentType::Muted, AggregationOp::Any) => {
                Some(Aggregator::MutedAny {
                    found: false,
                })
            }
            (ComponentType::Muted, AggregationOp::All) => {
                Some(Aggregator::MutedAll {
                    all_true: true,
                })
            }
            (ComponentType::AccuracyDecimals, AggregationOp::Sum) => {
                Some(Aggregator::AccuracyDecimalsSum {
                    sum: 0,
                })
            }
            (ComponentType::AccuracyDecimals, AggregationOp::Mean) => {
                Some(Aggregator::AccuracyDecimalsMean {
                    sum: 0,
                    count: 0,
                })
            }
            (ComponentType::AccuracyDecimals, AggregationOp::Max) => {
                Some(Aggregator::AccuracyDecimalsMax {
                    val: None,
                })
            }
            (ComponentType::AccuracyDecimals, AggregationOp::Min) => {
                Some(Aggregator::AccuracyDecimalsMin {
                    val: None,
                })
            }
            (ComponentType::SensorStateClass, AggregationOp::Mean) => {
                Some(Aggregator::SensorStateClassMean {
                    counts: [0; 3usize],
                    total: 0,
                })
            }
            (ComponentType::Unit, AggregationOp::Mean) => {
                Some(Aggregator::UnitMean {
                    counts: [0; 120usize],
                    total: 0,
                })
            }
            (ComponentType::FanOscillation, AggregationOp::Mean) => {
                Some(Aggregator::FanOscillationMean {
                    counts: [0; 5usize],
                    total: 0,
                })
            }
            (ComponentType::FanDirection, AggregationOp::Mean) => {
                Some(Aggregator::FanDirectionMean {
                    counts: [0; 2usize],
                    total: 0,
                })
            }
            (ComponentType::FanSpeed, AggregationOp::Mean) => {
                Some(Aggregator::FanSpeedMean {
                    counts: [0; 10usize],
                    total: 0,
                })
            }
            (ComponentType::ClimateMode, AggregationOp::Mean) => {
                Some(Aggregator::ClimateModeMean {
                    counts: [0; 8usize],
                    total: 0,
                })
            }
            (ComponentType::LockState, AggregationOp::Mean) => {
                Some(Aggregator::LockStateMean {
                    counts: [0; 6usize],
                    total: 0,
                })
            }
            (ComponentType::MediaState, AggregationOp::Mean) => {
                Some(Aggregator::MediaStateMean {
                    counts: [0; 4usize],
                    total: 0,
                })
            }
            (ComponentType::CoverState, AggregationOp::Mean) => {
                Some(Aggregator::CoverStateMean {
                    counts: [0; 6usize],
                    total: 0,
                })
            }
            (ComponentType::Position, AggregationOp::Sum) => {
                Some(Aggregator::PositionSum {
                    sum: 0.0,
                })
            }
            (ComponentType::Position, AggregationOp::Mean) => {
                Some(Aggregator::PositionMean {
                    sum: 0.0,
                    count: 0,
                })
            }
            (ComponentType::Position, AggregationOp::Max) => {
                Some(Aggregator::PositionMax {
                    val: None,
                })
            }
            (ComponentType::Position, AggregationOp::Min) => {
                Some(Aggregator::PositionMin {
                    val: None,
                })
            }
            (ComponentType::Tilt, AggregationOp::Sum) => {
                Some(Aggregator::TiltSum { sum: 0.0 })
            }
            (ComponentType::Tilt, AggregationOp::Mean) => {
                Some(Aggregator::TiltMean {
                    sum: 0.0,
                    count: 0,
                })
            }
            (ComponentType::Tilt, AggregationOp::Max) => {
                Some(Aggregator::TiltMax { val: None })
            }
            (ComponentType::Tilt, AggregationOp::Min) => {
                Some(Aggregator::TiltMin { val: None })
            }
            (ComponentType::ValveState, AggregationOp::Mean) => {
                Some(Aggregator::ValveStateMean {
                    counts: [0; 3usize],
                    total: 0,
                })
            }
            (ComponentType::AlarmState, AggregationOp::Mean) => {
                Some(Aggregator::AlarmStateMean {
                    counts: [0; 10usize],
                    total: 0,
                })
            }
            _ => None,
        }
    }
    pub fn push(&mut self, comp: &Component) -> ControlFlow<()> {
        match self {
            Aggregator::IntegerSum { sum } => {
                if let Component::Integer(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::IntegerMean { sum, count } => {
                if let Component::Integer(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::IntegerMax { val } => {
                if let Component::Integer(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v > m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::IntegerMin { val } => {
                if let Component::Integer(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v < m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::RealSum { sum } => {
                if let Component::Real(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::RealMean { sum, count } => {
                if let Component::Real(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::RealMax { val } => {
                if let Component::Real(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Greater => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::RealMin { val } => {
                if let Component::Real(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Less => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::BooleanMean { true_count, total } => {
                if let Component::Boolean(v) = comp {
                    if *v {
                        *true_count += 1;
                    }
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::BooleanAny { found } => {
                if let Component::Boolean(true) = comp {
                    *found = true;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
            Aggregator::BooleanAll { all_true } => {
                if let Component::Boolean(false) = comp {
                    *all_true = false;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorMean { sum_r, sum_g, sum_b, count } => {
                if let Component::Color(c) = comp {
                    *sum_r += c.r;
                    *sum_g += c.g;
                    *sum_b += c.b;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorMax { val } => {
                if let Component::Color(c) = comp {
                    *val = Some(
                        match val {
                            None => c.clone(),
                            Some(m) => {
                                let cmp = (c.r, c.g, c.b)
                                    .partial_cmp(&(m.r, m.g, m.b))
                                    .unwrap_or(Ordering::Equal);
                                if cmp == Ordering::Greater { c.clone() } else { m.clone() }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorMin { val } => {
                if let Component::Color(c) = comp {
                    *val = Some(
                        match val {
                            None => c.clone(),
                            Some(m) => {
                                let cmp = (c.r, c.g, c.b)
                                    .partial_cmp(&(m.r, m.g, m.b))
                                    .unwrap_or(Ordering::Equal);
                                if cmp == Ordering::Less { c.clone() } else { m.clone() }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::DateMean { sum, count } => {
                if let Component::Date(d) = comp {
                    *sum += d.days_since_epoch() as i64;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::DateMax { val } => {
                if let Component::Date(d) = comp {
                    *val = Some(
                        match *val {
                            None => d.clone(),
                            Some(m) => {
                                if d.days_since_epoch() > m.days_since_epoch() {
                                    d.clone()
                                } else {
                                    m
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::DateMin { val } => {
                if let Component::Date(d) = comp {
                    *val = Some(
                        match *val {
                            None => d.clone(),
                            Some(m) => {
                                if d.days_since_epoch() < m.days_since_epoch() {
                                    d.clone()
                                } else {
                                    m
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimeMean { sum, count } => {
                if let Component::Time(t) = comp {
                    *sum += t.to_seconds() as i64;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimeMax { val } => {
                if let Component::Time(t) = comp {
                    *val = Some(
                        match *val {
                            None => t.clone(),
                            Some(m) => {
                                if t.to_seconds() > m.to_seconds() { t.clone() } else { m }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimeMin { val } => {
                if let Component::Time(t) = comp {
                    *val = Some(
                        match *val {
                            None => t.clone(),
                            Some(m) => {
                                if t.to_seconds() < m.to_seconds() { t.clone() } else { m }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimestampSum { sum } => {
                if let Component::Timestamp(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimestampMean { sum, count } => {
                if let Component::Timestamp(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimestampMax { val } => {
                if let Component::Timestamp(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v > m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::TimestampMin { val } => {
                if let Component::Timestamp(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v < m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::DurationSum { sum } => {
                if let Component::Duration(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::DurationMean { sum, count } => {
                if let Component::Duration(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::DurationMax { val } => {
                if let Component::Duration(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v > m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::DurationMin { val } => {
                if let Component::Duration(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v < m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::WeekdayMean { counts, total } => {
                if let Component::Weekday(v) = comp {
                    let idx = match v {
                        Weekday::Sunday => 0usize,
                        Weekday::Monday => 1usize,
                        Weekday::Tuesday => 2usize,
                        Weekday::Wednesday => 3usize,
                        Weekday::Thursday => 4usize,
                        Weekday::Friday => 5usize,
                        Weekday::Saturday => 6usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::SwitchMean { true_count, total } => {
                if let Component::Switch(v) = comp {
                    if *v {
                        *true_count += 1;
                    }
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::SwitchAny { found } => {
                if let Component::Switch(true) = comp {
                    *found = true;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
            Aggregator::SwitchAll { all_true } => {
                if let Component::Switch(false) = comp {
                    *all_true = false;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
            Aggregator::DimmerSum { sum } => {
                if let Component::Dimmer(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::DimmerMean { sum, count } => {
                if let Component::Dimmer(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::DimmerMax { val } => {
                if let Component::Dimmer(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Greater => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::DimmerMin { val } => {
                if let Component::Dimmer(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Less => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorModeMean { counts, total } => {
                if let Component::ColorMode(v) = comp {
                    let idx = match v {
                        ColorMode::RGB => 0usize,
                        ColorMode::Temperature => 1usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorTemperatureSum { sum } => {
                if let Component::ColorTemperature(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorTemperatureMean { sum, count } => {
                if let Component::ColorTemperature(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorTemperatureMax { val } => {
                if let Component::ColorTemperature(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v > m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::ColorTemperatureMin { val } => {
                if let Component::ColorTemperature(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v < m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::VolumeSum { sum } => {
                if let Component::Volume(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::VolumeMean { sum, count } => {
                if let Component::Volume(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::VolumeMax { val } => {
                if let Component::Volume(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Greater => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::VolumeMin { val } => {
                if let Component::Volume(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Less => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::MutedMean { true_count, total } => {
                if let Component::Muted(v) = comp {
                    if *v {
                        *true_count += 1;
                    }
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::MutedAny { found } => {
                if let Component::Muted(true) = comp {
                    *found = true;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
            Aggregator::MutedAll { all_true } => {
                if let Component::Muted(false) = comp {
                    *all_true = false;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            }
            Aggregator::AccuracyDecimalsSum { sum } => {
                if let Component::AccuracyDecimals(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::AccuracyDecimalsMean { sum, count } => {
                if let Component::AccuracyDecimals(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::AccuracyDecimalsMax { val } => {
                if let Component::AccuracyDecimals(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v > m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::AccuracyDecimalsMin { val } => {
                if let Component::AccuracyDecimals(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => if *v < m { *v } else { m }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::SensorStateClassMean { counts, total } => {
                if let Component::SensorStateClass(v) = comp {
                    let idx = match v {
                        SensorStateClass::Measurement => 0usize,
                        SensorStateClass::TotalIncreasing => 1usize,
                        SensorStateClass::Total => 2usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::UnitMean { counts, total } => {
                if let Component::Unit(v) = comp {
                    let idx = match v {
                        Unit::VoltAmperes => 0usize,
                        Unit::Watts => 1usize,
                        Unit::KiloWatts => 2usize,
                        Unit::BtusPerHour => 3usize,
                        Unit::VoltAmpereReactive => 4usize,
                        Unit::WattHours => 5usize,
                        Unit::KiloWattHours => 6usize,
                        Unit::MegaWattHours => 7usize,
                        Unit::Milliamperes => 8usize,
                        Unit::Amperes => 9usize,
                        Unit::Millivolts => 10usize,
                        Unit::Volts => 11usize,
                        Unit::Degrees => 12usize,
                        Unit::Euros => 13usize,
                        Unit::Dollars => 14usize,
                        Unit::Cents => 15usize,
                        Unit::Celsius => 16usize,
                        Unit::Fahrenheit => 17usize,
                        Unit::Kelvin => 18usize,
                        Unit::Microseconds => 19usize,
                        Unit::Milliseconds => 20usize,
                        Unit::Seconds => 21usize,
                        Unit::Minutes => 22usize,
                        Unit::Hours => 23usize,
                        Unit::Days => 24usize,
                        Unit::Weeks => 25usize,
                        Unit::Months => 26usize,
                        Unit::Years => 27usize,
                        Unit::Millimeters => 28usize,
                        Unit::Centimeters => 29usize,
                        Unit::Meters => 30usize,
                        Unit::Kilometers => 31usize,
                        Unit::Inches => 32usize,
                        Unit::Feet => 33usize,
                        Unit::Yard => 34usize,
                        Unit::Miles => 35usize,
                        Unit::Hertz => 36usize,
                        Unit::Kilohertz => 37usize,
                        Unit::Megahertz => 38usize,
                        Unit::Gigahertz => 39usize,
                        Unit::Pascal => 40usize,
                        Unit::Hectopascal => 41usize,
                        Unit::Kilopascal => 42usize,
                        Unit::Bar => 43usize,
                        Unit::Centibar => 44usize,
                        Unit::Millibar => 45usize,
                        Unit::MillimeterMercury => 46usize,
                        Unit::InchMercury => 47usize,
                        Unit::Psi => 48usize,
                        Unit::Decibel => 49usize,
                        Unit::DecibelAWeighted => 50usize,
                        Unit::DecibelsMilliwatt => 51usize,
                        Unit::Liters => 52usize,
                        Unit::Milliliters => 53usize,
                        Unit::CubicMeters => 54usize,
                        Unit::CubicFeet => 55usize,
                        Unit::Gallons => 56usize,
                        Unit::FluidOunce => 57usize,
                        Unit::CubicMetersPerHour => 58usize,
                        Unit::CubicFeetPerMinute => 59usize,
                        Unit::SquareMeters => 60usize,
                        Unit::Grams => 61usize,
                        Unit::Kilograms => 62usize,
                        Unit::Milligrams => 63usize,
                        Unit::Micrograms => 64usize,
                        Unit::Ounces => 65usize,
                        Unit::Pounds => 66usize,
                        Unit::MicrosiemensPerCentimeter => 67usize,
                        Unit::Lux => 68usize,
                        Unit::UvIndex => 69usize,
                        Unit::Percentage => 70usize,
                        Unit::WattsPerSquareMeter => 71usize,
                        Unit::BtusPerHourSquareFoot => 72usize,
                        Unit::MicrogramsPerCubicMeter => 73usize,
                        Unit::MilligramsPerCubicMeter => 74usize,
                        Unit::MicrogramsPerCubicFoot => 75usize,
                        Unit::PartsPerCubicMeter => 76usize,
                        Unit::PartsPerMillion => 77usize,
                        Unit::PartsPerBillion => 78usize,
                        Unit::MillimetersPerDay => 79usize,
                        Unit::MillimetersPerHour => 80usize,
                        Unit::FeetPerSecond => 81usize,
                        Unit::InchesPerDay => 82usize,
                        Unit::MetersPerSecond => 83usize,
                        Unit::InchesPerHour => 84usize,
                        Unit::KilometersPerHour => 85usize,
                        Unit::Knots => 86usize,
                        Unit::MilesPerHour => 87usize,
                        Unit::Bits => 88usize,
                        Unit::Kilobits => 89usize,
                        Unit::Megabits => 90usize,
                        Unit::Gigabits => 91usize,
                        Unit::Bytes => 92usize,
                        Unit::Kilobytes => 93usize,
                        Unit::Megabytes => 94usize,
                        Unit::Gigabytes => 95usize,
                        Unit::Terabytes => 96usize,
                        Unit::Petabytes => 97usize,
                        Unit::Exabytes => 98usize,
                        Unit::Zettabytes => 99usize,
                        Unit::Yottabytes => 100usize,
                        Unit::Kibibytes => 101usize,
                        Unit::Mebibytes => 102usize,
                        Unit::Gibibytes => 103usize,
                        Unit::Tebibytes => 104usize,
                        Unit::Pebibytes => 105usize,
                        Unit::Exbibytes => 106usize,
                        Unit::Zebibytes => 107usize,
                        Unit::Yobibytes => 108usize,
                        Unit::BitsPerSecond => 109usize,
                        Unit::KilobitsPerSecond => 110usize,
                        Unit::MegabitsPerSecond => 111usize,
                        Unit::GigabitsPerSecond => 112usize,
                        Unit::BytesPerSecond => 113usize,
                        Unit::KilobytesPerSecond => 114usize,
                        Unit::MegabytesPerSecond => 115usize,
                        Unit::GigabytesPerSecond => 116usize,
                        Unit::KibibytesPerSecond => 117usize,
                        Unit::MebibytesPerSecond => 118usize,
                        Unit::GibibytesPerSecond => 119usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::FanOscillationMean { counts, total } => {
                if let Component::FanOscillation(v) = comp {
                    let idx = match v {
                        FanOscillation::Off => 0usize,
                        FanOscillation::On => 1usize,
                        FanOscillation::Vertical => 2usize,
                        FanOscillation::Horizontal => 3usize,
                        FanOscillation::Both => 4usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::FanDirectionMean { counts, total } => {
                if let Component::FanDirection(v) = comp {
                    let idx = match v {
                        FanDirection::Forward => 0usize,
                        FanDirection::Reverse => 1usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::FanSpeedMean { counts, total } => {
                if let Component::FanSpeed(v) = comp {
                    let idx = match v {
                        FanSpeed::On => 0usize,
                        FanSpeed::Off => 1usize,
                        FanSpeed::Auto => 2usize,
                        FanSpeed::Low => 3usize,
                        FanSpeed::Medium => 4usize,
                        FanSpeed::High => 5usize,
                        FanSpeed::Middle => 6usize,
                        FanSpeed::Focus => 7usize,
                        FanSpeed::Diffuse => 8usize,
                        FanSpeed::Quiet => 9usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::ClimateModeMean { counts, total } => {
                if let Component::ClimateMode(v) = comp {
                    let idx = match v {
                        ClimateMode::Off => 0usize,
                        ClimateMode::Auto => 1usize,
                        ClimateMode::Heat => 2usize,
                        ClimateMode::Cool => 3usize,
                        ClimateMode::HeatCool => 4usize,
                        ClimateMode::FanOnly => 5usize,
                        ClimateMode::Dry => 6usize,
                        ClimateMode::Eco => 7usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::LockStateMean { counts, total } => {
                if let Component::LockState(v) = comp {
                    let idx = match v {
                        LockState::Unknown => 0usize,
                        LockState::Locked => 1usize,
                        LockState::Unlocked => 2usize,
                        LockState::Jammed => 3usize,
                        LockState::Locking => 4usize,
                        LockState::Unlocking => 5usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::MediaStateMean { counts, total } => {
                if let Component::MediaState(v) = comp {
                    let idx = match v {
                        MediaState::Unknown => 0usize,
                        MediaState::Idle => 1usize,
                        MediaState::Playing => 2usize,
                        MediaState::Paused => 3usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::CoverStateMean { counts, total } => {
                if let Component::CoverState(v) = comp {
                    let idx = match v {
                        CoverState::Idle => 0usize,
                        CoverState::Opening => 1usize,
                        CoverState::Closing => 2usize,
                        CoverState::Stopped => 3usize,
                        CoverState::Open => 4usize,
                        CoverState::Closed => 5usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::PositionSum { sum } => {
                if let Component::Position(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::PositionMean { sum, count } => {
                if let Component::Position(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::PositionMax { val } => {
                if let Component::Position(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Greater => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::PositionMin { val } => {
                if let Component::Position(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Less => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::TiltSum { sum } => {
                if let Component::Tilt(v) = comp {
                    *sum += v;
                }
                ControlFlow::Continue(())
            }
            Aggregator::TiltMean { sum, count } => {
                if let Component::Tilt(v) = comp {
                    *sum += v;
                    *count += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::TiltMax { val } => {
                if let Component::Tilt(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Greater => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::TiltMin { val } => {
                if let Component::Tilt(v) = comp {
                    *val = Some(
                        match *val {
                            None => *v,
                            Some(m) => {
                                match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                                    Ordering::Less => *v,
                                    _ => m,
                                }
                            }
                        },
                    );
                }
                ControlFlow::Continue(())
            }
            Aggregator::ValveStateMean { counts, total } => {
                if let Component::ValveState(v) = comp {
                    let idx = match v {
                        ValveState::Idle => 0usize,
                        ValveState::Opening => 1usize,
                        ValveState::Closing => 2usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
            Aggregator::AlarmStateMean { counts, total } => {
                if let Component::AlarmState(v) = comp {
                    let idx = match v {
                        AlarmState::Disarmed => 0usize,
                        AlarmState::ArmedHome => 1usize,
                        AlarmState::ArmedAway => 2usize,
                        AlarmState::ArmedNight => 3usize,
                        AlarmState::ArmedVacation => 4usize,
                        AlarmState::ArmedUnknown => 5usize,
                        AlarmState::Pending => 6usize,
                        AlarmState::Triggered => 7usize,
                        AlarmState::Arming => 8usize,
                        AlarmState::Disarming => 9usize,
                    };
                    counts[idx] += 1;
                    *total += 1;
                }
                ControlFlow::Continue(())
            }
        }
    }
    pub fn finish(self) -> Option<IglooValue> {
        match self {
            Aggregator::IntegerSum { sum } => Some(IglooValue::Integer(sum)),
            Aggregator::IntegerMean { sum, count } => {
                (count > 0).then(|| IglooValue::Integer(sum / count as i64))
            }
            Aggregator::IntegerMax { val } => val.map(IglooValue::Integer),
            Aggregator::IntegerMin { val } => val.map(IglooValue::Integer),
            Aggregator::RealSum { sum } => Some(IglooValue::Real(sum)),
            Aggregator::RealMean { sum, count } => {
                (count > 0).then(|| IglooValue::Real(sum / count as f64))
            }
            Aggregator::RealMax { val } => val.map(IglooValue::Real),
            Aggregator::RealMin { val } => val.map(IglooValue::Real),
            Aggregator::BooleanMean { true_count, total } => {
                (total > 0).then(|| IglooValue::Boolean(true_count * 2 >= total))
            }
            Aggregator::BooleanAny { found } => Some(IglooValue::Boolean(found)),
            Aggregator::BooleanAll { all_true } => Some(IglooValue::Boolean(all_true)),
            Aggregator::ColorMean { sum_r, sum_g, sum_b, count } => {
                (count > 0)
                    .then(|| IglooValue::Color(IglooColor {
                        r: sum_r / count as f64,
                        g: sum_g / count as f64,
                        b: sum_b / count as f64,
                    }))
            }
            Aggregator::ColorMax { val } => val.map(IglooValue::Color),
            Aggregator::ColorMin { val } => val.map(IglooValue::Color),
            Aggregator::DateMean { sum, count } => {
                (count > 0)
                    .then(|| IglooValue::Date(
                        IglooDate::from_days_since_epoch((sum / count as i64) as i32),
                    ))
            }
            Aggregator::DateMax { val } => val.map(IglooValue::Date),
            Aggregator::DateMin { val } => val.map(IglooValue::Date),
            Aggregator::TimeMean { sum, count } => {
                (count > 0)
                    .then(|| IglooValue::Time(
                        IglooTime::from_seconds((sum / count as i64) as i32),
                    ))
            }
            Aggregator::TimeMax { val } => val.map(IglooValue::Time),
            Aggregator::TimeMin { val } => val.map(IglooValue::Time),
            Aggregator::TimestampSum { sum } => Some(IglooValue::Integer(sum)),
            Aggregator::TimestampMean { sum, count } => {
                (count > 0).then(|| IglooValue::Integer(sum / count as i64))
            }
            Aggregator::TimestampMax { val } => val.map(IglooValue::Integer),
            Aggregator::TimestampMin { val } => val.map(IglooValue::Integer),
            Aggregator::DurationSum { sum } => Some(IglooValue::Integer(sum)),
            Aggregator::DurationMean { sum, count } => {
                (count > 0).then(|| IglooValue::Integer(sum / count as i64))
            }
            Aggregator::DurationMax { val } => val.map(IglooValue::Integer),
            Aggregator::DurationMin { val } => val.map(IglooValue::Integer),
            Aggregator::WeekdayMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..7usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => Weekday::Sunday,
                    1usize => Weekday::Monday,
                    2usize => Weekday::Tuesday,
                    3usize => Weekday::Wednesday,
                    4usize => Weekday::Thursday,
                    5usize => Weekday::Friday,
                    6usize => Weekday::Saturday,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::Weekday(result)))
            }
            Aggregator::SwitchMean { true_count, total } => {
                (total > 0).then(|| IglooValue::Boolean(true_count * 2 >= total))
            }
            Aggregator::SwitchAny { found } => Some(IglooValue::Boolean(found)),
            Aggregator::SwitchAll { all_true } => Some(IglooValue::Boolean(all_true)),
            Aggregator::DimmerSum { sum } => Some(IglooValue::Real(sum)),
            Aggregator::DimmerMean { sum, count } => {
                (count > 0).then(|| IglooValue::Real(sum / count as f64))
            }
            Aggregator::DimmerMax { val } => val.map(IglooValue::Real),
            Aggregator::DimmerMin { val } => val.map(IglooValue::Real),
            Aggregator::ColorModeMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..2usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => ColorMode::RGB,
                    1usize => ColorMode::Temperature,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::ColorMode(result)))
            }
            Aggregator::ColorTemperatureSum { sum } => Some(IglooValue::Integer(sum)),
            Aggregator::ColorTemperatureMean { sum, count } => {
                (count > 0).then(|| IglooValue::Integer(sum / count as i64))
            }
            Aggregator::ColorTemperatureMax { val } => val.map(IglooValue::Integer),
            Aggregator::ColorTemperatureMin { val } => val.map(IglooValue::Integer),
            Aggregator::VolumeSum { sum } => Some(IglooValue::Real(sum)),
            Aggregator::VolumeMean { sum, count } => {
                (count > 0).then(|| IglooValue::Real(sum / count as f64))
            }
            Aggregator::VolumeMax { val } => val.map(IglooValue::Real),
            Aggregator::VolumeMin { val } => val.map(IglooValue::Real),
            Aggregator::MutedMean { true_count, total } => {
                (total > 0).then(|| IglooValue::Boolean(true_count * 2 >= total))
            }
            Aggregator::MutedAny { found } => Some(IglooValue::Boolean(found)),
            Aggregator::MutedAll { all_true } => Some(IglooValue::Boolean(all_true)),
            Aggregator::AccuracyDecimalsSum { sum } => Some(IglooValue::Integer(sum)),
            Aggregator::AccuracyDecimalsMean { sum, count } => {
                (count > 0).then(|| IglooValue::Integer(sum / count as i64))
            }
            Aggregator::AccuracyDecimalsMax { val } => val.map(IglooValue::Integer),
            Aggregator::AccuracyDecimalsMin { val } => val.map(IglooValue::Integer),
            Aggregator::SensorStateClassMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..3usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => SensorStateClass::Measurement,
                    1usize => SensorStateClass::TotalIncreasing,
                    2usize => SensorStateClass::Total,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::SensorStateClass(result)))
            }
            Aggregator::UnitMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..120usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => Unit::VoltAmperes,
                    1usize => Unit::Watts,
                    2usize => Unit::KiloWatts,
                    3usize => Unit::BtusPerHour,
                    4usize => Unit::VoltAmpereReactive,
                    5usize => Unit::WattHours,
                    6usize => Unit::KiloWattHours,
                    7usize => Unit::MegaWattHours,
                    8usize => Unit::Milliamperes,
                    9usize => Unit::Amperes,
                    10usize => Unit::Millivolts,
                    11usize => Unit::Volts,
                    12usize => Unit::Degrees,
                    13usize => Unit::Euros,
                    14usize => Unit::Dollars,
                    15usize => Unit::Cents,
                    16usize => Unit::Celsius,
                    17usize => Unit::Fahrenheit,
                    18usize => Unit::Kelvin,
                    19usize => Unit::Microseconds,
                    20usize => Unit::Milliseconds,
                    21usize => Unit::Seconds,
                    22usize => Unit::Minutes,
                    23usize => Unit::Hours,
                    24usize => Unit::Days,
                    25usize => Unit::Weeks,
                    26usize => Unit::Months,
                    27usize => Unit::Years,
                    28usize => Unit::Millimeters,
                    29usize => Unit::Centimeters,
                    30usize => Unit::Meters,
                    31usize => Unit::Kilometers,
                    32usize => Unit::Inches,
                    33usize => Unit::Feet,
                    34usize => Unit::Yard,
                    35usize => Unit::Miles,
                    36usize => Unit::Hertz,
                    37usize => Unit::Kilohertz,
                    38usize => Unit::Megahertz,
                    39usize => Unit::Gigahertz,
                    40usize => Unit::Pascal,
                    41usize => Unit::Hectopascal,
                    42usize => Unit::Kilopascal,
                    43usize => Unit::Bar,
                    44usize => Unit::Centibar,
                    45usize => Unit::Millibar,
                    46usize => Unit::MillimeterMercury,
                    47usize => Unit::InchMercury,
                    48usize => Unit::Psi,
                    49usize => Unit::Decibel,
                    50usize => Unit::DecibelAWeighted,
                    51usize => Unit::DecibelsMilliwatt,
                    52usize => Unit::Liters,
                    53usize => Unit::Milliliters,
                    54usize => Unit::CubicMeters,
                    55usize => Unit::CubicFeet,
                    56usize => Unit::Gallons,
                    57usize => Unit::FluidOunce,
                    58usize => Unit::CubicMetersPerHour,
                    59usize => Unit::CubicFeetPerMinute,
                    60usize => Unit::SquareMeters,
                    61usize => Unit::Grams,
                    62usize => Unit::Kilograms,
                    63usize => Unit::Milligrams,
                    64usize => Unit::Micrograms,
                    65usize => Unit::Ounces,
                    66usize => Unit::Pounds,
                    67usize => Unit::MicrosiemensPerCentimeter,
                    68usize => Unit::Lux,
                    69usize => Unit::UvIndex,
                    70usize => Unit::Percentage,
                    71usize => Unit::WattsPerSquareMeter,
                    72usize => Unit::BtusPerHourSquareFoot,
                    73usize => Unit::MicrogramsPerCubicMeter,
                    74usize => Unit::MilligramsPerCubicMeter,
                    75usize => Unit::MicrogramsPerCubicFoot,
                    76usize => Unit::PartsPerCubicMeter,
                    77usize => Unit::PartsPerMillion,
                    78usize => Unit::PartsPerBillion,
                    79usize => Unit::MillimetersPerDay,
                    80usize => Unit::MillimetersPerHour,
                    81usize => Unit::FeetPerSecond,
                    82usize => Unit::InchesPerDay,
                    83usize => Unit::MetersPerSecond,
                    84usize => Unit::InchesPerHour,
                    85usize => Unit::KilometersPerHour,
                    86usize => Unit::Knots,
                    87usize => Unit::MilesPerHour,
                    88usize => Unit::Bits,
                    89usize => Unit::Kilobits,
                    90usize => Unit::Megabits,
                    91usize => Unit::Gigabits,
                    92usize => Unit::Bytes,
                    93usize => Unit::Kilobytes,
                    94usize => Unit::Megabytes,
                    95usize => Unit::Gigabytes,
                    96usize => Unit::Terabytes,
                    97usize => Unit::Petabytes,
                    98usize => Unit::Exabytes,
                    99usize => Unit::Zettabytes,
                    100usize => Unit::Yottabytes,
                    101usize => Unit::Kibibytes,
                    102usize => Unit::Mebibytes,
                    103usize => Unit::Gibibytes,
                    104usize => Unit::Tebibytes,
                    105usize => Unit::Pebibytes,
                    106usize => Unit::Exbibytes,
                    107usize => Unit::Zebibytes,
                    108usize => Unit::Yobibytes,
                    109usize => Unit::BitsPerSecond,
                    110usize => Unit::KilobitsPerSecond,
                    111usize => Unit::MegabitsPerSecond,
                    112usize => Unit::GigabitsPerSecond,
                    113usize => Unit::BytesPerSecond,
                    114usize => Unit::KilobytesPerSecond,
                    115usize => Unit::MegabytesPerSecond,
                    116usize => Unit::GigabytesPerSecond,
                    117usize => Unit::KibibytesPerSecond,
                    118usize => Unit::MebibytesPerSecond,
                    119usize => Unit::GibibytesPerSecond,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::Unit(result)))
            }
            Aggregator::FanOscillationMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..5usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => FanOscillation::Off,
                    1usize => FanOscillation::On,
                    2usize => FanOscillation::Vertical,
                    3usize => FanOscillation::Horizontal,
                    4usize => FanOscillation::Both,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::FanOscillation(result)))
            }
            Aggregator::FanDirectionMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..2usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => FanDirection::Forward,
                    1usize => FanDirection::Reverse,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::FanDirection(result)))
            }
            Aggregator::FanSpeedMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..10usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => FanSpeed::On,
                    1usize => FanSpeed::Off,
                    2usize => FanSpeed::Auto,
                    3usize => FanSpeed::Low,
                    4usize => FanSpeed::Medium,
                    5usize => FanSpeed::High,
                    6usize => FanSpeed::Middle,
                    7usize => FanSpeed::Focus,
                    8usize => FanSpeed::Diffuse,
                    9usize => FanSpeed::Quiet,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::FanSpeed(result)))
            }
            Aggregator::ClimateModeMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..8usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => ClimateMode::Off,
                    1usize => ClimateMode::Auto,
                    2usize => ClimateMode::Heat,
                    3usize => ClimateMode::Cool,
                    4usize => ClimateMode::HeatCool,
                    5usize => ClimateMode::FanOnly,
                    6usize => ClimateMode::Dry,
                    7usize => ClimateMode::Eco,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::ClimateMode(result)))
            }
            Aggregator::LockStateMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..6usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => LockState::Unknown,
                    1usize => LockState::Locked,
                    2usize => LockState::Unlocked,
                    3usize => LockState::Jammed,
                    4usize => LockState::Locking,
                    5usize => LockState::Unlocking,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::LockState(result)))
            }
            Aggregator::MediaStateMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..4usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => MediaState::Unknown,
                    1usize => MediaState::Idle,
                    2usize => MediaState::Playing,
                    3usize => MediaState::Paused,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::MediaState(result)))
            }
            Aggregator::CoverStateMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..6usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => CoverState::Idle,
                    1usize => CoverState::Opening,
                    2usize => CoverState::Closing,
                    3usize => CoverState::Stopped,
                    4usize => CoverState::Open,
                    5usize => CoverState::Closed,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::CoverState(result)))
            }
            Aggregator::PositionSum { sum } => Some(IglooValue::Real(sum)),
            Aggregator::PositionMean { sum, count } => {
                (count > 0).then(|| IglooValue::Real(sum / count as f64))
            }
            Aggregator::PositionMax { val } => val.map(IglooValue::Real),
            Aggregator::PositionMin { val } => val.map(IglooValue::Real),
            Aggregator::TiltSum { sum } => Some(IglooValue::Real(sum)),
            Aggregator::TiltMean { sum, count } => {
                (count > 0).then(|| IglooValue::Real(sum / count as f64))
            }
            Aggregator::TiltMax { val } => val.map(IglooValue::Real),
            Aggregator::TiltMin { val } => val.map(IglooValue::Real),
            Aggregator::ValveStateMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..3usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => ValveState::Idle,
                    1usize => ValveState::Opening,
                    2usize => ValveState::Closing,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::ValveState(result)))
            }
            Aggregator::AlarmStateMean { counts, total } => {
                if total == 0 {
                    return None;
                }
                let mut max_idx = 0;
                for i in 1..10usize {
                    if counts[i] > counts[max_idx] {
                        max_idx = i;
                    }
                }
                let result = match max_idx {
                    0usize => AlarmState::Disarmed,
                    1usize => AlarmState::ArmedHome,
                    2usize => AlarmState::ArmedAway,
                    3usize => AlarmState::ArmedNight,
                    4usize => AlarmState::ArmedVacation,
                    5usize => AlarmState::ArmedUnknown,
                    6usize => AlarmState::Pending,
                    7usize => AlarmState::Triggered,
                    8usize => AlarmState::Arming,
                    9usize => AlarmState::Disarming,
                    _ => unreachable!(),
                };
                Some(IglooValue::Enum(IglooEnumValue::AlarmState(result)))
            }
        }
    }
}
impl AggregationOp {
    pub fn can_apply(&self, comp_type: &ComponentType) -> bool {
        use AggregationOp::*;
        match comp_type {
            ComponentType::Integer => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Real => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Boolean => matches!(self, Mean | Any | All),
            ComponentType::Color => matches!(self, Mean | Max | Min),
            ComponentType::Date => matches!(self, Mean | Max | Min),
            ComponentType::Time => matches!(self, Mean | Max | Min),
            ComponentType::Timestamp => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Duration => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Weekday => matches!(self, Mean),
            ComponentType::Switch => matches!(self, Mean | Any | All),
            ComponentType::Dimmer => matches!(self, Sum | Mean | Max | Min),
            ComponentType::ColorMode => matches!(self, Mean),
            ComponentType::ColorTemperature => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Volume => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Muted => matches!(self, Mean | Any | All),
            ComponentType::AccuracyDecimals => matches!(self, Sum | Mean | Max | Min),
            ComponentType::SensorStateClass => matches!(self, Mean),
            ComponentType::Unit => matches!(self, Mean),
            ComponentType::FanOscillation => matches!(self, Mean),
            ComponentType::FanDirection => matches!(self, Mean),
            ComponentType::FanSpeed => matches!(self, Mean),
            ComponentType::ClimateMode => matches!(self, Mean),
            ComponentType::LockState => matches!(self, Mean),
            ComponentType::MediaState => matches!(self, Mean),
            ComponentType::CoverState => matches!(self, Mean),
            ComponentType::Position => matches!(self, Sum | Mean | Max | Min),
            ComponentType::Tilt => matches!(self, Sum | Mean | Max | Min),
            ComponentType::ValveState => matches!(self, Mean),
            ComponentType::AlarmState => matches!(self, Mean),
            _ => false,
        }
    }
}
