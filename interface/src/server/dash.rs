use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{Display, From};

use crate::{Component, ComponentType, QueryFilter, QueryTarget};

// TODO we need to experiment with different systems
// for sizing, margins, and padding. For now we will
// leave that off

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Dashboard {
    pub display_name: String,
    /// used for custom queries in this
    /// dashboard, not defined inside
    /// CustomElements
    pub targets: HashMap<String, QueryTarget>,
    pub child: DashElement,
    /// Overwritten by Igloo at runtime
    pub idx: Option<u16>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, From, PartialEq)]
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
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DashQuery {
    pub target: String,
    pub filter: QueryFilter,
    pub comp_type: ComponentType,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DashQueryNoType {
    pub target: String,
    pub filter: QueryFilter,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CustomElement {
    pub name: String,
    pub selected_targets: HashMap<String, QueryTarget>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct IfElement {
    pub condition: Expr,
    pub then: Vec<DashElement>,
    pub r#else: Vec<DashElement>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct RepeatElement {
    pub count: Expr,
    pub each: Vec<DashElement>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ForEachElement {}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct HStackElement {
    pub justify: HAlign,
    pub align: VAlign, // TODO this right?
    pub scroll: bool,
    pub children: Vec<DashElement>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct VStackElement {
    pub justify: VAlign,
    pub align: HAlign,
    pub scroll: bool,
    pub children: Vec<DashElement>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TabsElement {
    pub pages: HashMap<String, Vec<DashElement>>,
}

/// make a badge with `Card(HStack { children: [..] })`
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CardElement {
    pub child: Box<DashElement>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SwitchElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a bool (ex ::Bool, ::Switch)
    /// Will register a ::WatchAvg query
    /// When interacted, calls a ::Set query
    pub binding: DashQuery,
    pub size: Size,
    // TODO variant?
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CheckboxElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a bool (ex ::Bool, ::Switch)
    /// Will register a ::WatchAvg query
    /// When interacted, calls a ::Set query
    pub binding: DashQuery,
    pub size: Size,
    // TODO variant?
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ToggleButtonElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a bool (ex ::Bool, ::Switch)
    /// Will register a ::WatchAvg query
    /// When interacted, calls a ::Set query
    pub binding: DashQuery,
    pub size: Size,
    // TODO variant?
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct IconElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// instead of getting icon from `name`
    /// query for Component::Icon
    pub icon_value: Option<DashQueryNoType>,
    pub icon: Option<String>,
    pub size: Size,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TextElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub value: Option<DashQuery>,
    pub prefix: String,
    pub suffix: String,
    pub size: Size,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TextInputElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a string (ex ::Text)
    pub binding: DashQuery,
    pub title: String,
    pub placeholder: String,
    /// Disables \*MaxLength, \*MinLength, \*Pattern enforcement
    pub disable_validation: bool,
    pub is_password: bool,
    pub multi_line: bool,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct NumberInputElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a number (ex ::Int, ::Float)
    pub binding: DashQuery,
    pub title: String,
    pub placeholder: String,
    /// If only 1 component is queried, and it's entity
    /// also has bounds (\*Min, \*Max, \*Step) those will
    /// be enforced UNLESS this is set to true
    pub disable_validation: bool,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TimePickerElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQueryNoType,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DatePickerElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQueryNoType,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DateTimePickerElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQueryNoType,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct DurationPickerElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQueryNoType,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct WeekdayPickerElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQueryNoType,
    /// If multi uses WeekdayList
    /// Else Weekday
    pub multi: bool,
}

// TODO orientation??
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SliderElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// ComponentType must have a number (ex ::Int, ::Float)
    pub binding: DashQuery,
    /// Find min,max,step from entity Components \*Min,\*Max,\*Step
    /// (ex. IntMax, IntMin, IntStep)
    /// Note: only works if query is for 1 entity
    pub auto_validate: bool,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ColorPickerElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQueryNoType,
    pub variant: ColorPickerVariant,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TextSelectElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    // Finds entities marked TextSelect
    // Current value is Component::Text
    // Options are Component::TextList
    pub binding: DashQueryNoType,
    pub variant: SelectVariant,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ModeSelectElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    /// Component must have Supported type
    /// For example, you'd put FanOscillation here, the options
    /// will be taken from SupportedFanOscillations
    pub binding: DashQuery,
    pub variant: SelectVariant,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CustomSelectElement {
    /// DO NOT SAVE
    /// Will be set by Igloo Server
    pub watch_id: Option<u32>,
    pub binding: DashQuery,
    pub variant: SelectVariant,
    /// (option name, value)
    pub options: Vec<(String, Component)>,
}

/// filler for now
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ChartElement {}

/// filler for now
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TableElement {}

/// filler for now
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct VideoFeedElement {}

/// filler for now
/// should be able to link to internal pages (other dashboards)
/// and external links
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct LinkElement {}

/// filler for now
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ImageElement {}

/// filler for now
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CollapsableElement {}

#[derive(Debug, Clone, Default, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ColorPickerVariant {
    /// hue/saturation circle
    #[default]
    ColorWheel,

    /// HSV satuation
    SaturationSlider,
    /// HSV value
    ValueSlider,
    /// HSV hue
    HueSlider,

    RedSlider,
    GreenSlider,
    BlueSlider,

    /// saturation/value square
    Square,
}

#[derive(Debug, Clone, Default, Display, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum SelectVariant {
    Dropdown,
    #[default]
    Panel,
    Radio,
}

#[derive(Debug, Clone, Default, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Debug, Clone, Default, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Debug, Clone, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Debug, Clone, Display, BorshSerialize, BorshDeserialize, PartialEq)]
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

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
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
