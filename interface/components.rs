use std::collections::HashMap;

struct Int(pub i32);

struct Float(pub f64);

struct Bool(pub bool);

struct Text(pub String);

struct Object(pub HashMap<String, ComponentValue>);

struct List(pub Vec<ComponentValue>);

struct Light;

struct Switch(pub bool);

struct Dimmer(pub u8);

struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

enum Unit {
    Seconds,
}
