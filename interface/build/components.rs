use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct ComponentsConfig {
    components: Vec<Component>,
}

#[derive(Debug, Deserialize, Clone)]
struct Component {
    name: String,
    id: u16,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    related: Vec<Related>,
    /// generates Supported{name}s
    /// a vector of the entire type
    #[serde(default)]
    gen_supported_type: Option<u16>,
    /// generates {name}List
    /// a vector of the entire type
    #[serde(default)]
    gen_list_type: Option<u16>,
    #[serde(flatten)]
    kind: ComponentKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ComponentKind {
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
struct Field {
    name: String,
    r#type: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Variant {
    name: String,
    aliases: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
struct Related {
    name: String,
    reason: String,
}

pub fn run() {
    println!("cargo:rerun-if-changed=components.toml");

    // read toml file
    let man_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&man_dir).join("components.toml");
    let toml_content = fs::read_to_string(toml_path).expect("Failed to read components.toml");
    let config: ComponentsConfig =
        toml::from_str(&toml_content).expect("Failed to parse components.toml");
    let mut comps = config.components;

    // add more components based off gen_* flags
    let extra_code = add_gen_comps(&mut comps);

    // make sure no IDs conflict or are skipped
    validate_ids(&comps);

    // generate rust code
    let code = gen_all(&comps, extra_code);

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("components.rs");
    fs::write(&out_path, formatted).expect("Failed to write components.rs");
}

fn validate_ids(comps: &[Component]) {
    let mut ids = HashSet::new();

    for comp in comps {
        if !ids.insert(comp.id) {
            panic!(
                "Component {} tried to use ID {} but it's already taken! Please take extreme caution to make sure IDs are consistent with old versions",
                comp.name, comp.id
            );
        }
    }

    if let (Some(&min), Some(&max)) = (ids.iter().min(), ids.iter().max()) {
        for id in min..=max {
            if !ids.contains(&id) {
                panic!("ID {} was skipped!", id);
            }
        }
    }
}

fn gen_all(comps: &[Component], extra_code: TokenStream) -> TokenStream {
    let comp_enum = gen_component_enum(comps);
    let comp_type_enum = gen_component_type_enum(comps);
    let comp_get_type = gen_component_get_type(comps);
    let comp_type_id = gen_component_type_id(comps);
    let comp_codable = gen_component_codable(comps);
    let prim_codable = gen_primitives_codable();
    let gen_comps = gen_components(comps);

    quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY
        // Generated from components.toml by build.rs

        use std::ops::{Add, Sub};
        use bytes::{Bytes, BytesMut, BufMut, Buf};

        #comp_enum

        #comp_type_enum

        #comp_get_type

        #comp_type_id

        #comp_codable

        #prim_codable

        #gen_comps

        #extra_code
    }
}

fn gen_primitives_codable() -> TokenStream {
    let nums = vec![
        ("u8", "u8"),
        ("u16", "u16_le"),
        ("u32", "u32_le"),
        ("u64", "u64_le"),
        ("u128", "u128_le"),
        ("i8", "i8"),
        ("i16", "i16_le"),
        ("i32", "i32_le"),
        ("i64", "i64_le"),
        ("i128", "i128_le"),
        ("f32", "f32_le"),
        ("f64", "f64_le"),
    ];

    let nums_codable: Vec<_> = nums
        .iter()
        .map(|(typ, cmd)| {
            let typ = ident(typ);
            let put = ident(&format!("put_{cmd}"));
            let get = ident(&format!("get_{cmd}"));
            quote! {
                impl IglooCodable for #typ {
                    #[inline(always)]
                    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                        buf.#put(*self);
                        Ok(())
                    }

                    #[inline(always)]
                    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                        Ok(buf.#get())
                    }
                }
            }
        })
        .collect();

    let vec_types = vec![
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
        "String",
    ];

    let vec_codable: Vec<_> = vec_types
        .iter()
        .map(|typ| {
            let typ = ident(typ);
            quote! {
                impl IglooCodable for Vec<#typ> {
                    #[inline(always)]
                    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                        buf.put_u32_le(self.len() as u32);
                        for item in self {
                            item.encode(buf)?;
                        }
                        Ok(())
                    }

                    #[inline(always)]
                    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                        let len = buf.get_u32_le() as usize;
                        let mut vec = Vec::with_capacity(len);
                        for _ in 0..len {
                            vec.push(#typ::decode(buf)?);
                        }
                        Ok(vec)
                    }
                }
            }
        })
        .collect();

    quote! {
        #(#nums_codable)*
        #(#vec_codable)*
    }
}

