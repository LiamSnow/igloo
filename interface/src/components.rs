// THIS IS GENERATED CODE - DO NOT MODIFY
// Generated from components.rs by build.rs

use serde::{Deserialize, Serialize};
use crate::{Entity, Averageable};
use std::ops::{Add, Sub};

// This file is transpiled into components.rs via build.rs
// Which generates necessary methods, add #[derive(..)] and pub prefix
// Note: Make sure to make a fields public!!

/// signed 32-bit integer
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Int(pub i32);

impl Averageable for Int {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as i32)
    }
}
    
/// unsigned 32-bit integer
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Uint(pub u32);

impl Averageable for Uint {
    type Sum = u64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as u32)
    }
}
    
/// signed 64-bit integer
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Long(pub i64);

impl Averageable for Long {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    
/// 64-bit floating point
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Float(pub f64);

impl Averageable for Float {
    type Sum = f64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Bool(pub bool);

impl Averageable for Bool {
    type Sum = u32;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) != 0)
    }
}
    
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Text(pub String);

// TODO should we have this?
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Object(pub std::collections::HashMap<String, Component>);

// TODO should we have this?
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct List(pub Vec<Component>);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IntList(pub Vec<i32>);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UintList(pub Vec<u32>);
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LongList(pub Vec<i64>);
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FloatList(pub Vec<f64>);
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextList(pub Vec<String>);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Date(pub jiff::civil::Date);
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Time(pub jiff::civil::Time);
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DateTime(pub jiff::civil::DateTime);
// Weekday(Weekday),
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Duration(pub jiff::SignedDuration);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Switch(pub bool);

impl Averageable for Switch {
    type Sum = u32;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) != 0)
    }
}
    
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dimmer(pub u8);

impl Averageable for Dimmer {
    type Sum = u64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as u8)
    }
}
    
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Color {
    // IMPLEMENT AVERAGEABLE
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
#[derive(Clone, Debug, Default)]
pub struct ColorSum {
	pub r: u64,
	pub g: u64,
	pub b: u64,
}
impl Add for ColorSum {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		ColorSum {
			r: self.r + other.r,
			g: self.g + other.g,
			b: self.b + other.b,
		}
	}
}
impl Sub for ColorSum {
	type Output = Self;
	fn sub(self, other: Self) -> Self {
		ColorSum {
			r: self.r - other.r,
			g: self.g - other.g,
			b: self.b - other.b,
		}
	}
}
impl Averageable for Color {
	type Sum = ColorSum;
	fn to_sum_component(&self) -> Self::Sum {
		ColorSum {
			r: self.r as u64,
			g: self.g as u64,
			b: self.b as u64,
		}
	}
	fn from_sum(sum: Self::Sum, len: usize) -> Self {
		Color {
			r: (sum.r / len as u64) as u8,
			g: (sum.g / len as u64) as u8,
			b: (sum.b / len as u64) as u8,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Unit {
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FloatMin(pub f64);

impl Averageable for FloatMin {
    type Sum = f64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    /// usually used in combination with a Float to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FloatMax(pub f64);

impl Averageable for FloatMax {
    type Sum = f64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    /// usually used in combination with a Float to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FloatStep(pub f64);

impl Averageable for FloatStep {
    type Sum = f64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    
/// usually used in combination with an Int to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IntMin(pub i32);

impl Averageable for IntMin {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as i32)
    }
}
    /// usually used in combination with an Int to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IntMax(pub i32);

impl Averageable for IntMax {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as i32)
    }
}
    /// usually used in combination with an Int to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IntStep(pub i32);

impl Averageable for IntStep {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as i32)
    }
}
    
/// usually used in combination with an Long to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LongMin(pub i64);

impl Averageable for LongMin {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    /// usually used in combination with an Long to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LongMax(pub i64);

impl Averageable for LongMax {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    /// usually used in combination with an Long to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LongStep(pub i64);

impl Averageable for LongStep {
    type Sum = i64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    
/// usually used in combination with an Uint to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UintMin(pub u32);

impl Averageable for UintMin {
    type Sum = u64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as u32)
    }
}
    /// usually used in combination with an Uint to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UintMax(pub u32);

impl Averageable for UintMax {
    type Sum = u64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as u32)
    }
}
    /// usually used in combination with an Uint to set a range
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UintStep(pub u32);

impl Averageable for UintStep {
    type Sum = u64;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self((sum / len as Self::Sum) as u32)
    }
}
    
/// usually used in combination with a Switch and sometimes a Dimmer, Color, and/or ColorTemperature
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Light;

/// usually used in combination with a Switch and sometimes a Dimmer, Color, and/or ColorTemperature
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LightBulb;

/// this is just a marker meant to be comined with a Uint
/// and optional UintMin and UintMax
/// for clarity you can also add Unit::Kelvin
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ColorTemperature;

