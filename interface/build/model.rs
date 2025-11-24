use crate::types::ident;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::{fs, path::PathBuf};
use syn::Ident;

#[derive(Debug, Deserialize)]
pub struct ComponentsConfig {
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub name: String,
    pub id: u16,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub related: Vec<Related>,
    #[serde(flatten)]
    #[serde(default)]
    pub kind: ComponentKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ComponentKind {
    Enum {
        kind: EnumTag,
        variants: Vec<Variant>,
    },
    Single {
        kind: IglooType,
    },
    Marker {
        #[serde(default)]
        kind: Option<MarkerTag>,
    },
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Marker {
            kind: Some(MarkerTag::Marker),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum EnumTag {
    Enum,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MarkerTag {
    Marker,
}

#[derive(Debug, Deserialize, Clone)]
pub enum IglooType {
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
}

#[derive(Debug, Deserialize, Clone)]
pub struct Variant {
    pub id: u8,
    pub name: String,
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Related {
    pub name: String,
    pub reason: String,
}

impl ComponentsConfig {
    pub fn read(pathbuf: PathBuf) -> Self {
        let contents = fs::read_to_string(pathbuf).expect("Failed to read components file");
        toml::from_str(&contents).expect("Failed to parse components file")
    }
}

impl IglooType {
    pub fn tokens(&self) -> TokenStream {
        match self {
            Self::Integer => quote! { Integer },
            Self::Real => quote! { Real },
            Self::Text => quote! { Text },
            Self::Boolean => quote! { Boolean },
            Self::Color => quote! { Color },
            Self::Date => quote! { Date },
            Self::Time => quote! { Time },
            Self::IntegerList => quote! { IntegerList },
            Self::RealList => quote! { RealList },
            Self::TextList => quote! { TextList },
            Self::BooleanList => quote! { BooleanList },
            Self::ColorList => quote! { ColorList },
            Self::DateList => quote! { DateList },
            Self::TimeList => quote! { TimeList },
        }
    }

    pub fn direct_type_tokens(&self) -> TokenStream {
        match self {
            Self::Integer => quote! { IglooInteger },
            Self::Real => quote! { IglooReal },
            Self::Text => quote! { IglooText },
            Self::Boolean => quote! { IglooBoolean },
            Self::Color => quote! { IglooColor },
            Self::Date => quote! { IglooDate },
            Self::Time => quote! { IglooTime },
            Self::IntegerList => quote! { IglooIntegerList },
            Self::RealList => quote! { IglooRealList },
            Self::TextList => quote! { IglooTextList },
            Self::BooleanList => quote! { IglooBooleanList },
            Self::ColorList => quote! { IglooColorList },
            Self::DateList => quote! { IglooDateList },
            Self::TimeList => quote! { IglooTimeList },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AggOp {
    Sum,
    Mean,
    Max,
    Min,
    Any,
    All,
}

impl AggOp {
    pub fn ident(&self) -> Ident {
        ident(match self {
            AggOp::Sum => "Sum",
            AggOp::Mean => "Mean",
            AggOp::Max => "Max",
            AggOp::Min => "Min",
            AggOp::Any => "Any",
            AggOp::All => "All",
        })
    }
}