fn gen_component_codable(comps: &[Component]) -> TokenStream {
    let encode_arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let id = comp.id;
            quote! {
                Component::#name(val) => {
                    buf.put_u16_le(#id);
                    val.encode(buf)?;
                }
            }
        })
        .collect();

    let decode_arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let id = comp.id;
            quote! {
                #id => Ok(Component::#name(#name::decode(buf)?))
            }
        })
        .collect();

    quote! {
        impl IglooCodable for Component {
            fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                match self {
                    #(#encode_arms),*
                }

                Ok(())
            }

            fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                if buf.is_empty() {
                    return Err(IglooCodecError::InvalidMessage);
                }

                let id = buf.get_u16_le();
                match id {
                    #(#decode_arms),*,
                    _ => Err(IglooCodecError::UnknownComponent(id)),
                }
            }
        }
    }
}

fn gen_component_enum(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .map(|c| {
            let name = ident(&c.name);
            quote! { #name(#name) }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum Component {
            #(#variants),*
        }
    }
}

fn gen_component_type_enum(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .map(|c| {
            let name = ident(&c.name);
            quote! { #name }
        })
        .collect();

    quote! {
        #[derive(Debug, PartialEq, Eq, Clone, Hash, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        #[repr(u16)]
        pub enum ComponentType {
            #(#variants),*
        }
    }
}

fn gen_component_get_type(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|c| {
            let name = ident(&c.name);
            quote! {
                Component::#name(_) => ComponentType::#name
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn get_type(&self) -> ComponentType {
                match self {
                    #(#arms),*
                }
            }
        }
    }
}

fn gen_component_type_id(comps: &[Component]) -> TokenStream {
    let mut ctarms = Vec::with_capacity(comps.len());
    let mut carms = Vec::with_capacity(comps.len());
    let mut earms = Vec::with_capacity(comps.len());
    let mut darms = Vec::with_capacity(comps.len());

    for comp in comps {
        let name = ident(&comp.name);
        let id = comp.id;

        ctarms.push(quote! {
            ComponentType::#name => #id
        });

        carms.push(quote! {
            Component::#name(..) => #id
        });

        earms.push(quote! {
            Component::#name(data) => {
                bincode::Encode::encode(&#id, encoder)?;
                bincode::Encode::encode(&data, encoder)
            }
        });

        darms.push(quote! {
            #id => {
                Ok(Component::#name(<#name as bincode::Decode<C>>::decode(decoder)?))
            }
        });
    }

    quote! {
        // TODO remove this
        impl ComponentType {
            pub fn get_id(&self) -> u16 {
                match self {
                    #(#ctarms),*
                }
            }
        }

        // TODO remove this
        impl Component {
            pub fn get_id(&self) -> u16 {
                match self {
                    #(#carms),*
                }
            }
        }

        impl bincode::Encode for Component {
            fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
                match self {
                    #(#earms),*
                }
            }
        }

        impl<C> bincode::Decode<C> for Component {
            fn decode<D: bincode::de::Decoder<Context = C>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
                let id = <u16 as bincode::Decode<C>>::decode(decoder)?;
                match id {
                    #(#darms),*
                    _ => Err(bincode::error::DecodeError::Other("Unknown component ID")),
                }
            }
        }

        impl<'de, C> bincode::BorrowDecode<'de, C> for Component {
            fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
                decoder: &mut D,
            ) -> Result<Self, bincode::error::DecodeError> {
                <Self as bincode::Decode<C>>::decode(decoder)
            }
        }
    }
}

