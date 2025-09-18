// use std::collections::HashMap;

// use crate::Color;
// use crate::CompositeType;
// use crate::PrimitiveType;
// use crate::Type;

// pub fn test() {
//     let mut obj = HashMap::new();
//     obj.insert("on", Type::Primitive(PrimitiveType::Bool));
//     obj.insert(
//         "brightness",
//         Type::Composite(Box::new(CompositeType::Optional(Type::Primitive(
//             PrimitiveType::Float,
//         )))),
//     );
//     let light = Type::Composite(Box::new(CompositeType::Object(obj)));
// }

// pub const Light: Type = Type::Composite(Box::new());

// pub enum CustomType {
//     Light,
//     FloatSensor,
//     IntSensor,
//     FanDirection,
//     FanOscillation,
//     FanSpeed,
//     ClimateMode,
//     Lock,
//     MediaPlayer,
// }

// pub enum CustomValue {
//     Light(LightValue),
//     FloatSensor(SensorValue<f64>),
//     IntSensor(SensorValue<i32>),
//     FanDirection(FanDirection),
//     FanOscillation(FanOscillation),
//     FanSpeed(FanSpeed),
//     ClimateMode(ClimateMode),
//     Lock(Lock),
//     MediaPlayer(MediaPlayer),
// }

// pub struct LightValue {
//     pub on: bool,
//     pub brightness: Option<f64>,
//     pub color_temp: Option<u16>,
//     pub color: Color,
// }

// pub struct SensorValue<T> {
//     pub icon: Option<String>,
//     pub unit: String,
//     pub value: T,
// }

// pub enum FanDirection {
//     Forward,
//     Reverse,
// }

// pub enum FanOscillation {
//     Off,
//     Vertical,
//     Horizontal,
//     Both,
// }

// pub enum FanSpeed {
//     Auto,
//     Percent(f64),
// }

// pub enum ClimateMode {
//     Off,
//     Auto,
//     Heat,
//     Cool,
//     HeatCool,
//     Fan,
//     Dry,
// }

// pub enum Lock {
//     None,
//     Locked,
//     Unlocked,
//     Jammed,
//     Locking,
//     Unlocking,
// }

// pub enum MediaPlayer {
//     None,
//     Idle,
//     Playing,
//     Paused,

//     // commands here?
//     Play,
//     Pause,
//     Stop,
//     Mute,
//     Unmute,
// }
