use derive_more::Display;
use igloo_interface::{Component, ComponentType};
use rustc_hash::FxHashMap;

use crate::glacier::query::{QueryFilter, QueryTarget};

// TODO we need to experiment with different systems
// for sizing, margins, and padding. For now we will
// leave that off

pub struct Dashboard {
    pub name: String,

    /// used for custom queries in this
    /// dashboard, not defined inside
    /// CustomElements
    pub targets: FxHashMap<String, QueryTarget>,

    pub child: Element,
}

/// Custom element, defined in Ron
/// To aid users easily making composable
/// Dashboards
pub struct CustomElement {
    pub(super) name: String,

    /// When placing this element, user will
    /// have to select a QueryTarget:: for each
    /// of these
    /// In your query bindings below, you can use
    /// these query_targets by name
    pub(super) targets: Vec<String>,

    pub(super) children: Vec<Element>,
}

/// target, filter, component
/// target links to .query_targets
pub type DashQuery = (String, QueryFilter, ComponentType);

/// target, filter
/// target links to .query_targets
pub type DashQueryNoType = (String, QueryFilter);

pub enum Element {
    Custom {
        name: String,
        selected_targets: FxHashMap<String, QueryTarget>,
    },
    If {
        condition: Expr,
        then: Vec<Element>,
        r#else: Vec<Element>,
    },
    Repeat {
        count: Expr,
        each: Vec<Element>,
    },
    ForEach {},
    HStack {
        justify: HAlign,
        align: VAlign, // TODO this right?
        scroll: bool,
        children: Vec<Element>,
    },
    VStack {
        justify: VAlign,
        align: HAlign,
        scroll: bool,
        children: Vec<Element>,
    },
    Tabs {
        pages: FxHashMap<String, Vec<Element>>,
    },
    /// make a badge with `Card(HStack { children: [..] })`
    Card {
        child: Box<Element>,
    },
    Switch {
        /// ComponentType must have a bool (ex ::Bool, ::Switch)
        /// Will register a ::WatchAvg query
        /// When interacted, calls a ::Set query
        binding: DashQuery,
        size: Size,
        // TODO variant?
    },
    Checkbox {
        /// ComponentType must have a bool (ex ::Bool, ::Switch)
        /// Will register a ::WatchAvg query
        /// When interacted, calls a ::Set query
        binding: DashQuery,
        size: Size,
        // TODO variant?
    },
    ToggleButton {
        /// ComponentType must have a bool (ex ::Bool, ::Switch)
        /// Will register a ::WatchAvg query
        /// When interacted, calls a ::Set query
        binding: DashQuery,
        size: Size,
        // TODO variant?
    },
    Icon {
        icon: String,
        /// instead of getting icon from `name`
        /// query for Component::Icon
        icon_value: Option<DashQueryNoType>,
        size: Size,
    },
    Button {
        /// calls a ::Set query with ComponentType::Trigger
        // TODO should also be able to run Penguin script
        // Or maybe call custom query with specific value?
        // And definetely be able to navigate to other Dashboards
        on_click: Option<DashQueryNoType>,
        size: Size,
        variant: ButtonVariant,
        children: Vec<Element>,
    },
    Text {
        value: Option<DashQuery>,
        prefix: String,
        suffix: String,
        size: Size,
    },
    TextInput {
        title: String,
        placeholder: String,
        /// ComponentType must have a string (ex ::Text)
        binding: DashQuery,
        /// Disables \*MaxLength, \*MinLength, \*Pattern enforcement
        disable_validation: bool,
        is_password: bool,
        multi_line: bool,
    },
    NumberInput {
        title: String,
        placeholder: String,
        /// ComponentType must have a number (ex ::Int, ::Float)
        binding: DashQuery,
        /// If only 1 component is queried, and it's entity
        /// also has bounds (\*Min, \*Max, \*Step) those will
        /// be enforced UNLESS this is set to true
        disable_validation: bool,
    },
    TimePicker {
        binding: DashQueryNoType,
    },
    DatePicker {
        binding: DashQueryNoType,
    },
    DateTimePicker {
        binding: DashQueryNoType,
    },
    DurationPicker {
        binding: DashQueryNoType,
    },
    WeekdayPicker {
        binding: DashQueryNoType,
        /// If multi uses WeekdayList
        /// Else Weekday
        multi: bool,
    },
    Slider {
        /// ComponentType must have a number (ex ::Int, ::Float)
        binding: DashQuery,
        /// If only 1 component is queried, and it's entity
        /// also has bounds (\*Min, \*Max, \*Step) those will
        /// be enforced UNLESS this is set to true
        disable_validation: bool,
        /// Override valiation params
        /// Must be Component ::Int or ::Float
        min: Option<Component>,
        max: Option<Component>,
        step: Option<Component>,
    },
    ColorTemperaturePicker {
        binding: DashQueryNoType,
        // for now its just a wide colored slider, but might
        // add variants
    },
    ColorPicker {
        binding: DashQueryNoType,
        variant: ColorPickerVariant,
    },
    TextSelect {
        // Finds entities marked TextSelect
        // Current value is Component::Text
        // Options are Component::TextList
        binding: DashQueryNoType,
        variant: SelectVariant,
    },
    ModeSelect {
        /// Component must have Supported type
        /// For example, you'd put FanOscillation here, the options
        /// will be taken from SupportedFanOscillations
        binding: DashQuery,
        variant: SelectVariant,
    },
    CustomSelect {
        binding: DashQuery,
        variant: SelectVariant,
        /// (option name, value)
        options: Vec<(String, Component)>,
    },
    /// filler for now
    Chart,
    /// filler for now
    Table,
    /// filler for now
    VideoFeed,
    /// filler for now
    /// should be able to link to internal pages (other dashboards)
    /// and external links
    Link,
    /// filler for now
    Image,
    /// filler for now
    Collapsable,
    Hr,
}