/// used in combination with Text to set a max length
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextMaxLength(pub usize);

impl Averageable for TextMaxLength {
    type Sum = usize;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    /// used in combination with Text to set a min length
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextMinLength(pub usize);

impl Averageable for TextMinLength {
    type Sum = usize;

    fn to_sum_component(&self) -> Self::Sum {
        self.0 as Self::Sum
    }

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {
        Self(sum / len as Self::Sum)
    }
}
    /// used in combination with Text to require a regex expr
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextPattern(pub String);

/// A marker for this to be a text selection used with:
///   Text: the currently selected option
///   TextList: the options available
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextSelect;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FanOscillation {
    Off,
    Vertical,
    Horizontal,
    Both,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FanDirection {
    Forward,
    Reverse,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FanSpeed {
    Auto,
    Percent(f64),
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LockState {
    Unknown,
    Locked,
    Unlocked,
    Jammed,
    Locking,
    Unlocking,
    Custom(String),
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MediaState {
    Unknown,
    Idle,
    Playing,
    Paused,
    Custom(String),
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CoverState {
    Opening,
    Closing,
    Stopped,
    Open,
    Closed,
    Custom(String),
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum BinarySensorType {
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AlarmState {
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
#[repr(u16)]
pub enum ComponentType {
	Int,
	Uint,
	Long,
	Float,
	Bool,
	Text,
	Object,
	List,
	IntList,
	UintList,
	LongList,
	FloatList,
	TextList,
	Date,
	Time,
	DateTime,
	Duration,
	Switch,
	Dimmer,
	Color,
	Unit,
	FloatMin,
	FloatMax,
	FloatStep,
	IntMin,
	IntMax,
	IntStep,
	LongMin,
	LongMax,
	LongStep,
	UintMin,
	UintMax,
	UintStep,
	Light,
	LightBulb,
	ColorTemperature,
	TextMaxLength,
	TextMinLength,
	TextPattern,
	TextSelect,
	FanOscillation,
	FanDirection,
	FanSpeed,
	ClimateMode,
	LockState,
	MediaState,
	CoverState,
	BinarySensorType,
	AlarmState,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Component {
	Int(Int),
	Uint(Uint),
	Long(Long),
	Float(Float),
	Bool(Bool),
	Text(Text),
	Object(Object),
	List(List),
	IntList(IntList),
	UintList(UintList),
	LongList(LongList),
	FloatList(FloatList),
	TextList(TextList),
	Date(Date),
	Time(Time),
	DateTime(DateTime),
	Duration(Duration),
	Switch(Switch),
	Dimmer(Dimmer),
	Color(Color),
	Unit(Unit),
	FloatMin(FloatMin),
	FloatMax(FloatMax),
	FloatStep(FloatStep),
	IntMin(IntMin),
	IntMax(IntMax),
	IntStep(IntStep),
	LongMin(LongMin),
	LongMax(LongMax),
	LongStep(LongStep),
	UintMin(UintMin),
	UintMax(UintMax),
	UintStep(UintStep),
	Light(Light),
	LightBulb(LightBulb),
	ColorTemperature(ColorTemperature),
	TextMaxLength(TextMaxLength),
	TextMinLength(TextMinLength),
	TextPattern(TextPattern),
	TextSelect(TextSelect),
	FanOscillation(FanOscillation),
	FanDirection(FanDirection),
	FanSpeed(FanSpeed),
	ClimateMode(ClimateMode),
	LockState(LockState),
	MediaState(MediaState),
	CoverState(CoverState),
	BinarySensorType(BinarySensorType),
	AlarmState(AlarmState),
}

impl Component {
	pub fn get_type(&self) -> ComponentType {
		match self {
			Component::Int(_) => ComponentType::Int,
			Component::Uint(_) => ComponentType::Uint,
			Component::Long(_) => ComponentType::Long,
			Component::Float(_) => ComponentType::Float,
			Component::Bool(_) => ComponentType::Bool,
			Component::Text(_) => ComponentType::Text,
			Component::Object(_) => ComponentType::Object,
			Component::List(_) => ComponentType::List,
			Component::IntList(_) => ComponentType::IntList,
			Component::UintList(_) => ComponentType::UintList,
			Component::LongList(_) => ComponentType::LongList,
			Component::FloatList(_) => ComponentType::FloatList,
			Component::TextList(_) => ComponentType::TextList,
			Component::Date(_) => ComponentType::Date,
			Component::Time(_) => ComponentType::Time,
			Component::DateTime(_) => ComponentType::DateTime,
			Component::Duration(_) => ComponentType::Duration,
			Component::Switch(_) => ComponentType::Switch,
			Component::Dimmer(_) => ComponentType::Dimmer,
			Component::Color(_) => ComponentType::Color,
			Component::Unit(_) => ComponentType::Unit,
			Component::FloatMin(_) => ComponentType::FloatMin,
			Component::FloatMax(_) => ComponentType::FloatMax,
			Component::FloatStep(_) => ComponentType::FloatStep,
			Component::IntMin(_) => ComponentType::IntMin,
			Component::IntMax(_) => ComponentType::IntMax,
			Component::IntStep(_) => ComponentType::IntStep,
			Component::LongMin(_) => ComponentType::LongMin,
			Component::LongMax(_) => ComponentType::LongMax,
			Component::LongStep(_) => ComponentType::LongStep,
			Component::UintMin(_) => ComponentType::UintMin,
			Component::UintMax(_) => ComponentType::UintMax,
			Component::UintStep(_) => ComponentType::UintStep,
			Component::Light(_) => ComponentType::Light,
			Component::LightBulb(_) => ComponentType::LightBulb,
			Component::ColorTemperature(_) => ComponentType::ColorTemperature,
			Component::TextMaxLength(_) => ComponentType::TextMaxLength,
			Component::TextMinLength(_) => ComponentType::TextMinLength,
			Component::TextPattern(_) => ComponentType::TextPattern,
			Component::TextSelect(_) => ComponentType::TextSelect,
			Component::FanOscillation(_) => ComponentType::FanOscillation,
			Component::FanDirection(_) => ComponentType::FanDirection,
			Component::FanSpeed(_) => ComponentType::FanSpeed,
			Component::ClimateMode(_) => ComponentType::ClimateMode,
			Component::LockState(_) => ComponentType::LockState,
			Component::MediaState(_) => ComponentType::MediaState,
			Component::CoverState(_) => ComponentType::CoverState,
			Component::BinarySensorType(_) => ComponentType::BinarySensorType,
			Component::AlarmState(_) => ComponentType::AlarmState,
		}
	}
}

impl Entity {

    pub fn int(&self) -> Option<&Int> {
        match self.0.get(&ComponentType::Int) {
            Some(Component::Int(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn int_mut(&mut self) -> Option<&mut Int> {
        match self.0.get_mut(&ComponentType::Int) {
            Some(Component::Int(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_int(&mut self, val: Int) {
        self.0.insert(ComponentType::Int, Component::Int(val));
    }
            
    pub fn has_int(&self) -> bool {
        self.int().is_some()
    }
            

    pub fn uint(&self) -> Option<&Uint> {
        match self.0.get(&ComponentType::Uint) {
            Some(Component::Uint(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn uint_mut(&mut self) -> Option<&mut Uint> {
        match self.0.get_mut(&ComponentType::Uint) {
            Some(Component::Uint(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_uint(&mut self, val: Uint) {
        self.0.insert(ComponentType::Uint, Component::Uint(val));
    }
            
    pub fn has_uint(&self) -> bool {
        self.uint().is_some()
    }
            

    pub fn long(&self) -> Option<&Long> {
        match self.0.get(&ComponentType::Long) {
            Some(Component::Long(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn long_mut(&mut self) -> Option<&mut Long> {
        match self.0.get_mut(&ComponentType::Long) {
            Some(Component::Long(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_long(&mut self, val: Long) {
        self.0.insert(ComponentType::Long, Component::Long(val));
    }
            
    pub fn has_long(&self) -> bool {
        self.long().is_some()
    }
            

    pub fn float(&self) -> Option<&Float> {
        match self.0.get(&ComponentType::Float) {
            Some(Component::Float(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn float_mut(&mut self) -> Option<&mut Float> {
        match self.0.get_mut(&ComponentType::Float) {
            Some(Component::Float(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_float(&mut self, val: Float) {
        self.0.insert(ComponentType::Float, Component::Float(val));
    }
            
    pub fn has_float(&self) -> bool {
        self.float().is_some()
    }
            

    pub fn bool(&self) -> Option<&Bool> {
        match self.0.get(&ComponentType::Bool) {
            Some(Component::Bool(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn bool_mut(&mut self) -> Option<&mut Bool> {
        match self.0.get_mut(&ComponentType::Bool) {
            Some(Component::Bool(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_bool(&mut self, val: Bool) {
        self.0.insert(ComponentType::Bool, Component::Bool(val));
    }
            
    pub fn has_bool(&self) -> bool {
        self.bool().is_some()
    }
            

    pub fn text(&self) -> Option<&Text> {
        match self.0.get(&ComponentType::Text) {
            Some(Component::Text(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn text_mut(&mut self) -> Option<&mut Text> {
        match self.0.get_mut(&ComponentType::Text) {
            Some(Component::Text(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_text(&mut self, val: Text) {
        self.0.insert(ComponentType::Text, Component::Text(val));
    }
            
    pub fn has_text(&self) -> bool {
        self.text().is_some()
    }
            

    pub fn object(&self) -> Option<&Object> {
        match self.0.get(&ComponentType::Object) {
            Some(Component::Object(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn object_mut(&mut self) -> Option<&mut Object> {
        match self.0.get_mut(&ComponentType::Object) {
            Some(Component::Object(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_object(&mut self, val: Object) {
        self.0.insert(ComponentType::Object, Component::Object(val));
    }
            
    pub fn has_object(&self) -> bool {
        self.object().is_some()
    }
            

    pub fn list(&self) -> Option<&List> {
        match self.0.get(&ComponentType::List) {
            Some(Component::List(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn list_mut(&mut self) -> Option<&mut List> {
        match self.0.get_mut(&ComponentType::List) {
            Some(Component::List(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_list(&mut self, val: List) {
        self.0.insert(ComponentType::List, Component::List(val));
    }
            
    pub fn has_list(&self) -> bool {
        self.list().is_some()
    }
            

    pub fn int_list(&self) -> Option<&IntList> {
        match self.0.get(&ComponentType::IntList) {
            Some(Component::IntList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn int_list_mut(&mut self) -> Option<&mut IntList> {
        match self.0.get_mut(&ComponentType::IntList) {
            Some(Component::IntList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_int_list(&mut self, val: IntList) {
        self.0.insert(ComponentType::IntList, Component::IntList(val));
    }
            
    pub fn has_int_list(&self) -> bool {
        self.int_list().is_some()
    }
            

    pub fn uint_list(&self) -> Option<&UintList> {
        match self.0.get(&ComponentType::UintList) {
            Some(Component::UintList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn uint_list_mut(&mut self) -> Option<&mut UintList> {
        match self.0.get_mut(&ComponentType::UintList) {
            Some(Component::UintList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_uint_list(&mut self, val: UintList) {
        self.0.insert(ComponentType::UintList, Component::UintList(val));
    }
            
    pub fn has_uint_list(&self) -> bool {
        self.uint_list().is_some()
    }
            

    pub fn long_list(&self) -> Option<&LongList> {
        match self.0.get(&ComponentType::LongList) {
            Some(Component::LongList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn long_list_mut(&mut self) -> Option<&mut LongList> {
        match self.0.get_mut(&ComponentType::LongList) {
            Some(Component::LongList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_long_list(&mut self, val: LongList) {
        self.0.insert(ComponentType::LongList, Component::LongList(val));
    }
            
    pub fn has_long_list(&self) -> bool {
        self.long_list().is_some()
    }
            

    pub fn float_list(&self) -> Option<&FloatList> {
        match self.0.get(&ComponentType::FloatList) {
            Some(Component::FloatList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn float_list_mut(&mut self) -> Option<&mut FloatList> {
        match self.0.get_mut(&ComponentType::FloatList) {
            Some(Component::FloatList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_float_list(&mut self, val: FloatList) {
        self.0.insert(ComponentType::FloatList, Component::FloatList(val));
    }
            
    pub fn has_float_list(&self) -> bool {
        self.float_list().is_some()
    }
            

    pub fn text_list(&self) -> Option<&TextList> {
        match self.0.get(&ComponentType::TextList) {
            Some(Component::TextList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn text_list_mut(&mut self) -> Option<&mut TextList> {
        match self.0.get_mut(&ComponentType::TextList) {
            Some(Component::TextList(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_text_list(&mut self, val: TextList) {
        self.0.insert(ComponentType::TextList, Component::TextList(val));
    }
            
    pub fn has_text_list(&self) -> bool {
        self.text_list().is_some()
    }
            

    pub fn date(&self) -> Option<&Date> {
        match self.0.get(&ComponentType::Date) {
            Some(Component::Date(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn date_mut(&mut self) -> Option<&mut Date> {
        match self.0.get_mut(&ComponentType::Date) {
            Some(Component::Date(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_date(&mut self, val: Date) {
        self.0.insert(ComponentType::Date, Component::Date(val));
    }
            
    pub fn has_date(&self) -> bool {
        self.date().is_some()
    }
            

    pub fn time(&self) -> Option<&Time> {
        match self.0.get(&ComponentType::Time) {
            Some(Component::Time(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn time_mut(&mut self) -> Option<&mut Time> {
        match self.0.get_mut(&ComponentType::Time) {
            Some(Component::Time(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_time(&mut self, val: Time) {
        self.0.insert(ComponentType::Time, Component::Time(val));
    }
            
    pub fn has_time(&self) -> bool {
        self.time().is_some()
    }
            

    pub fn date_time(&self) -> Option<&DateTime> {
        match self.0.get(&ComponentType::DateTime) {
            Some(Component::DateTime(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn date_time_mut(&mut self) -> Option<&mut DateTime> {
        match self.0.get_mut(&ComponentType::DateTime) {
            Some(Component::DateTime(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_date_time(&mut self, val: DateTime) {
        self.0.insert(ComponentType::DateTime, Component::DateTime(val));
    }
            
    pub fn has_date_time(&self) -> bool {
        self.date_time().is_some()
    }
            

    pub fn duration(&self) -> Option<&Duration> {
        match self.0.get(&ComponentType::Duration) {
            Some(Component::Duration(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn duration_mut(&mut self) -> Option<&mut Duration> {
        match self.0.get_mut(&ComponentType::Duration) {
            Some(Component::Duration(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_duration(&mut self, val: Duration) {
        self.0.insert(ComponentType::Duration, Component::Duration(val));
    }
            
    pub fn has_duration(&self) -> bool {
        self.duration().is_some()
    }
            

    pub fn switch(&self) -> Option<&Switch> {
        match self.0.get(&ComponentType::Switch) {
            Some(Component::Switch(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn switch_mut(&mut self) -> Option<&mut Switch> {
        match self.0.get_mut(&ComponentType::Switch) {
            Some(Component::Switch(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_switch(&mut self, val: Switch) {
        self.0.insert(ComponentType::Switch, Component::Switch(val));
    }
            
    pub fn has_switch(&self) -> bool {
        self.switch().is_some()
    }
            

    pub fn dimmer(&self) -> Option<&Dimmer> {
        match self.0.get(&ComponentType::Dimmer) {
            Some(Component::Dimmer(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn dimmer_mut(&mut self) -> Option<&mut Dimmer> {
        match self.0.get_mut(&ComponentType::Dimmer) {
            Some(Component::Dimmer(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_dimmer(&mut self, val: Dimmer) {
        self.0.insert(ComponentType::Dimmer, Component::Dimmer(val));
    }
            
    pub fn has_dimmer(&self) -> bool {
        self.dimmer().is_some()
    }
            

    pub fn color(&self) -> Option<&Color> {
        match self.0.get(&ComponentType::Color) {
            Some(Component::Color(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn color_mut(&mut self) -> Option<&mut Color> {
        match self.0.get_mut(&ComponentType::Color) {
            Some(Component::Color(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_color(&mut self, val: Color) {
        self.0.insert(ComponentType::Color, Component::Color(val));
    }
            
    pub fn has_color(&self) -> bool {
        self.color().is_some()
    }
            

    pub fn unit(&self) -> Option<&Unit> {
        match self.0.get(&ComponentType::Unit) {
            Some(Component::Unit(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn unit_mut(&mut self) -> Option<&mut Unit> {
        match self.0.get_mut(&ComponentType::Unit) {
            Some(Component::Unit(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_unit(&mut self, val: Unit) {
        self.0.insert(ComponentType::Unit, Component::Unit(val));
    }
            
    pub fn has_unit(&self) -> bool {
        self.unit().is_some()
    }
            

    pub fn float_min(&self) -> Option<&FloatMin> {
        match self.0.get(&ComponentType::FloatMin) {
            Some(Component::FloatMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn float_min_mut(&mut self) -> Option<&mut FloatMin> {
        match self.0.get_mut(&ComponentType::FloatMin) {
            Some(Component::FloatMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_float_min(&mut self, val: FloatMin) {
        self.0.insert(ComponentType::FloatMin, Component::FloatMin(val));
    }
            
    pub fn has_float_min(&self) -> bool {
        self.float_min().is_some()
    }
            

    pub fn float_max(&self) -> Option<&FloatMax> {
        match self.0.get(&ComponentType::FloatMax) {
            Some(Component::FloatMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn float_max_mut(&mut self) -> Option<&mut FloatMax> {
        match self.0.get_mut(&ComponentType::FloatMax) {
            Some(Component::FloatMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_float_max(&mut self, val: FloatMax) {
        self.0.insert(ComponentType::FloatMax, Component::FloatMax(val));
    }
            
    pub fn has_float_max(&self) -> bool {
        self.float_max().is_some()
    }
            

    pub fn float_step(&self) -> Option<&FloatStep> {
        match self.0.get(&ComponentType::FloatStep) {
            Some(Component::FloatStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn float_step_mut(&mut self) -> Option<&mut FloatStep> {
        match self.0.get_mut(&ComponentType::FloatStep) {
            Some(Component::FloatStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_float_step(&mut self, val: FloatStep) {
        self.0.insert(ComponentType::FloatStep, Component::FloatStep(val));
    }
            
    pub fn has_float_step(&self) -> bool {
        self.float_step().is_some()
    }
            

    pub fn int_min(&self) -> Option<&IntMin> {
        match self.0.get(&ComponentType::IntMin) {
            Some(Component::IntMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn int_min_mut(&mut self) -> Option<&mut IntMin> {
        match self.0.get_mut(&ComponentType::IntMin) {
            Some(Component::IntMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_int_min(&mut self, val: IntMin) {
        self.0.insert(ComponentType::IntMin, Component::IntMin(val));
    }
            
    pub fn has_int_min(&self) -> bool {
        self.int_min().is_some()
    }
            

    pub fn int_max(&self) -> Option<&IntMax> {
        match self.0.get(&ComponentType::IntMax) {
            Some(Component::IntMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn int_max_mut(&mut self) -> Option<&mut IntMax> {
        match self.0.get_mut(&ComponentType::IntMax) {
            Some(Component::IntMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_int_max(&mut self, val: IntMax) {
        self.0.insert(ComponentType::IntMax, Component::IntMax(val));
    }
            
    pub fn has_int_max(&self) -> bool {
        self.int_max().is_some()
    }
            

    pub fn int_step(&self) -> Option<&IntStep> {
        match self.0.get(&ComponentType::IntStep) {
            Some(Component::IntStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn int_step_mut(&mut self) -> Option<&mut IntStep> {
        match self.0.get_mut(&ComponentType::IntStep) {
            Some(Component::IntStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_int_step(&mut self, val: IntStep) {
        self.0.insert(ComponentType::IntStep, Component::IntStep(val));
    }
            
    pub fn has_int_step(&self) -> bool {
        self.int_step().is_some()
    }
            

    pub fn long_min(&self) -> Option<&LongMin> {
        match self.0.get(&ComponentType::LongMin) {
            Some(Component::LongMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn long_min_mut(&mut self) -> Option<&mut LongMin> {
        match self.0.get_mut(&ComponentType::LongMin) {
            Some(Component::LongMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_long_min(&mut self, val: LongMin) {
        self.0.insert(ComponentType::LongMin, Component::LongMin(val));
    }
            
    pub fn has_long_min(&self) -> bool {
        self.long_min().is_some()
    }
            

    pub fn long_max(&self) -> Option<&LongMax> {
        match self.0.get(&ComponentType::LongMax) {
            Some(Component::LongMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn long_max_mut(&mut self) -> Option<&mut LongMax> {
        match self.0.get_mut(&ComponentType::LongMax) {
            Some(Component::LongMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_long_max(&mut self, val: LongMax) {
        self.0.insert(ComponentType::LongMax, Component::LongMax(val));
    }
            
    pub fn has_long_max(&self) -> bool {
        self.long_max().is_some()
    }
            

    pub fn long_step(&self) -> Option<&LongStep> {
        match self.0.get(&ComponentType::LongStep) {
            Some(Component::LongStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn long_step_mut(&mut self) -> Option<&mut LongStep> {
        match self.0.get_mut(&ComponentType::LongStep) {
            Some(Component::LongStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_long_step(&mut self, val: LongStep) {
        self.0.insert(ComponentType::LongStep, Component::LongStep(val));
    }
            
    pub fn has_long_step(&self) -> bool {
        self.long_step().is_some()
    }
            

    pub fn uint_min(&self) -> Option<&UintMin> {
        match self.0.get(&ComponentType::UintMin) {
            Some(Component::UintMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn uint_min_mut(&mut self) -> Option<&mut UintMin> {
        match self.0.get_mut(&ComponentType::UintMin) {
            Some(Component::UintMin(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_uint_min(&mut self, val: UintMin) {
        self.0.insert(ComponentType::UintMin, Component::UintMin(val));
    }
            
    pub fn has_uint_min(&self) -> bool {
        self.uint_min().is_some()
    }
            

    pub fn uint_max(&self) -> Option<&UintMax> {
        match self.0.get(&ComponentType::UintMax) {
            Some(Component::UintMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn uint_max_mut(&mut self) -> Option<&mut UintMax> {
        match self.0.get_mut(&ComponentType::UintMax) {
            Some(Component::UintMax(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_uint_max(&mut self, val: UintMax) {
        self.0.insert(ComponentType::UintMax, Component::UintMax(val));
    }
            
    pub fn has_uint_max(&self) -> bool {
        self.uint_max().is_some()
    }
            

    pub fn uint_step(&self) -> Option<&UintStep> {
        match self.0.get(&ComponentType::UintStep) {
            Some(Component::UintStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn uint_step_mut(&mut self) -> Option<&mut UintStep> {
        match self.0.get_mut(&ComponentType::UintStep) {
            Some(Component::UintStep(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_uint_step(&mut self, val: UintStep) {
        self.0.insert(ComponentType::UintStep, Component::UintStep(val));
    }
            
    pub fn has_uint_step(&self) -> bool {
        self.uint_step().is_some()
    }
            

    pub fn light(&self) -> Option<&Light> {
        match self.0.get(&ComponentType::Light) {
            Some(Component::Light(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn light_mut(&mut self) -> Option<&mut Light> {
        match self.0.get_mut(&ComponentType::Light) {
            Some(Component::Light(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_light(&mut self, val: Light) {
        self.0.insert(ComponentType::Light, Component::Light(val));
    }
            
    pub fn has_light(&self) -> bool {
        self.light().is_some()
    }
            

    pub fn light_bulb(&self) -> Option<&LightBulb> {
        match self.0.get(&ComponentType::LightBulb) {
            Some(Component::LightBulb(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn light_bulb_mut(&mut self) -> Option<&mut LightBulb> {
        match self.0.get_mut(&ComponentType::LightBulb) {
            Some(Component::LightBulb(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_light_bulb(&mut self, val: LightBulb) {
        self.0.insert(ComponentType::LightBulb, Component::LightBulb(val));
    }
            
    pub fn has_light_bulb(&self) -> bool {
        self.light_bulb().is_some()
    }
            

    pub fn color_temperature(&self) -> Option<&ColorTemperature> {
        match self.0.get(&ComponentType::ColorTemperature) {
            Some(Component::ColorTemperature(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn color_temperature_mut(&mut self) -> Option<&mut ColorTemperature> {
        match self.0.get_mut(&ComponentType::ColorTemperature) {
            Some(Component::ColorTemperature(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_color_temperature(&mut self, val: ColorTemperature) {
        self.0.insert(ComponentType::ColorTemperature, Component::ColorTemperature(val));
    }
            
    pub fn has_color_temperature(&self) -> bool {
        self.color_temperature().is_some()
    }
            

    pub fn text_max_length(&self) -> Option<&TextMaxLength> {
        match self.0.get(&ComponentType::TextMaxLength) {
            Some(Component::TextMaxLength(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn text_max_length_mut(&mut self) -> Option<&mut TextMaxLength> {
        match self.0.get_mut(&ComponentType::TextMaxLength) {
            Some(Component::TextMaxLength(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_text_max_length(&mut self, val: TextMaxLength) {
        self.0.insert(ComponentType::TextMaxLength, Component::TextMaxLength(val));
    }
            
    pub fn has_text_max_length(&self) -> bool {
        self.text_max_length().is_some()
    }
            

    pub fn text_min_length(&self) -> Option<&TextMinLength> {
        match self.0.get(&ComponentType::TextMinLength) {
            Some(Component::TextMinLength(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn text_min_length_mut(&mut self) -> Option<&mut TextMinLength> {
        match self.0.get_mut(&ComponentType::TextMinLength) {
            Some(Component::TextMinLength(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_text_min_length(&mut self, val: TextMinLength) {
        self.0.insert(ComponentType::TextMinLength, Component::TextMinLength(val));
    }
            
    pub fn has_text_min_length(&self) -> bool {
        self.text_min_length().is_some()
    }
            

    pub fn text_pattern(&self) -> Option<&TextPattern> {
        match self.0.get(&ComponentType::TextPattern) {
            Some(Component::TextPattern(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn text_pattern_mut(&mut self) -> Option<&mut TextPattern> {
        match self.0.get_mut(&ComponentType::TextPattern) {
            Some(Component::TextPattern(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_text_pattern(&mut self, val: TextPattern) {
        self.0.insert(ComponentType::TextPattern, Component::TextPattern(val));
    }
            
    pub fn has_text_pattern(&self) -> bool {
        self.text_pattern().is_some()
    }
            

    pub fn text_select(&self) -> Option<&TextSelect> {
        match self.0.get(&ComponentType::TextSelect) {
            Some(Component::TextSelect(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn text_select_mut(&mut self) -> Option<&mut TextSelect> {
        match self.0.get_mut(&ComponentType::TextSelect) {
            Some(Component::TextSelect(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_text_select(&mut self, val: TextSelect) {
        self.0.insert(ComponentType::TextSelect, Component::TextSelect(val));
    }
            
    pub fn has_text_select(&self) -> bool {
        self.text_select().is_some()
    }
            

    pub fn fan_oscillation(&self) -> Option<&FanOscillation> {
        match self.0.get(&ComponentType::FanOscillation) {
            Some(Component::FanOscillation(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn fan_oscillation_mut(&mut self) -> Option<&mut FanOscillation> {
        match self.0.get_mut(&ComponentType::FanOscillation) {
            Some(Component::FanOscillation(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_fan_oscillation(&mut self, val: FanOscillation) {
        self.0.insert(ComponentType::FanOscillation, Component::FanOscillation(val));
    }
            
    pub fn has_fan_oscillation(&self) -> bool {
        self.fan_oscillation().is_some()
    }
            

    pub fn fan_direction(&self) -> Option<&FanDirection> {
        match self.0.get(&ComponentType::FanDirection) {
            Some(Component::FanDirection(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn fan_direction_mut(&mut self) -> Option<&mut FanDirection> {
        match self.0.get_mut(&ComponentType::FanDirection) {
            Some(Component::FanDirection(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_fan_direction(&mut self, val: FanDirection) {
        self.0.insert(ComponentType::FanDirection, Component::FanDirection(val));
    }
            
    pub fn has_fan_direction(&self) -> bool {
        self.fan_direction().is_some()
    }
            

    pub fn fan_speed(&self) -> Option<&FanSpeed> {
        match self.0.get(&ComponentType::FanSpeed) {
            Some(Component::FanSpeed(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn fan_speed_mut(&mut self) -> Option<&mut FanSpeed> {
        match self.0.get_mut(&ComponentType::FanSpeed) {
            Some(Component::FanSpeed(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_fan_speed(&mut self, val: FanSpeed) {
        self.0.insert(ComponentType::FanSpeed, Component::FanSpeed(val));
    }
            
    pub fn has_fan_speed(&self) -> bool {
        self.fan_speed().is_some()
    }
            

    pub fn climate_mode(&self) -> Option<&ClimateMode> {
        match self.0.get(&ComponentType::ClimateMode) {
            Some(Component::ClimateMode(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn climate_mode_mut(&mut self) -> Option<&mut ClimateMode> {
        match self.0.get_mut(&ComponentType::ClimateMode) {
            Some(Component::ClimateMode(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_climate_mode(&mut self, val: ClimateMode) {
        self.0.insert(ComponentType::ClimateMode, Component::ClimateMode(val));
    }
            
    pub fn has_climate_mode(&self) -> bool {
        self.climate_mode().is_some()
    }
            

    pub fn lock_state(&self) -> Option<&LockState> {
        match self.0.get(&ComponentType::LockState) {
            Some(Component::LockState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn lock_state_mut(&mut self) -> Option<&mut LockState> {
        match self.0.get_mut(&ComponentType::LockState) {
            Some(Component::LockState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_lock_state(&mut self, val: LockState) {
        self.0.insert(ComponentType::LockState, Component::LockState(val));
    }
            
    pub fn has_lock_state(&self) -> bool {
        self.lock_state().is_some()
    }
            

    pub fn media_state(&self) -> Option<&MediaState> {
        match self.0.get(&ComponentType::MediaState) {
            Some(Component::MediaState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn media_state_mut(&mut self) -> Option<&mut MediaState> {
        match self.0.get_mut(&ComponentType::MediaState) {
            Some(Component::MediaState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_media_state(&mut self, val: MediaState) {
        self.0.insert(ComponentType::MediaState, Component::MediaState(val));
    }
            
    pub fn has_media_state(&self) -> bool {
        self.media_state().is_some()
    }
            

    pub fn cover_state(&self) -> Option<&CoverState> {
        match self.0.get(&ComponentType::CoverState) {
            Some(Component::CoverState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn cover_state_mut(&mut self) -> Option<&mut CoverState> {
        match self.0.get_mut(&ComponentType::CoverState) {
            Some(Component::CoverState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_cover_state(&mut self, val: CoverState) {
        self.0.insert(ComponentType::CoverState, Component::CoverState(val));
    }
            
    pub fn has_cover_state(&self) -> bool {
        self.cover_state().is_some()
    }
            

    pub fn binary_sensor_type(&self) -> Option<&BinarySensorType> {
        match self.0.get(&ComponentType::BinarySensorType) {
            Some(Component::BinarySensorType(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn binary_sensor_type_mut(&mut self) -> Option<&mut BinarySensorType> {
        match self.0.get_mut(&ComponentType::BinarySensorType) {
            Some(Component::BinarySensorType(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_binary_sensor_type(&mut self, val: BinarySensorType) {
        self.0.insert(ComponentType::BinarySensorType, Component::BinarySensorType(val));
    }
            
    pub fn has_binary_sensor_type(&self) -> bool {
        self.binary_sensor_type().is_some()
    }
            

    pub fn alarm_state(&self) -> Option<&AlarmState> {
        match self.0.get(&ComponentType::AlarmState) {
            Some(Component::AlarmState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn alarm_state_mut(&mut self) -> Option<&mut AlarmState> {
        match self.0.get_mut(&ComponentType::AlarmState) {
            Some(Component::AlarmState(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }
            
    pub fn set_alarm_state(&mut self, val: AlarmState) {
        self.0.insert(ComponentType::AlarmState, Component::AlarmState(val));
    }
            
    pub fn has_alarm_state(&self) -> bool {
        self.alarm_state().is_some()
    }
            }
