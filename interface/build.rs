use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
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
    #[serde(default)]
    desc: String,
    #[serde(default)]
    related: Vec<Related>,
    #[serde(default)]
    gen_supported_type: bool,
    #[serde(flatten)]
    kind: ComponentKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ComponentKind {
    Single {
        field: String,
        #[serde(default)]
        gen_bound_types: bool,
        #[serde(default)]
        gen_list_type: bool,
        #[serde(default)]
        gen_string_bound_types: bool,
    },
    Struct {
        fields: Vec<Field>,
    },
    Enum {
        variants: Vec<Variant>,
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
    data: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Related {
    name: String,
    reason: String,
}

fn main() {
    println!("cargo:rerun-if-changed=components.toml");

    // read toml file
    let man_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&man_dir).join("components.toml");
    let toml_content = fs::read_to_string(toml_path).expect("Failed to read components.toml");
    let config: ComponentsConfig =
        toml::from_str(&toml_content).expect("Failed to parse components.toml");
    let mut comps = config.components;

    // add more components based off gen_* flags
    add_gen_comps(&mut comps);

    // generate rust code
    let code = gen_all(&comps);

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("components.rs");
    fs::write(&out_path, formatted).expect("Failed to write components.rs");
}

fn gen_all(comps: &[Component]) -> TokenStream {
    let comp_enum = gen_component_enum(comps);
    let comp_type_enum = gen_component_type_enum(comps);
    let comp_type_impl = gen_component_type_impl(comps);
    let entity_methods = gen_entity_methods(comps);
    let gen_comps = gen_components(comps);

    quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY
        // Generated from components.toml by build.rs

        use serde::{Deserialize, Serialize};
        use std::ops::{Add, Sub};

        #comp_enum

        #comp_type_enum

        #comp_type_impl

        #gen_comps

        impl Entity {
            #entity_methods
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
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
        #[repr(u16)]
        pub enum ComponentType {
            #(#variants),*
        }
    }
}

fn gen_component_type_impl(comps: &[Component]) -> TokenStream {
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

fn gen_components(comps: &[Component]) -> TokenStream {
    let comps: Vec<_> = comps
        .iter()
        .map(|comp| {
            let doc = gen_doc_attr(&comp.desc, &comp.related);
            let comp_code = match &comp.kind {
                ComponentKind::Single { field, .. } => gen_single_component(comp, field),
                ComponentKind::Struct { fields } => gen_struct_component(comp, fields),
                ComponentKind::Enum { variants } => gen_enum_component(comp, variants),
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
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
        pub struct #name;
    }
}

fn gen_single_component(comp: &Component, field: &str) -> TokenStream {
    let name = ident(&comp.name);
    let field_type = field.parse::<TokenStream>().unwrap_or_else(|_| {
        quote! { #field }
    });

    let averageable_impl = gen_single_averageable_impl(&comp.name, field);

    quote! {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
        pub struct #name(pub #field_type);

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

    quote! {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
        pub struct #name {
            #(#field_defs),*
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
        #[derive(Clone, Debug, Default)]
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

fn gen_enum_component(comp: &Component, variants: &[Variant]) -> TokenStream {
    let name = ident(&comp.name);

    let variant_defs: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = ident(&v.name);
            let data = v
                .data
                .as_ref()
                .map(|d| {
                    let data_type: TokenStream = d.parse().unwrap();
                    quote! { (#data_type) }
                })
                .unwrap_or_else(|| quote! {});

            // FIXME my eyes hurt
            let aliases = v
                .aliases
                .as_ref()
                .map(|aliases| {
                    let alias_attrs: Vec<_> = aliases
                        .iter()
                        .map(|a| {
                            quote! { alias = #a }
                        })
                        .collect();
                    quote! {
                        #[serde(#(#alias_attrs),*)]
                    }
                })
                .unwrap_or_else(|| quote! {});

            quote! {
                #aliases
                #variant_name #data
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
        pub enum #name {
            #(#variant_defs),*
        }
    }
}

fn gen_entity_methods(comp: &[Component]) -> TokenStream {
    let methods: Vec<_> = comp
        .iter()
        .map(|c| {
            let comp_name = ident(&c.name);
            let get = ident(&to_snake_case(&c.name));
            let get_mut = format_ident!("{}_mut", to_snake_case(&c.name));
            let set = format_ident!("set_{}", to_snake_case(&c.name));
            let with = format_ident!("with_{}", to_snake_case(&c.name));
            let has = format_ident!("has_{}", to_snake_case(&c.name));

            quote! {
                pub fn #get(&self) -> Option<&#comp_name> {
                    match self.0.get(&ComponentType::#comp_name) {
                        Some(Component::#comp_name(val)) => Some(val),
                        Some(_) => panic!("Entity Type/Value Mismatch!"),
                        None => None,
                    }
                }

                pub fn #get_mut(&mut self) -> Option<&mut #comp_name> {
                    match self.0.get_mut(&ComponentType::#comp_name) {
                        Some(Component::#comp_name(val)) => Some(val),
                        Some(_) => panic!("Entity Type/Value Mismatch!"),
                        None => None,
                    }
                }

                pub fn #set(&mut self, val: #comp_name) {
                    self.0.insert(ComponentType::#comp_name, Component::#comp_name(val));
                }

                pub fn #with(mut self, val: #comp_name) -> Self {
                    self.0.insert(ComponentType::#comp_name, Component::#comp_name(val));
                    self
                }

                pub fn #has(&self) -> bool {
                    self.#get().is_some()
                }
            }
        })
        .collect();

    quote! { #(#methods)* }
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

fn add_gen_comps(comps: &mut Vec<Component>) {
    let mut new_comps = Vec::new();

    for comp in comps.iter_mut() {
        if let ComponentKind::Single {
            field,
            gen_bound_types,
            gen_list_type,
            gen_string_bound_types,
            ..
        } = &comp.kind
        {
            if *gen_bound_types {
                for suffix in &["Min", "Max", "Step"] {
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
                        desc: "Marks a bound. Only enforced by dashboard components that use it."
                            .to_string(),
                        gen_supported_type: false,
                        kind: ComponentKind::single(field.clone()),
                        related: vec![Related {
                            name: comp.name.clone(),
                            reason,
                        }],
                    });
                }
            }

            if *gen_list_type {
                new_comps.push(Component {
                    name: format!("{}List", comp.name),
                    desc: format!("a variable-length list of {}", field),
                    related: Vec::new(),
                    gen_supported_type: false,
                    kind: ComponentKind::single(format!("Vec<{}>", field)),
                });
            }

            if *gen_string_bound_types {
                if field != "String" {
                    panic!("Cannot implement String bound types for a non-String {field}")
                }

                for suffix in &["MaxLength", "MinLength", "Pattern"] {
                    let new_name = format!("{}{}", comp.name, suffix);
                    let (reason, field_type) = match *suffix {
                        "MaxLength" => (
                            "sets a max length bound on".to_string(),
                            "usize".to_string(),
                        ),
                        "MinLength" => (
                            "sets a min length bound on".to_string(),
                            "usize".to_string(),
                        ),
                        "Pattern" => ("set a regex pattern for".to_string(), "String".to_string()),
                        _ => unreachable!(),
                    };

                    new_comps.push(Component {
                        name: new_name,
                        desc: "Marks a requirement. Only enforced by dashbaord components that use it.".to_string(),
                        gen_supported_type: false,
                        kind: ComponentKind::single(field_type),
                        related: vec![Related {
                            name: comp.name.clone(),
                            reason
                        }]
                    });
                }
            }
        }

        if comp.gen_supported_type {
            new_comps.push(Component {
                name: format!("Supported{}s", comp.name),
                desc: format!("specifies what {}s are supported by this entity", comp.name),
                related: Vec::new(),
                gen_supported_type: false,
                kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
            });

            comp.related.push(Related {
                name: format!("Supported{}s", comp.name),
                reason: "specifies what is supported by the entity".to_string(),
            });
        }
    }

    comps.append(&mut new_comps);
}

impl ComponentKind {
    fn single(field: String) -> Self {
        ComponentKind::Single {
            field,
            gen_bound_types: false,
            gen_list_type: false,
            gen_string_bound_types: false,
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
        "usize" => Some("usize"),
        "i8" | "i16" | "i32" => Some("i64"),
        "i64" => Some("i64"),
        "i128" => Some("i128"),
        "isize" => Some("isize"),
        "f32" => Some("f64"),
        "f64" => Some("f64"),
        "bool" => Some("u32"),
        _ => None,
    }
}

fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            res.push('_');
        }
        res.push(c.to_ascii_lowercase());
    }
    res
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
