use super::model::*;
use crate::rust::ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

fn ops_for_kind(kind: &IglooType) -> &'static [AggOp] {
    use AggOp::*;
    match kind {
        IglooType::Integer | IglooType::Real => &[Sum, Mean, Max, Min],
        IglooType::Boolean => &[Mean, Any, All],
        IglooType::Color | IglooType::Date | IglooType::Time => &[Mean, Max, Min],
        _ => &[],
    }
}

pub fn gen_aggregator(comps: &[Component]) -> TokenStream {
    let agg_enum = gen_enum(comps);
    let agg_new = gen_new(comps);
    let agg_push = gen_push(comps);
    let agg_finish = gen_finish(comps);
    let can_apply = gen_can_apply(comps);

    quote! {
        use std::ops::ControlFlow;

        #agg_enum

        impl Aggregator {
            #agg_new
            #agg_push
            #agg_finish
        }

        #can_apply
    }
}

fn gen_enum(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .filter(|c| is_aggregatable(c))
        .flat_map(|comp| {
            let comp_name = &comp.name;
            match &comp.kind {
                ComponentKind::Single { kind } => ops_for_kind(kind)
                    .iter()
                    .map(|op| {
                        let variant_name = variant_ident(comp_name, *op);
                        let fields = variant_fields(kind, *op);
                        quote! { #variant_name { #fields } }
                    })
                    .collect::<Vec<_>>(),
                ComponentKind::Enum { variants, .. } => {
                    let variant_name = variant_ident(comp_name, AggOp::Mean);
                    let n = variants.len();
                    vec![quote! { #variant_name { counts: [usize; #n], total: usize } }]
                }
                ComponentKind::Marker { .. } => vec![],
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone)]
        pub enum Aggregator {
            #(#variants,)*
        }
    }
}

fn gen_new(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .filter(|c| is_aggregatable(c))
        .flat_map(|comp| {
            let comp_name = ident(&comp.name);
            match &comp.kind {
                ComponentKind::Single { kind } => ops_for_kind(kind)
                    .iter()
                    .map(|op| {
                        let variant_name = variant_ident(&comp.name, *op);
                        let op_ident = op.ident();
                        let init = initial_state(kind, *op);
                        quote! {
                            (ComponentType::#comp_name, AggregationOp::#op_ident) =>
                                Some(Aggregator::#variant_name { #init })
                        }
                    })
                    .collect::<Vec<_>>(),
                ComponentKind::Enum { variants, .. } => {
                    let variant_name = variant_ident(&comp.name, AggOp::Mean);
                    let n = variants.len();
                    vec![quote! {
                        (ComponentType::#comp_name, AggregationOp::Mean) =>
                            Some(Aggregator::#variant_name { counts: [0; #n], total: 0 })
                    }]
                }
                ComponentKind::Marker { .. } => vec![],
            }
        })
        .collect();

    quote! {
        pub fn new(comp_type: ComponentType, op: AggregationOp) -> Option<Self> {
            match (comp_type, op) {
                #(#arms,)*
                _ => None,
            }
        }
    }
}

fn gen_push(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .filter(|c| is_aggregatable(c))
        .flat_map(|comp| {
            let comp_name = ident(&comp.name);
            match &comp.kind {
                ComponentKind::Single { kind } => ops_for_kind(kind)
                    .iter()
                    .map(|op| {
                        let variant_name = variant_ident(&comp.name, *op);
                        let bindings = field_bindings(kind, *op);
                        let logic = push_logic(&comp_name, kind, *op);
                        quote! {
                            Aggregator::#variant_name { #bindings } => { #logic }
                        }
                    })
                    .collect::<Vec<_>>(),
                ComponentKind::Enum { variants, .. } => {
                    let variant_name = variant_ident(&comp.name, AggOp::Mean);
                    let idx_arms: Vec<_> = variants
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let vname = ident(&v.name);
                            quote! { #comp_name::#vname => #i }
                        })
                        .collect();

                    vec![quote! {
                        Aggregator::#variant_name { counts, total } => {
                            if let Component::#comp_name(v) = comp {
                                let idx = match v { #(#idx_arms,)* };
                                counts[idx] += 1;
                                *total += 1;
                            }
                            ControlFlow::Continue(())
                        }
                    }]
                }
                ComponentKind::Marker { .. } => vec![],
            }
        })
        .collect();

    quote! {
        pub fn push(&mut self, comp: &Component) -> ControlFlow<()> {
            match self {
                #(#arms)*
            }
        }
    }
}

fn gen_finish(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .filter(|c| is_aggregatable(c))
        .flat_map(|comp| {
            let comp_name = ident(&comp.name);
            match &comp.kind {
                ComponentKind::Single { kind } => ops_for_kind(kind)
                    .iter()
                    .map(|op| {
                        let variant_name = variant_ident(&comp.name, *op);
                        let bindings = field_bindings(kind, *op);
                        let logic = finish_logic(kind, *op);
                        quote! {
                            Aggregator::#variant_name { #bindings } => { #logic }
                        }
                    })
                    .collect::<Vec<_>>(),
                ComponentKind::Enum { variants, .. } => {
                    let variant_name = variant_ident(&comp.name, AggOp::Mean);
                    let n = variants.len();
                    let from_idx_arms: Vec<_> = variants
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let vname = ident(&v.name);
                            quote! { #i => #comp_name::#vname }
                        })
                        .collect();

                    vec![quote! {
                        Aggregator::#variant_name { counts, total } => {
                            if total == 0 {
                                return None;
                            }
                            let mut max_idx = 0;
                            for i in 1..#n {
                                if counts[i] > counts[max_idx] {
                                    max_idx = i;
                                }
                            }
                            let result = match max_idx {
                                #(#from_idx_arms,)*
                                _ => unreachable!(),
                            };
                            Some(IglooValue::Enum(IglooEnumValue::#comp_name(result)))
                        }
                    }]
                }
                ComponentKind::Marker { .. } => vec![],
            }
        })
        .collect();

    quote! {
        pub fn finish(self) -> Option<IglooValue> {
            match self {
                #(#arms)*
            }
        }
    }
}

fn gen_can_apply(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .filter_map(|comp| {
            let name = ident(&comp.name);
            let ops = match &comp.kind {
                ComponentKind::Single { kind } => match kind {
                    IglooType::Integer | IglooType::Real => quote! { Sum | Mean | Max | Min },
                    IglooType::Boolean => quote! { Mean | Any | All },
                    IglooType::Color | IglooType::Date | IglooType::Time => {
                        quote! { Mean | Max | Min }
                    }
                    _ => return None,
                },
                ComponentKind::Enum { .. } => quote! { Mean },
                ComponentKind::Marker { .. } => return None,
            };
            Some(quote! { ComponentType::#name => matches!(self, #ops) })
        })
        .collect();

    quote! {
        impl AggregationOp {
            pub fn can_apply(&self, comp_type: &ComponentType) -> bool {
                use AggregationOp::*;
                match comp_type {
                    #(#arms,)*
                    _ => false,
                }
            }
        }
    }
}

fn variant_ident(comp_name: &str, op: AggOp) -> Ident {
    let suffix = match op {
        AggOp::Sum => "Sum",
        AggOp::Mean => "Mean",
        AggOp::Max => "Max",
        AggOp::Min => "Min",
        AggOp::Any => "Any",
        AggOp::All => "All",
    };
    ident(&format!("{}{}", comp_name, suffix))
}

fn variant_fields(kind: &IglooType, op: AggOp) -> TokenStream {
    use AggOp::*;
    match (kind, op) {
        (IglooType::Integer, Sum) => quote! { sum: i64 },
        (IglooType::Integer, Mean) => quote! { sum: i64, count: usize },
        (IglooType::Integer, Max | Min) => quote! { val: Option<i64> },

        (IglooType::Real, Sum) => quote! { sum: f64 },
        (IglooType::Real, Mean) => quote! { sum: f64, count: usize },
        (IglooType::Real, Max | Min) => quote! { val: Option<f64> },

        (IglooType::Boolean, Mean) => quote! { true_count: usize, total: usize },
        (IglooType::Boolean, Any) => quote! { found: bool },
        (IglooType::Boolean, All) => quote! { all_true: bool },

        (IglooType::Color, Mean) => quote! { sum_r: f64, sum_g: f64, sum_b: f64, count: usize },
        (IglooType::Color, Max | Min) => quote! { val: Option<IglooColor> },

        (IglooType::Date, Mean) => quote! { sum: i64, count: usize },
        (IglooType::Date, Max | Min) => quote! { val: Option<IglooDate> },

        (IglooType::Time, Mean) => quote! { sum: i64, count: usize },
        (IglooType::Time, Max | Min) => quote! { val: Option<IglooTime> },

        _ => quote! {},
    }
}

fn field_bindings(kind: &IglooType, op: AggOp) -> TokenStream {
    use AggOp::*;
    match (kind, op) {
        (IglooType::Integer | IglooType::Real, Sum) => quote! { sum },
        (IglooType::Integer | IglooType::Real, Mean) => quote! { sum, count },
        (IglooType::Integer | IglooType::Real, Max | Min) => quote! { val },

        (IglooType::Boolean, Mean) => quote! { true_count, total },
        (IglooType::Boolean, Any) => quote! { found },
        (IglooType::Boolean, All) => quote! { all_true },

        (IglooType::Color, Mean) => quote! { sum_r, sum_g, sum_b, count },
        (IglooType::Color, Max | Min) => quote! { val },

        (IglooType::Date | IglooType::Time, Mean) => quote! { sum, count },
        (IglooType::Date | IglooType::Time, Max | Min) => quote! { val },

        _ => quote! {},
    }
}

fn push_logic(comp_name: &Ident, kind: &IglooType, op: AggOp) -> TokenStream {
    use AggOp::*;
    match (kind, op) {
        (IglooType::Integer, Sum) => quote! {
            if let Component::#comp_name(v) = comp { *sum += v; }
            ControlFlow::Continue(())
        },
        (IglooType::Integer, Mean) => quote! {
            if let Component::#comp_name(v) = comp {
                *sum += v;
                *count += 1;
            }
            ControlFlow::Continue(())
        },
        (IglooType::Integer, Max) => quote! {
            if let Component::#comp_name(v) = comp {
                *val = Some(match *val {
                    None => *v,
                    Some(m) => if *v > m { *v } else { m },
                });
            }
            ControlFlow::Continue(())
        },
        (IglooType::Integer, Min) => quote! {
            if let Component::#comp_name(v) = comp {
                *val = Some(match *val {
                    None => *v,
                    Some(m) => if *v < m { *v } else { m },
                });
            }
            ControlFlow::Continue(())
        },

        (IglooType::Real, Sum) => quote! {
            if let Component::#comp_name(v) = comp { *sum += v; }
            ControlFlow::Continue(())
        },
        (IglooType::Real, Mean) => quote! {
            if let Component::#comp_name(v) = comp {
                *sum += v;
                *count += 1;
            }
            ControlFlow::Continue(())
        },
        (IglooType::Real, Max) => quote! {
            if let Component::#comp_name(v) = comp {
                *val = Some(match *val {
                    None => *v,
                    Some(m) => match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                        Ordering::Greater => *v,
                        _ => m,
                    },
                });
            }
            ControlFlow::Continue(())
        },
        (IglooType::Real, Min) => quote! {
            if let Component::#comp_name(v) = comp {
                *val = Some(match *val {
                    None => *v,
                    Some(m) => match v.partial_cmp(&m).unwrap_or(Ordering::Equal) {
                        Ordering::Less => *v,
                        _ => m,
                    },
                });
            }
            ControlFlow::Continue(())
        },

        (IglooType::Boolean, Mean) => quote! {
            if let Component::#comp_name(v) = comp {
                if *v { *true_count += 1; }
                *total += 1;
            }
            ControlFlow::Continue(())
        },
        (IglooType::Boolean, Any) => quote! {
            if let Component::#comp_name(true) = comp {
                *found = true;
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        },
        (IglooType::Boolean, All) => quote! {
            if let Component::#comp_name(false) = comp {
                *all_true = false;
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        },

        (IglooType::Color, Mean) => quote! {
            if let Component::#comp_name(c) = comp {
                *sum_r += c.r;
                *sum_g += c.g;
                *sum_b += c.b;
                *count += 1;
            }
            ControlFlow::Continue(())
        },
        (IglooType::Color, Max) => quote! {
            if let Component::#comp_name(c) = comp {
                *val = Some(match val {
                    None => c.clone(),
                    Some(m) => {
                        let cmp = (c.r, c.g, c.b)
                            .partial_cmp(&(m.r, m.g, m.b))
                            .unwrap_or(Ordering::Equal);
                        if cmp == Ordering::Greater { c.clone() } else { m.clone() }
                    }
                });
            }
            ControlFlow::Continue(())
        },
        (IglooType::Color, Min) => quote! {
            if let Component::#comp_name(c) = comp {
                *val = Some(match val {
                    None => c.clone(),
                    Some(m) => {
                        let cmp = (c.r, c.g, c.b)
                            .partial_cmp(&(m.r, m.g, m.b))
                            .unwrap_or(Ordering::Equal);
                        if cmp == Ordering::Less { c.clone() } else { m.clone() }
                    }
                });
            }
            ControlFlow::Continue(())
        },

        (IglooType::Date, Mean) => quote! {
            if let Component::#comp_name(d) = comp {
                *sum += d.days_since_epoch() as i64;
                *count += 1;
            }
            ControlFlow::Continue(())
        },
        (IglooType::Date, Max) => quote! {
            if let Component::#comp_name(d) = comp {
                *val = Some(match *val {
                    None => d.clone(),
                    Some(m) => if d.days_since_epoch() > m.days_since_epoch() { d.clone() } else { m },
                });
            }
            ControlFlow::Continue(())
        },
        (IglooType::Date, Min) => quote! {
            if let Component::#comp_name(d) = comp {
                *val = Some(match *val {
                    None => d.clone(),
                    Some(m) => if d.days_since_epoch() < m.days_since_epoch() { d.clone() } else { m },
                });
            }
            ControlFlow::Continue(())
        },

        (IglooType::Time, Mean) => quote! {
            if let Component::#comp_name(t) = comp {
                *sum += t.to_seconds() as i64;
                *count += 1;
            }
            ControlFlow::Continue(())
        },
        (IglooType::Time, Max) => quote! {
            if let Component::#comp_name(t) = comp {
                *val = Some(match *val {
                    None => t.clone(),
                    Some(m) => if t.to_seconds() > m.to_seconds() { t.clone() } else { m },
                });
            }
            ControlFlow::Continue(())
        },
        (IglooType::Time, Min) => quote! {
            if let Component::#comp_name(t) = comp {
                *val = Some(match *val {
                    None => t.clone(),
                    Some(m) => if t.to_seconds() < m.to_seconds() { t.clone() } else { m },
                });
            }
            ControlFlow::Continue(())
        },

        _ => quote! { ControlFlow::Continue(()) },
    }
}