#[derive(Default)]
pub enum ColorPickerVariant {
    #[default]
    Circle,
    HueSlider,
    HSL,
}

#[derive(Default, Display)]
pub enum SelectVariant {
    Dropdown,
    #[default]
    Panel,
    Radio,
}

#[derive(Default, Display)]
pub enum ButtonVariant {
    #[default]
    #[display("normal")]
    Normal,
    #[display("primary")]
    Primary,
    #[display("secondary")]
    Secondary,
    #[display("warning")]
    Warning,
    #[display("error")]
    Error,
    // Note there are no outline and ghost variants
    // those are specific to the theme
    // For example, a theme might make all ::Normal
    // buttons outlined or something
}

#[derive(Default, Display)]
pub enum Size {
    #[display("xsmall")]
    XSmall,
    #[display("small")]
    Small,
    #[default]
    #[display("medium")]
    Medium,
    #[display("large")]
    Large,
    #[display("xlarge")]
    XLarge,
}

#[derive(Display)]
pub enum HAlign {
    #[display("flex-start")]
    Start,
    #[display("center")]
    Center,
    #[display("flex-end")]
    End,
    #[display("space-between")]
    SpaceBetween,
    #[display("space-around")]
    SpaceAround,
    #[display("space-evenly")]
    SpaceEvenly,
}

#[derive(Display)]
pub enum VAlign {
    #[display("flex-start")]
    Start,
    #[display("flex-center")]
    Center,
    #[display("flex-end")]
    End,
    #[display("stretch")]
    Stretch,
}

pub enum Primitive {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

pub enum Expr {
    Primitive(Primitive),
    Query(DashQuery),
    QueryNT(DashQueryNoType),
    Field(Box<Expr>, String),
    Index(Box<Expr>, usize),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    Pow,

    EqEq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,

    AndAnd,
    OrOr,
    Xor,
    And,
    Or,
}