fn gen_components(comps: &[Component]) -> TokenStream {
    let comps: Vec<_> = comps
        .iter()
        .map(|comp| {
            let doc = gen_doc_attr(&comp.desc, &comp.related);
            let comp_code = match &comp.kind {
                ComponentKind::Single { field, .. } => gen_single_component(comp, field),
                ComponentKind::Struct { fields } => gen_struct_component(comp, fields),
                ComponentKind::Enum {
                    variants,
                    allow_custom,
                } => gen_enum_component(comp, variants, *allow_custom),
                ComponentKind::Marker => gen_marker_component(comp),
            };
            quote! {
                #doc
                #comp_code
            }
        })
        .collect();

    quote! { #(#comps)* }
}

fn gen_doc_attr(desc: &str, related: &[Related]) -> TokenStream {
    let mut doc_parts = Vec::new();

    if !desc.is_empty() {
        doc_parts.push(desc.to_string());
    }

    if !related.is_empty() {
        if !doc_parts.is_empty() {
            doc_parts.push(String::new());
        }
        doc_parts.push("Usually paired with:".to_string());

        for rel in related {
            doc_parts.push(format!(" - [{}] {}", rel.name, rel.reason));
        }
    }

    if doc_parts.is_empty() {
        quote! {}
    } else {
        let combined_doc = doc_parts.join("\n");
        quote! { #[doc = #combined_doc] }
    }
}

fn gen_marker_component(comp: &Component) -> TokenStream {
    let name = ident(&comp.name);

    quote! {
        #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        pub struct #name;

        impl IglooCodable for #name {
            fn encode(&self, _buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                Ok(())
            }

            fn decode(_buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                Ok(#name)
            }
        }
    }
}

fn gen_single_component(comp: &Component, field: &str) -> TokenStream {
    let name = ident(&comp.name);
    let field_type = field.parse::<TokenStream>().unwrap_or_else(|_| {
        quote! { #field }
    });

    let averageable_impl = gen_single_averageable_impl(&comp.name, field);

    quote! {
        #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        pub struct #name(pub #field_type);

        impl IglooCodable for #name {
            fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                self.0.encode(buf)
            }

            fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                Ok(#name(<#field_type>::decode(buf)?))
            }
        }

        #averageable_impl
    }
}

fn gen_struct_component(comp: &Component, fields: &[Field]) -> TokenStream {
    let name = ident(&comp.name);

    let field_defs: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = ident(&f.name);
            let field_type: TokenStream = f.r#type.parse().unwrap_or_else(|_| {
                let type_str = &f.r#type;
                quote! { #type_str }
            });
            quote! { pub #field_name: #field_type }
        })
        .collect();

    let averageable_impl = gen_struct_averageable_impl(comp, fields);

    let encode_lines: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = ident(&f.name);
            quote! {
                self.#field_name.encode(buf)?;
            }
        })
        .collect();

    let decode_lines: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = ident(&f.name);
            let field_type: TokenStream = f.r#type.parse().unwrap_or_else(|_| {
                let type_str = &f.r#type;
                quote! { #type_str }
            });
            quote! {
                #field_name: <#field_type>::decode(buf)?
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        pub struct #name {
            #(#field_defs),*
        }

        impl IglooCodable for #name {
            fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                #(#encode_lines)*
                Ok(())
            }

            fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                Ok(Self {
                    #(#decode_lines),*
                })
            }
        }

        #averageable_impl
    }
}

