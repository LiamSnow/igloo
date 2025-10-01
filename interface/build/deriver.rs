use super::model::*;

pub fn add_derived_comps(comps: &mut Vec<Component>) {
    let mut result = Vec::with_capacity(comps.len());

    for mut comp in comps.drain(..) {
        let mut new_comps = Vec::new();

        if let ComponentKind::Single {
            field,
            derive_bound_types,
            derive_inner_list_type,
            derive_string_bound_types,
            ..
        } = &comp.kind
        {
            if let Some(ids) = derive_bound_types {
                add_derived_bound_types(&mut new_comps, &comp, field, ids);
            }
            if let Some(id) = derive_inner_list_type {
                add_derived_inner_list_type(&mut new_comps, &comp, field, *id);
            }
            if let Some(ids) = derive_string_bound_types {
                add_derived_string_bound_types(&mut new_comps, &comp, field, ids);
            }
        }

        if let Some(id) = comp.derive_supported_type {
            add_derived_supported_type(&mut new_comps, &mut comp, id);
        }
        if let Some(id) = comp.derive_list_type {
            add_derived_list_type(&mut new_comps, &comp, id);
        }

        result.push(comp);
        result.append(&mut new_comps);
    }

    *comps = result;
}

fn add_derived_bound_types(
    new_comps: &mut Vec<Component>,
    comp: &Component,
    field: &str,
    ids: &[u16; 3],
) {
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
            desc: "Marks a bound. Only enforced by dashboard components that use it.".to_string(),
            derive_supported_type: None,
            derive_list_type: None,
            kind: ComponentKind::single(field.to_string()),
            related: vec![Related {
                name: comp.name.clone(),
                reason,
            }],
        });
    }
}

fn add_derived_inner_list_type(
    new_comps: &mut Vec<Component>,
    comp: &Component,
    field: &str,
    id: u16,
) {
    new_comps.push(Component {
        name: format!("{}List", comp.name),
        id,
        desc: format!("a variable-length list of {}", field),
        related: Vec::new(),
        derive_supported_type: None,
        derive_list_type: None,
        kind: ComponentKind::single(format!("Vec<{}>", field)),
    });
}

fn add_derived_string_bound_types(
    new_comps: &mut Vec<Component>,
    comp: &Component,
    field: &str,
    ids: &[u16; 3],
) {
    if field != "String" {
        panic!("Cannot implement String bound types for a non-String {field}")
    }

    for (i, suffix) in ["MaxLength", "MinLength", "Pattern"].iter().enumerate() {
        let new_name = format!("{}{}", comp.name, suffix);
        let (reason, field_type) = match *suffix {
            "MaxLength" => ("sets a max length bound on".to_string(), "u32".to_string()),
            "MinLength" => ("sets a min length bound on".to_string(), "u32".to_string()),
            "Pattern" => ("set a regex pattern for".to_string(), "String".to_string()),
            _ => unreachable!(),
        };

        new_comps.push(Component {
            name: new_name,
            id: ids[i],
            desc: "Marks a requirement. Only enforced by dashbaord components that use it."
                .to_string(),
            derive_supported_type: None,
            derive_list_type: None,
            kind: ComponentKind::single(field_type),
            related: vec![Related {
                name: comp.name.clone(),
                reason,
            }],
        });
    }
}

fn add_derived_supported_type(new_comps: &mut Vec<Component>, comp: &mut Component, id: u16) {
    new_comps.push(Component {
        name: format!("Supported{}s", comp.name),
        id,
        desc: format!("specifies what {}s are supported by this entity", comp.name),
        related: Vec::new(),
        derive_supported_type: None,
        derive_list_type: None,
        kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
    });

    comp.related.push(Related {
        name: format!("Supported{}s", comp.name),
        reason: "specifies what is supported by the entity".to_string(),
    });
}

fn add_derived_list_type(new_comps: &mut Vec<Component>, comp: &Component, id: u16) {
    new_comps.push(Component {
        name: format!("{}List", comp.name),
        id,
        desc: format!("A list of {}", comp.name),
        related: Vec::new(),
        derive_supported_type: None,
        derive_list_type: None,
        kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
    });
}

impl ComponentKind {
    fn single(field: String) -> Self {
        ComponentKind::Single {
            field,
            derive_bound_types: None,
            derive_inner_list_type: None,
            derive_string_bound_types: None,
        }
    }
}
