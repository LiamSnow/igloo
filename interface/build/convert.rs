use super::model::*;
use super::types::ident;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_to_igloo_value(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let igloo_variant = kind.tokens();
                    quote! {
                        Component::#name(v) => Some(IglooValue::#igloo_variant(v.clone()))
                    }
                }
                ComponentKind::Enum { .. } => {
                    quote! {
                        Component::#name(v) => Some(IglooValue::Enum(IglooEnumValue::#name(v.clone())))
                    }
                }
                ComponentKind::Marker { .. } => {
                    quote! {
                        Component::#name => None
                    }
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn to_igloo_value(&self) -> Option<IglooValue> {
                match self {
                    #(#arms,)*
                }
            }
        }
    }
}

pub fn gen_from_igloo_value(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let igloo_variant = kind.tokens();
                    quote! {
                        ComponentType::#name => {
                            if let IglooValue::#igloo_variant(v) = value {
                                Some(Component::#name(v))
                            } else {
                                None
                            }
                        }
                    }
                }
                ComponentKind::Enum { .. } => {
                    quote! {
                        ComponentType::#name => {
                            if let IglooValue::Enum(IglooEnumValue::#name(v)) = value {
                                Some(Component::#name(v))
                            } else {
                                None
                            }
                        }
                    }
                }
                ComponentKind::Marker { .. } => {
                    quote! {
                        ComponentType::#name => Some(Component::#name)
                    }
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn from_igloo_value(r#type: ComponentType, value: IglooValue) -> Option<Self> {
                match r#type {
                    #(#arms,)*
                }
            }
        }
    }
}

pub fn gen_comp_from_string(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .filter_map(|comp| {
            let comp_ident = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let parse_logic = match kind {
                        IglooType::Integer
                        | IglooType::Real
                        | IglooType::Boolean
                        | IglooType::Color
                        | IglooType::Date
                        | IglooType::Time => {
                            quote! { s.parse().ok().map(Component::#comp_ident) }
                        }
                        IglooType::Text => {
                            quote! { Some(Component::#comp_ident(s)) }
                        }
                        IglooType::IntegerList
                        | IglooType::RealList
                        | IglooType::BooleanList
                        | IglooType::ColorList
                        | IglooType::DateList
                        | IglooType::TimeList => {
                            quote! {
                                parse_list(&s)?
                                    .into_iter()
                                    .map(|item| item.parse().ok())
                                    .collect::<Option<Vec<_>>>()
                                    .map(Component::#comp_ident)
                            }
                        }
                        IglooType::TextList => {
                            quote! { parse_list(&s).map(Component::#comp_ident) }
                        }
                    };

                    Some(quote! {
                        ComponentType::#comp_ident => #parse_logic
                    })
                }
                ComponentKind::Enum { .. } => Some(quote! {
                    ComponentType::#comp_ident => s.try_into().ok().map(Component::#comp_ident)
                }),
                ComponentKind::Marker { .. } => None,
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn from_string(comp_type: ComponentType, s: String) -> Option<Component> {
                match comp_type {
                    #(#arms,)*
                    _ => None
                }
            }
        }
    }
}

pub fn gen_comp_inner(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            if comp.is_marker() {
                quote! {}
            } else {
                quote! {
                    Component::#name(payload) => {
                        Some(format!("{payload:?}"))
                    }
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn inner_string(&self) -> Option<String> {
                match self {
                    #(#arms)*
                    _ => None
                }
            }
        }
    }
}