fn finish_logic(kind: &IglooType, op: AggOp) -> TokenStream {
    use AggOp::*;
    match (kind, op) {
        (IglooType::Integer, Sum) => quote! { Some(IglooValue::Integer(sum)) },
        (IglooType::Integer, Mean) => quote! {
            (count > 0).then(|| IglooValue::Integer(sum / count as i64))
        },
        (IglooType::Integer, Max | Min) => quote! { val.map(IglooValue::Integer) },

        (IglooType::Real, Sum) => quote! { Some(IglooValue::Real(sum)) },
        (IglooType::Real, Mean) => quote! {
            (count > 0).then(|| IglooValue::Real(sum / count as f64))
        },
        (IglooType::Real, Max | Min) => quote! { val.map(IglooValue::Real) },

        (IglooType::Boolean, Mean) => quote! {
            (total > 0).then(|| IglooValue::Boolean(true_count * 2 >= total))
        },
        (IglooType::Boolean, Any) => quote! { Some(IglooValue::Boolean(found)) },
        (IglooType::Boolean, All) => quote! { Some(IglooValue::Boolean(all_true)) },

        (IglooType::Color, Mean) => quote! {
            (count > 0).then(|| IglooValue::Color(IglooColor {
                r: sum_r / count as f64,
                g: sum_g / count as f64,
                b: sum_b / count as f64,
            }))
        },
        (IglooType::Color, Max | Min) => quote! { val.map(IglooValue::Color) },

        (IglooType::Date, Mean) => quote! {
            (count > 0).then(|| IglooValue::Date(
                IglooDate::from_days_since_epoch((sum / count as i64) as i32)
            ))
        },
        (IglooType::Date, Max | Min) => quote! { val.map(IglooValue::Date) },

        (IglooType::Time, Mean) => quote! {
            (count > 0).then(|| IglooValue::Time(
                IglooTime::from_seconds((sum / count as i64) as i32)
            ))
        },
        (IglooType::Time, Max | Min) => quote! { val.map(IglooValue::Time) },

        _ => quote! { None },
    }
}

