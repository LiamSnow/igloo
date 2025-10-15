use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{Display, From};

use crate::{Component, ComponentType, QueryFilter, QueryTarget};

// TODO we need to experiment with different systems
// for sizing, margins, and padding. For now we will
// leave that off

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Dashboard {
    pub name: String,
    /// used for custom queries in this
    /// dashboard, not defined inside
    /// CustomElements
    pub targets: HashMap<String, QueryTarget>,
    pub child: DashElement,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, From, PartialEq)]
pub enum DashElement {
    Custom(CustomElement),
    If(IfElement),
    Repeat(RepeatElement),
    ForEach(ForEachElement),
    HStack(HStackElement),
    VStack(VStackElement),
    Tabs(TabsElement),
    Card(CardElement),
    Switch(SwitchElement),
    Checkbox(CheckboxElement),
    ToggleButton(ToggleButtonElement),
    Icon(IconElement),
    Button(ButtonElement),
    Text(TextElement),
    TextInput(TextInputElement),
    NumberInput(NumberInputElement),
    TimePicker(TimePickerElement),
    DatePicker(DatePickerElement),
    DateTimePicker(DateTimePickerElement),
    DurationPicker(DurationPickerElement),
    WeekdayPicker(WeekdayPickerElement),
    Slider(SliderElement),
    ColorTemperaturePicker(ColorTemperaturePickerElement),
    ColorPicker(ColorPickerElement),
    TextSelect(TextSelectElement),
    ModeSelect(ModeSelectElement),
    CustomSelect(CustomSelectElement),
    Chart(ChartElement),
    Table(TableElement),
    VideoFeed(VideoFeedElement),
    Link(LinkElement),
    Image(ImageElement),
    Collapsable(CollapsableElement),
    Hr,
}

/// Custom element, defined in Ron
/// To aid users easily making composable
/// Dashboards
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CustomElementDefn {
    pub(super) name: String,

    /// When placing this element, user will
    /// have to select a QueryTarget:: for each
    /// of these
    /// In your query bindings below, you can use
    /// these query_targets by name
    pub(super) targets: Vec<String>,

    pub(super) children: Vec<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DashQuery {
    pub target: String,
    pub filter: QueryFilter,
    pub comp_type: ComponentType,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DashQueryNoType {
    pub target: String,
    pub filter: QueryFilter,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CustomElement {
    pub name: String,
    pub selected_targets: HashMap<String, QueryTarget>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct IfElement {
    pub condition: Expr,
    pub then: Vec<DashElement>,
    pub r#else: Vec<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct RepeatElement {
    pub count: Expr,
    pub each: Vec<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ForEachElement {}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct HStackElement {
    pub justify: HAlign,
    pub align: VAlign, // TODO this right?
    pub scroll: bool,
    pub children: Vec<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct VStackElement {
    pub justify: VAlign,
    pub align: HAlign,
    pub scroll: bool,
    pub children: Vec<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TabsElement {
    pub pages: HashMap<String, Vec<DashElement>>,
}

/// make a badge with `Card(HStack { children: [..] })`
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CardElement {
    pub child: Box<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SwitchElement {
    /// ComponentType must have a bool (ex ::Bool, ::Switch)
    /// Will register a ::WatchAvg query
    /// When interacted, calls a ::Set query
    pub binding: DashQuery,
    pub size: Size,
    // TODO variant?
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CheckboxElement {
    /// ComponentType must have a bool (ex ::Bool, ::Switch)
    /// Will register a ::WatchAvg query
    /// When interacted, calls a ::Set query
    pub binding: DashQuery,
    pub size: Size,
    // TODO variant?
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ToggleButtonElement {
    /// ComponentType must have a bool (ex ::Bool, ::Switch)
    /// Will register a ::WatchAvg query
    /// When interacted, calls a ::Set query
    pub binding: DashQuery,
    pub size: Size,
    // TODO variant?
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct IconElement {
    pub icon: String,
    /// instead of getting icon from `name`
    /// query for Component::Icon
    pub icon_value: Option<DashQueryNoType>,
    pub size: Size,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ButtonElement {
    /// calls a ::Set query with ComponentType::Trigger
    // TODO should also be able to run Penguin script
    // Or maybe call custom query with specific value?
    // And definetely be able to navigate to other Dashboards
    pub on_click: Option<DashQueryNoType>,
    pub size: Size,
    pub variant: ButtonVariant,
    pub children: Vec<DashElement>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TextElement {
    pub value: Option<DashQuery>,
    pub prefix: String,
    pub suffix: String,
    pub size: Size,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TextInputElement {
    pub title: String,
    pub placeholder: String,
    /// ComponentType must have a string (ex ::Text)
    pub binding: DashQuery,
    /// Disables \*MaxLength, \*MinLength, \*Pattern enforcement
    pub disable_validation: bool,
    pub is_password: bool,
    pub multi_line: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct NumberInputElement {
    pub title: String,
    pub placeholder: String,
    /// ComponentType must have a number (ex ::Int, ::Float)
    pub binding: DashQuery,
    /// If only 1 component is queried, and it's entity
    /// also has bounds (\*Min, \*Max, \*Step) those will
    /// be enforced UNLESS this is set to true
    pub disable_validation: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TimePickerElement {
    pub binding: DashQueryNoType,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DatePickerElement {
    pub binding: DashQueryNoType,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DateTimePickerElement {
    pub binding: DashQueryNoType,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DurationPickerElement {
    pub binding: DashQueryNoType,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct WeekdayPickerElement {
    pub binding: DashQueryNoType,
    /// If multi uses WeekdayList
    /// Else Weekday
    pub multi: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SliderElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a number (ex ::Int, ::Float)
    pub binding: DashQuery,
    /// If only 1 component is queried, and it's entity
    /// also has bounds (\*Min, \*Max, \*Step) those will
    /// be enforced UNLESS this is set to true
    pub disable_validation: bool,
    /// Override valiation params
    /// Must be Component ::Int or ::Float
    pub min: Option<Component>,
    pub max: Option<Component>,
    pub step: Option<Component>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ColorTemperaturePickerElement {
    pub binding: DashQueryNoType,
    // for now its just a wide colored slider, but might
    // add variants
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ColorPickerElement {
    pub binding: DashQueryNoType,
    pub variant: ColorPickerVariant,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TextSelectElement {
    // Finds entities marked TextSelect
    // Current value is Component::Text
    // Options are Component::TextList
    pub binding: DashQueryNoType,
    pub variant: SelectVariant,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ModeSelectElement {
    /// Component must have Supported type
    /// For example, you'd put FanOscillation here, the options
    /// will be taken from SupportedFanOscillations
    pub binding: DashQuery,
    pub variant: SelectVariant,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CustomSelectElement {
    pub binding: DashQuery,
    pub variant: SelectVariant,
    /// (option name, value)
    pub options: Vec<(String, Component)>,
}

/// filler for now
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ChartElement {}

/// filler for now
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TableElement {}

/// filler for now
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct VideoFeedElement {}

/// filler for now
/// should be able to link to internal pages (other dashboards)
/// and external links
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct LinkElement {}

/// filler for now
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ImageElement {}

/// filler for now
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CollapsableElement {}

#[derive(Clone, Default, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ColorPickerVariant {
    #[default]
    Circle,
    HueSlider,
    Hsl,
}

#[derive(Clone, Default, Display, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum SelectVariant {
    Dropdown,
    #[default]
    Panel,
    Radio,
}

#[derive(Clone, Default, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Clone, Default, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Clone, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Clone, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
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