/// If all fields in this struct are averageable (numbers, booleans)
/// Then we will create a struct to sum up all of this struct
/// And implement averageable for it
fn gen_struct_averageable_impl(comp: &Component, fields: &[Field]) -> TokenStream {
    let mut sum_types = Vec::new();
    for field in fields {
        if let Some(sum_type_str) = get_sum_type(&field.r#type) {
            sum_types.push((field, sum_type_str));
        } else {
            return quote! {}; // not Averageable -> dont impl
        }
    }

    if sum_types.is_empty() {
        return quote! {};
    }

    let name = ident(&comp.name);
    let sum_name = format_ident!("{}Sum", comp.name);

    let mut sum_fields = Vec::new();
    let mut add_fields = Vec::new();
    let mut sub_fields = Vec::new();
    let mut to_sum_fields = Vec::new();
    let mut from_sum_fields = Vec::new();

    for (field, sum_type_str) in &sum_types {
        let field_name = ident(&field.name);
        let field_type: TokenStream = field.r#type.parse().unwrap();
        let sum_type: TokenStream = sum_type_str.parse().unwrap();

        sum_fields.push(quote! { pub #field_name: #sum_type });
        add_fields.push(quote! { #field_name: self.#field_name + other.#field_name });
        sub_fields.push(quote! { #field_name: self.#field_name - other.#field_name });
        to_sum_fields.push(quote! { #field_name: self.#field_name as #sum_type });
        from_sum_fields.push(if field.r#type == "bool" {
            quote! { #field_name: (sum.#field_name / len as #sum_type) != 0 }
        } else if *sum_type_str != field.r#type {
            quote! { #field_name: (sum.#field_name / len as #sum_type) as #field_type }
        } else {
            quote! { #field_name: sum.#field_name / len as #sum_type }
        });
    }

    quote! {
        #[derive(Clone, Debug, Default, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        pub struct #sum_name {
            #(#sum_fields),*
        }

        impl Add for #sum_name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                #sum_name {
                    #(#add_fields),*
                }
            }
        }

        impl Sub for #sum_name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                #sum_name {
                    #(#sub_fields),*
                }
            }
        }

        impl Averageable for #name {
            type Sum = #sum_name;

            fn to_sum_component(&self) -> Self::Sum {
                #sum_name {
                    #(#to_sum_fields),*
                }
            }

            fn from_sum(sum: Self::Sum, len: usize) -> Self {
                #name {
                    #(#from_sum_fields),*
                }
            }
        }
    }
}

// TODO make enums parse as IDs?
// Furthermore, shoud aliases and name be case sensititve?
fn gen_enum_component(comp: &Component, variants: &[Variant], allow_custom: bool) -> TokenStream {
    let name = ident(&comp.name);

    let mut variant_defs: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant = ident(&v.name);
            quote! {
                #variant
            }
        })
        .collect();

    let arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant = ident(&v.name);
            let variant_str = v.name.clone();
            let aliases = v.aliases.clone().unwrap_or_default();

            let pattern = if aliases.is_empty() {
                quote! { #variant_str }
            } else {
                let alias_patterns: Vec<_> = aliases.iter().map(|a| quote! { #a }).collect();
                quote! { #variant_str | #(#alias_patterns)|* }
            };

            quote! {
                #pattern => #name::#variant
            }
        })
        .collect();

    let display_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant = ident(&v.name);
            let variant_str = &v.name;
            quote! {
                #name::#variant => write!(f, "{}", #variant_str)
            }
        })
        .collect();

    let (from_impl, display_impl, codable_impl) = match allow_custom {
        true => {
            variant_defs.push(quote!(Custom(String)));

            let from = quote! {
                impl From<String> for #name {
                    fn from(s: String) -> Self {
                        match s.as_str() {
                            #(#arms),*,
                            s => #name::Custom(s.to_string())
                        }
                    }
                }
            };

            let display = quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_arms),*,
                            #name::Custom(s) => write!(f, "{}", s)
                        }
                    }
                }
            };

            let codable = quote! {
                impl IglooCodable for #name {
                    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                        let s = self.to_string();
                        s.encode(buf)
                    }

                    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                        let s = String::decode(buf)?;
                        Ok(#name::from(s))
                    }
                }
            };

            (from, display, codable)
        }
        false => {
            let from = quote! {
                impl TryFrom<String> for #name {
                    type Error = ();

                    fn try_from(s: String) -> Result<Self, Self::Error> {
                        Ok(match s.as_str() {
                            #(#arms),*,
                            _ => return Err(())
                        })
                    }
                }
            };

            let display = quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_arms),*
                        }
                    }
                }
            };

            let codable = quote! {
                impl IglooCodable for #name {
                    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                        let s = self.to_string();
                        s.encode(buf)
                    }

                    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                        let s = String::decode(buf)?;
                        #name::try_from(s.clone())
                            .map_err(|_| IglooCodecError::InvalidEnumVariant(s))
                    }
                }
            };

            (from, display, codable)
        }
    };

    quote! {
        #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        pub enum #name {
            #(#variant_defs),*
        }

        #from_impl
        #display_impl
        #codable_impl
    }
}

