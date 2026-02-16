use crate::types::ident;
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::Ident;

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentsFile {
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub related: Vec<Related>,
    #[serde(flatten)]
    #[serde(default)]
    pub kind: ComponentKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EnumTag {
    Enum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerTag {
    Marker,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub aliases: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Related {
    pub name: String,
    pub reason: String,
}

impl ComponentsFile {
    pub fn make_map(&self, filename: &'static str) -> HashMap<&String, &Component> {
        let mut map = HashMap::with_capacity(self.components.len());

        for comp in &self.components {
            if map.insert(&comp.name, comp).is_some() {
                panic!("{filename}: Duplicate components `{}`", comp.name);
            }
        }

        map
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