fn initial_state(kind: &IglooType, op: AggOp) -> TokenStream {
    use AggOp::*;
    match (kind, op) {
        (IglooType::Integer, Sum) => quote! { sum: 0 },
        (IglooType::Integer, Mean) => quote! { sum: 0, count: 0 },
        (IglooType::Integer, Max | Min) => quote! { val: None },

        (IglooType::Real, Sum) => quote! { sum: 0.0 },
        (IglooType::Real, Mean) => quote! { sum: 0.0, count: 0 },
        (IglooType::Real, Max | Min) => quote! { val: None },

        (IglooType::Boolean, Mean) => quote! { true_count: 0, total: 0 },
        (IglooType::Boolean, Any) => quote! { found: false },
        (IglooType::Boolean, All) => quote! { all_true: true },

        (IglooType::Color, Mean) => quote! { sum_r: 0.0, sum_g: 0.0, sum_b: 0.0, count: 0 },
        (IglooType::Color, Max | Min) => quote! { val: None },

        (IglooType::Date | IglooType::Time, Mean) => quote! { sum: 0, count: 0 },
        (IglooType::Date | IglooType::Time, Max | Min) => quote! { val: None },

        _ => quote! {},
    }
}

fn is_aggregatable(comp: &Component) -> bool {
    match &comp.kind {
        ComponentKind::Single { kind } => matches!(
            kind,
            IglooType::Integer
                | IglooType::Real
                | IglooType::Boolean
                | IglooType::Color
                | IglooType::Date
                | IglooType::Time
        ),
        ComponentKind::Enum { .. } => true,
        ComponentKind::Marker { .. } => false,
    }
}