/// If a Single kind has a type that is averageble (numbers, boolean)
/// Then we will impl the Averageable trait
fn gen_single_averageable_impl(name: &str, field_type: &str) -> TokenStream {
    let Some(sum_type) = get_sum_type(field_type) else {
        return quote! {};
    };

    let struct_name = ident(name);
    let sum_type_tokens: TokenStream = sum_type.parse().unwrap();
    let field_type_tokens: TokenStream = field_type.parse().unwrap();

    let from_sum_body = if field_type == "bool" {
        quote! { Self((sum / len as #sum_type_tokens) != 0) }
    } else if sum_type != field_type {
        quote! { Self((sum / len as #sum_type_tokens) as #field_type_tokens) }
    } else {
        quote! { Self(sum / len as #sum_type_tokens) }
    };

    quote! {
        impl Averageable for #struct_name {
            type Sum = #sum_type_tokens;

            fn to_sum_component(&self) -> Self::Sum {
                self.0 as Self::Sum
            }

            fn from_sum(sum: Self::Sum, len: usize) -> Self
            where
                Self: Sized,
            {
                #from_sum_body
            }
        }
    }
}

fn add_gen_comps(comps: &mut Vec<Component>) -> TokenStream {
    let mut new_comps = Vec::new();
    let mut extra_code_parts = Vec::new();

    for comp in comps.iter_mut() {
        if let ComponentKind::Single {
            field,
            gen_bound_types,
            gen_inner_list_type: gen_list_type,
            gen_string_bound_types,
            ..
        } = &comp.kind
        {
            if let Some(ids) = gen_bound_types {
                for (i, suffix) in ["Min", "Max", "Step"].iter().enumerate() {
                    let new_name = format!("{}{}", comp.name, suffix);
                    let reason = match *suffix {
                        "Min" => "sets a lower bound on",
                        "Max" => "sets an upper bound on",
                        "Step" => "sets a step requirement on",
                        _ => unreachable!(),
                    }
                    .to_string();

                    new_comps.push(Component {
                        name: new_name,
                        id: ids[i],
                        desc: "Marks a bound. Only enforced by dashboard components that use it."
                            .to_string(),
                        gen_supported_type: None,
                        gen_list_type: None,
                        kind: ComponentKind::single(field.clone()),
                        related: vec![Related {
                            name: comp.name.clone(),
                            reason,
                        }],
                    });
                }
            }

            if let Some(id) = gen_list_type {
                new_comps.push(Component {
                    name: format!("{}List", comp.name),
                    id: *id,
                    desc: format!("a variable-length list of {}", field),
                    related: Vec::new(),
                    gen_supported_type: None,
                    gen_list_type: None,
                    kind: ComponentKind::single(format!("Vec<{}>", field)),
                });
            }

            if let Some(ids) = gen_string_bound_types {
                if field != "String" {
                    panic!("Cannot implement String bound types for a non-String {field}")
                }

                for (i, suffix) in ["MaxLength", "MinLength", "Pattern"].iter().enumerate() {
                    let new_name = format!("{}{}", comp.name, suffix);
                    let (reason, field_type) = match *suffix {
                        "MaxLength" => {
                            ("sets a max length bound on".to_string(), "u32".to_string())
                        }
                        "MinLength" => {
                            ("sets a min length bound on".to_string(), "u32".to_string())
                        }
                        "Pattern" => ("set a regex pattern for".to_string(), "String".to_string()),
                        _ => unreachable!(),
                    };

                    new_comps.push(Component {
                        name: new_name,
                        id: ids[i],
                        desc: "Marks a requirement. Only enforced by dashbaord components that use it.".to_string(),
                        gen_supported_type: None,
                        gen_list_type: None,
                        kind: ComponentKind::single(field_type),
                        related: vec![Related {
                            name: comp.name.clone(),
                            reason
                        }]
                    });
                }
            }
        }

        if let Some(id) = comp.gen_supported_type {
            new_comps.push(Component {
                name: format!("Supported{}s", comp.name),
                id,
                desc: format!("specifies what {}s are supported by this entity", comp.name),
                related: Vec::new(),
                gen_supported_type: None,
                gen_list_type: None,
                kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
            });

            comp.related.push(Related {
                name: format!("Supported{}s", comp.name),
                reason: "specifies what is supported by the entity".to_string(),
            });

            // TODO move this somewhere else
            let comp_name = ident(&comp.name);
            extra_code_parts.push(quote! {
                impl IglooCodable for Vec<#comp_name> {
                    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                        buf.put_u32_le(self.len() as u32);
                        for item in self {
                            item.encode(buf)?;
                        }
                        Ok(())
                    }

                    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                        let len = buf.get_u32_le() as usize;
                        let mut vec = Vec::with_capacity(len);
                        for _ in 0..len {
                            vec.push(#comp_name::decode(buf)?);
                        }
                        Ok(vec)
                    }
                }
            });
        }

        if let Some(id) = comp.gen_list_type {
            new_comps.push(Component {
                name: format!("{}List", comp.name),
                id,
                desc: format!("A list of {}", comp.name),
                related: Vec::new(),
                gen_supported_type: None,
                gen_list_type: None,
                kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
            });

            // TODO move this plz!
            let comp_name = ident(&comp.name);
            extra_code_parts.push(quote! {
                impl IglooCodable for Vec<#comp_name> {
                    fn encode(&self, buf: &mut BytesMut) -> Result<(), IglooCodecError> {
                        buf.put_u32_le(self.len() as u32);
                        for item in self {
                            item.encode(buf)?;
                        }
                        Ok(())
                    }

                    fn decode(buf: &mut Bytes) -> Result<Self, IglooCodecError> {
                        let len = buf.get_u32_le() as usize;
                        let mut vec = Vec::with_capacity(len);
                        for _ in 0..len {
                            vec.push(#comp_name::decode(buf)?);
                        }
                        Ok(vec)
                    }
                }
            });
        }
    }

    comps.append(&mut new_comps);

    quote! {
        #(#extra_code_parts)*
    }
}

impl ComponentKind {
    fn single(field: String) -> Self {
        ComponentKind::Single {
            field,
            gen_bound_types: None,
            gen_inner_list_type: None,
            gen_string_bound_types: None,
        }
    }
}

/// returns what type to sum with when averaging
/// IE we can sum up u8's with u8's because it will overflow
fn get_sum_type(field_type: &str) -> Option<&'static str> {
    match field_type {
        "u8" | "u16" | "u32" => Some("u64"),
        "u64" => Some("u64"),
        "u128" => Some("u128"),
        "i8" | "i16" | "i32" => Some("i64"),
        "i64" => Some("i64"),
        "i128" => Some("i128"),
        "f32" => Some("f64"),
        "f64" => Some("f64"),
        "bool" => Some("u32"),
        _ => None,
    }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
