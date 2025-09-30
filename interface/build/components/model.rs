use serde::Deserialize;

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
    /// generates Supported{name}s
    /// a vector of the entire type
    #[serde(default)]
    pub gen_supported_type: Option<u16>,
    /// generates {name}List
    /// a vector of the entire type
    #[serde(default)]
    pub gen_list_type: Option<u16>,
    #[serde(flatten)]
    pub kind: ComponentKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ComponentKind {
    Single {
        field: String,
        /// generates {name}Min, {name}Max, {name}Step
        #[serde(default)]
        gen_bound_types: Option<[u16; 3]>,
        /// generates {name}List, which is a list
        /// of the INTERNAL type
        #[serde(default)]
        gen_inner_list_type: Option<u16>,
        /// generates {name}MinLength, {name}MaxLength, {name}Pattern
        /// only valid for String types
        #[serde(default)]
        gen_string_bound_types: Option<[u16; 3]>,
    },
    Struct {
        fields: Vec<Field>,
    },
    Enum {
        variants: Vec<Variant>,
        // Adds a variant Custom(String)
        // Implements From<String> instead of TryFrom<String>
        #[serde(default)]
        allow_custom: bool,
    },
    Marker,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    // TODO add docs?
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
