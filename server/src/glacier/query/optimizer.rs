use igloo_interface::{
    ComponentType,
    query::{
        DeviceFilter as D, EntityFilter as E, FloeFilter as F, GroupFilter as G, Query,
        QueryAction, QueryTarget,
    },
};
use std::mem;

pub const AVG_ENTITIES_PER_DEVICE: usize = 8;
pub const AVG_DEVICES_PER_FLOE: usize = 8;
pub const AVG_DEVICES_PER_GROUP: usize = 8;

pub trait QueryOptimizer {
    fn optimize(&mut self);
}

impl QueryOptimizer for Query {
    fn optimize(&mut self) {
        // HasAll is an optimization that allows for early exit
        // For example, if we are looking for Component::Dimmer
        // We can ignore devices that dont have any Dimmers
        // Furthermore we can group entity filters And([Has(_), Has(_)]) into HasAll([_, _])
        // TODO FIXME required_precence is bad, we could have Or([Type(_), Type(_)])
        // inside a component_filter. Need to be able to extract that entire Op
        let mut has_all = Vec::with_capacity(10);

        // every component action (but put) requires that
        // that components exists on the entity already
        if !matches!(self.action, QueryAction::Put(_))
            && let QueryTarget::Components(comp_type) = self.target
        {
            has_all.push(comp_type);
        }

        if self
            .entity_filter
            .as_mut()
            .is_some_and(|f| f.optimize(&mut has_all))
        {
            self.entity_filter = None;
        }

        if let Some(f) = &mut self.device_filter {
            f.optimize(&mut has_all);
        }

        // floe & groups have independent subtrees so
        // they dont change has_all
        if let Some(f) = &mut self.group_filter {
            f.optimize();
        }

        if let Some(f) = &mut self.floe_filter {
            f.optimize();
        }

        if !has_all.is_empty() {
            self.device_filter =
                merge_device_filter(self.device_filter.take(), D::HasAll(has_all.clone()));
            self.entity_filter = merge_entity_filter(self.entity_filter.take(), E::HasAll(has_all));
        }
    }
}

#[inline]
fn merge_device_filter(existing: Option<D>, new: D) -> Option<D> {
    match existing {
        Some(D::All(mut vec)) => {
            vec.push(new);
            Some(D::All(vec))
        }
        Some(e) => Some(D::All(vec![e, new])),
        None => Some(new),
    }
}

#[inline]
fn merge_entity_filter(existing: Option<E>, new: E) -> Option<E> {
    match existing {
        Some(E::All(mut vec)) => {
            vec.push(new);
            Some(E::All(vec))
        }
        Some(e) => Some(E::All(vec![e, new])),
        None => Some(new),
    }
}

trait EExt {
    fn optimize(&mut self, has_all: &mut Vec<ComponentType>) -> bool;
    fn cost(&self) -> usize;
}

impl EExt for E {
    /// returns true if filter should be dropped
    fn optimize(&mut self, has_all: &mut Vec<ComponentType>) -> bool {
        match self {
            E::Has(comp_type) => {
                if !has_all.contains(comp_type) {
                    has_all.push(*comp_type);
                }
                // drop this filter, since it will be added to HasAll
                true
            }

            E::Condition(_, component) => {
                let comp_type = component.get_type();
                if !has_all.contains(&comp_type) {
                    has_all.push(comp_type);
                }
                false
            }

            E::All(filters) => {
                // optimize all children, drop those that requested
                filters.retain_mut(|f| !f.optimize(has_all));

                // sort by cost, so those can early exits earlier (lol)
                if filters.len() == 2 {
                    if filters[0].cost() > filters[1].cost() {
                        filters.swap(0, 1);
                    }
                } else if filters.len() > 2 {
                    let mut with_costs: Vec<(usize, _)> =
                        filters.drain(..).map(|f| (f.cost(), f)).collect();
                    with_costs.sort_unstable_by_key(|(cost, _)| *cost);
                    *filters = with_costs.into_iter().map(|(_, f)| f).collect();
                }
                false
            }

            E::Any(filters) => {
                // optimize all children, drop those that requested
                // currently does not contribute to global has_all
                // since Any([Has(_), Has(_), _]) cannot be extracted
                let mut has_any = Vec::with_capacity(10);
                filters.retain_mut(|f| !f.optimize(&mut has_any));
                if !has_any.is_empty() {
                    filters.push(E::HasAny(has_any));
                }

                // sort by cost, so those can early exits earlier (lol)
                if filters.len() == 2 {
                    if filters[0].cost() > filters[1].cost() {
                        filters.swap(0, 1);
                    }
                } else if filters.len() > 2 {
                    let mut with_costs: Vec<(usize, _)> =
                        filters.drain(..).map(|f| (f.cost(), f)).collect();
                    with_costs.sort_unstable_by_key(|(cost, _)| *cost);
                    *filters = with_costs.into_iter().map(|(_, f)| f).collect();
                }
                false
            }

            E::Not(inner) => {
                // cant extract types
                // eventually (just like Any) should be able to extract this
                // into a Not(HasAll(_)) or something
                inner.optimize(&mut Vec::with_capacity(10));
                false
            }

            _ => false,
        }
    }

    #[inline]
    fn cost(&self) -> usize {
        match self {
            E::Has(_) => 1,
            E::UpdatedWithinSeconds(_) => 1,
            E::ComponentCount(_, _) => 1,
            E::HasAll(v) => v.len().max(2),
            E::HasAny(v) => v.len().max(2),
            E::Condition(_, _) => 15,
            E::NameEquals(name) => name.len().max(5),
            E::NameMatches(_) => usize::MAX,
            E::All(filters) | E::Any(filters) => {
                1 + filters.iter().map(|f| f.cost()).sum::<usize>()
            }
            E::Not(inner) => 1 + inner.cost(),
        }
    }
}

trait DExt {
    fn optimize(&mut self, has_all: &mut Vec<ComponentType>);
    fn cost(&self) -> usize;
}

impl DExt for D {
    fn optimize(&mut self, has_all: &mut Vec<ComponentType>) {
        match self {
            D::HasEntity(entity_filter) => {
                // TODO need to drop entity filter if true
                entity_filter.optimize(has_all);
            }

            D::AllEntities(entity_filter) => {
                // TODO need to drop entity filter if true
                entity_filter.optimize(has_all);
            }

            D::All(filters) => {
                for filter in filters.iter_mut() {
                    filter.optimize(has_all);
                }

                if filters.len() == 2 {
                    if filters[0].cost() > filters[1].cost() {
                        filters.swap(0, 1);
                    }
                } else if filters.len() > 2 {
                    let mut with_costs: Vec<(usize, _)> =
                        filters.drain(..).map(|f| (f.cost(), f)).collect();
                    with_costs.sort_unstable_by_key(|(cost, _)| *cost);
                    *filters = with_costs.into_iter().map(|(_, f)| f).collect();
                }
            }

            D::Any(filters) => {
                for filter in filters.iter_mut() {
                    filter.optimize(has_all);
                }

                if filters.len() == 2 {
                    if filters[0].cost() > filters[1].cost() {
                        filters.swap(0, 1);
                    }
                } else if filters.len() > 2 {
                    let mut with_costs: Vec<(usize, _)> =
                        filters.drain(..).map(|f| (f.cost(), f)).collect();
                    with_costs.sort_unstable_by_key(|(cost, _)| *cost);
                    *filters = with_costs.into_iter().map(|(_, f)| f).collect();
                }
            }

            D::Not(inner) => {
                inner.optimize(has_all);
            }

            _ => {}
        }
    }

    #[inline]
    fn cost(&self) -> usize {
        match self {
            D::Id(_) => 1,
            D::UpdatedWithinSeconds(_) => 1,
            D::EntityCount(_, _) => 1,
            D::HasAll(v) => v.len().max(2),
            D::Ids(ids) => ids.len().max(2),
            D::NameEquals(name) => name.len().max(5),
            D::NameMatches(_) => usize::MAX,
            D::HasEntity(f) => AVG_ENTITIES_PER_DEVICE * f.cost(),
            D::AllEntities(f) => AVG_ENTITIES_PER_DEVICE * f.cost(),
            D::All(f) | D::Any(f) => 1 + f.iter().map(|f| f.cost()).sum::<usize>(),
            D::Not(inner) => 1 + inner.cost(),
        }
    }
}

trait GExt {
    fn optimize(&mut self);
    fn cost(&self) -> usize;
}

impl GExt for G {
    fn optimize(&mut self) {
        match self {
            G::HasDevice(device_filter) | G::AllDevices(device_filter) => {
                // doesn't contribute to global has_all
                // because this isn't filtering down devices its
                // filtering groups
                //
                // for example G::HasDevice(Name("liam")) + D::HasDevice(Name("max"))
                // is saying exclude devices in groups without any devices named "liam",
                // then from there filter out devices not named "max"
                let mut has_all = Vec::with_capacity(10);
                device_filter.optimize(&mut has_all);

                if !has_all.is_empty() {
                    let existing = mem::replace(device_filter, D::Id(Default::default()));
                    *device_filter = match existing {
                        D::All(mut vec) => {
                            vec.push(D::HasAll(has_all));
                            D::All(vec)
                        }
                        _ => D::All(vec![D::HasAll(has_all), existing]),
                    };
                }
            }

            G::All(filters) | G::Any(filters) => {
                for filter in filters.iter_mut() {
                    filter.optimize();
                }

                if filters.len() == 2 {
                    if filters[0].cost() > filters[1].cost() {
                        filters.swap(0, 1);
                    }
                } else if filters.len() > 2 {
                    let mut with_costs: Vec<(usize, _)> =
                        filters.drain(..).map(|f| (f.cost(), f)).collect();
                    with_costs.sort_unstable_by_key(|(cost, _)| *cost);
                    *filters = with_costs.into_iter().map(|(_, f)| f).collect();
                }
            }

            G::Not(inner) => {
                inner.optimize();
            }

            _ => {}
        }
    }

    #[inline]
    fn cost(&self) -> usize {
        match self {
            G::Id(_) => 1,
            G::DeviceCount(_, _) => 1,
            G::Ids(ids) => ids.len().max(2),
            G::NameEquals(name) => name.len().max(5),
            G::NameMatches(_) => usize::MAX,
            G::HasDevice(device_filter) => AVG_DEVICES_PER_GROUP * device_filter.cost(),
            G::AllDevices(device_filter) => AVG_DEVICES_PER_GROUP * device_filter.cost(),
            G::All(filters) | G::Any(filters) => {
                1 + filters.iter().map(|f| f.cost()).sum::<usize>()
            }
            G::Not(inner) => 1 + inner.cost(),
        }
    }
}

trait FExt {
    fn optimize(&mut self);
    fn cost(&self) -> usize;
}

impl FExt for F {
    fn optimize(&mut self) {
        match self {
            F::HasDevice(device_filter) | F::AllDevices(device_filter) => {
                // see commonet above on G::HasDevice
                let mut has_all = Vec::with_capacity(10);
                device_filter.optimize(&mut has_all);

                if !has_all.is_empty() {
                    let existing = mem::replace(device_filter, D::Id(Default::default()));
                    *device_filter = match existing {
                        D::All(mut vec) => {
                            vec.push(D::HasAll(has_all));
                            D::All(vec)
                        }
                        _ => D::All(vec![D::HasAll(has_all), existing]),
                    };
                }
            }

            F::All(filters) | F::Any(filters) => {
                for filter in filters.iter_mut() {
                    filter.optimize();
                }

                if filters.len() == 2 {
                    if filters[0].cost() > filters[1].cost() {
                        filters.swap(0, 1);
                    }
                } else if filters.len() > 2 {
                    let mut with_costs: Vec<(usize, _)> =
                        filters.drain(..).map(|f| (f.cost(), f)).collect();
                    with_costs.sort_unstable_by_key(|(cost, _)| *cost);
                    *filters = with_costs.into_iter().map(|(_, f)| f).collect();
                }
            }

            F::Not(inner) => {
                inner.optimize();
            }

            _ => {}
        }
    }

    #[inline]
    fn cost(&self) -> usize {
        match self {
            F::Id(_) => 1,
            F::DeviceCount(_, _) => 1,
            F::Ids(ids) => ids.len().max(2),
            F::IdMatches(_) => usize::MAX,
            F::HasDevice(device_filter) => AVG_DEVICES_PER_FLOE * device_filter.cost(),
            F::AllDevices(device_filter) => AVG_DEVICES_PER_FLOE * device_filter.cost(),
            F::All(filters) | F::Any(filters) => {
                1 + filters.iter().map(|f| f.cost()).sum::<usize>()
            }
            F::Not(inner) => 1 + inner.cost(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use igloo_interface::{
        Component, ComponentType as T,
        id::DeviceID,
        query::{
            DeviceFilter as D, EntityFilter as E, FloeFilter as F, GroupFilter as G, QueryAction,
            QueryTarget,
        },
        types::compare::ComparisonOp,
    };
    use std::time::Instant;

    #[test]
    fn simple_comp_type_extraction() {
        println!("\n == simple_comp_type_extraction ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Components(T::Dimmer),
            floe_filter: None,
            group_filter: None,
            device_filter: None,
            entity_filter: None,
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        match &query.device_filter {
            Some(D::HasAll(set)) => {
                assert!(
                    set.contains(&T::Dimmer),
                    "Device filter should have Dimmer present"
                );
            }
            _ => panic!("Expected D::HasAll, got: {:?}", query.device_filter),
        }

        match &query.entity_filter {
            Some(E::HasAll(set)) => {
                assert!(
                    set.contains(&T::Dimmer),
                    "Entity filter should have Dimmer present"
                );
            }
            _ => panic!("Expected E::HasAll, got: {:?}", query.entity_filter),
        }
    }

    #[test]
    fn entity_has_optimization() {
        println!("\n == entity_has_optimization ==");
        let mut query = Query {
            action: QueryAction::Count,
            target: QueryTarget::Entities,
            floe_filter: None,
            group_filter: None,
            device_filter: None,
            entity_filter: Some(E::All(vec![E::Has(T::Light), E::Has(T::Color)])),
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        match &query.device_filter {
            Some(D::HasAll(set)) => {
                assert!(
                    set.contains(&T::Light) && set.contains(&T::Color),
                    "Should have both Light and Color"
                );
            }
            _ => panic!(
                "Expected device filter with types, got: {:?}",
                query.device_filter
            ),
        }

        match &query.entity_filter {
            Some(E::All(filters)) => {
                assert!(
                    filters.len() >= 2,
                    "Entity filter should have And with filters"
                );
            }
            _ => panic!("Expected entity filter, got: {:?}", query.entity_filter),
        }
    }

    #[test]
    fn complex_comp_type_extraction() {
        println!("\n == complex_comp_type_extraction ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Components(T::Dimmer),
            floe_filter: None,
            group_filter: None,
            device_filter: None,
            entity_filter: Some(E::Condition(ComparisonOp::Gte, Component::Dimmer(0.5))),
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        match &query.device_filter {
            Some(D::HasAll(set)) => {
                assert!(
                    set.contains(&T::Dimmer),
                    "Device should have Dimmer present"
                );
            }
            _ => panic!("Expected D::HasAll, got: {:?}", query.device_filter),
        }

        match &query.entity_filter {
            Some(E::All(filters)) => {
                assert!(filters.len() >= 2, "Should have at least 2 filters");
            }
            _ => panic!("Expected E::All, got: {:?}", query.entity_filter),
        }
    }

    #[test]
    fn complex_comp_type_extraction_2() {
        println!("\n == complex_comp_type_extraction_2 ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Devices,
            floe_filter: None,
            group_filter: None,
            device_filter: None,
            entity_filter: Some(E::All(vec![
                E::Any(vec![E::Has(T::Light), E::Has(T::Color)]),
                E::Condition(ComparisonOp::Gte, Component::Dimmer(0.5)),
            ])),
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        match &query.device_filter {
            Some(D::HasAll(set)) => {
                assert!(
                    set.contains(&T::Dimmer),
                    "Should have Dimmer (from Condition)"
                );
                assert!(
                    !set.contains(&T::Light) && !set.contains(&T::Color),
                    "Should not extract types from Or"
                );
            }
            _ => panic!(
                "Expected device filter with types, got: {:?}",
                query.device_filter
            ),
        }
    }

    #[test]
    fn no_comp_type_or_extraction() {
        println!("\n == no_comp_type_or_extraction ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Devices,
            floe_filter: None,
            group_filter: None,
            device_filter: None,
            entity_filter: Some(E::Any(vec![
                E::Has(T::Light),
                E::Condition(ComparisonOp::Gte, Component::Dimmer(0.5)),
            ])),
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        assert_eq!(query.device_filter, None);
        assert!(matches!(query.entity_filter, Some(E::Any(_))));
    }

    #[test]
    fn cost_based_reordering() {
        println!("\n == cost_based_reordering ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Devices,
            floe_filter: None,
            group_filter: None,
            device_filter: Some(D::All(vec![
                D::AllEntities(E::All(vec![
                    E::ComponentCount(ComparisonOp::Gte, 3),
                    E::Not(Box::new(E::UpdatedWithinSeconds(60))),
                ])),
                D::Id(DeviceID::from_comb(42)),
            ])),
            entity_filter: Some(E::Any(vec![
                E::All(vec![
                    E::Any(vec![E::UpdatedWithinSeconds(30), E::Has(T::TextList)]),
                    E::ComponentCount(ComparisonOp::Eq, 2),
                ]),
                E::NameEquals("RGBCT_Bulb".to_string()),
            ])),
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        match &query.device_filter.as_ref().unwrap() {
            D::All(filters) => {
                for i in 0..filters.len().saturating_sub(1) {
                    assert!(
                        filters[i].cost() <= filters[i + 1].cost(),
                        "Filter {} (cost {}) should be <= filter {} (cost {})",
                        i,
                        filters[i].cost(),
                        i + 1,
                        filters[i + 1].cost()
                    );
                }
            }
            _ => panic!("Expected And at device level"),
        }

        match &query.entity_filter.as_ref().unwrap() {
            E::Any(filters) => {
                for i in 0..filters.len().saturating_sub(1) {
                    assert!(
                        filters[i].cost() <= filters[i + 1].cost(),
                        "Filter {} (cost {}) should be <= filter {} (cost {})",
                        i,
                        filters[i].cost(),
                        i + 1,
                        filters[i + 1].cost()
                    );
                }
            }
            _ => panic!("Expected Or at entity level"),
        }
    }

    #[test]
    fn group_filter_optimization() {
        println!("\n == group_filter_optimization ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Groups,
            floe_filter: None,
            group_filter: Some(G::All(vec![
                G::HasDevice(D::HasEntity(E::Has(T::Light))),
                G::NameMatches("*_room_*".to_string()),
            ])),
            device_filter: None,
            entity_filter: None,
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        if let Some(G::All(filters)) = &query.group_filter {
            let has_device_with_present = filters.iter().any(|f| match f {
                G::HasDevice(D::All(dev_filters)) => {
                    dev_filters.iter().any(|df| matches!(df, D::HasAll(_)))
                }
                G::HasDevice(D::HasAll(_)) => true,
                _ => false,
            });
            assert!(
                has_device_with_present,
                "Should have HasAll(Light) extracted in device filter"
            );
        } else {
            panic!("Expected And at group level");
        }
    }

    #[test]
    fn floe_filter_optimization() {
        println!("\n == floe_filter_optimization ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Floes,
            floe_filter: Some(F::All(vec![
                F::IdMatches("prod_*".to_string()),
                F::HasDevice(D::UpdatedWithinSeconds(300)),
            ])),
            group_filter: None,
            device_filter: None,
            entity_filter: None,
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        if let Some(F::All(filters)) = &query.floe_filter {
            for i in 0..filters.len().saturating_sub(1) {
                assert!(
                    filters[i].cost() <= filters[i + 1].cost(),
                    "Filters should be sorted by cost"
                );
            }
        }
    }

    #[test]
    fn multi_level_type_extraction() {
        println!("\n == multi_level_type_extraction ==");
        let mut query = Query {
            action: QueryAction::Count,
            target: QueryTarget::Devices,
            floe_filter: None,
            group_filter: None,
            device_filter: Some(D::HasEntity(E::All(vec![
                E::Has(T::Light),
                E::Has(T::Dimmer),
            ]))),
            entity_filter: None,
            limit: None,
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        match &query.device_filter {
            Some(D::All(filters)) => {
                let has_present = filters.iter().any(|f| matches!(f, D::HasAll(_)));
                let has_entity = filters.iter().any(|f| matches!(f, D::HasEntity(_)));

                assert!(
                    has_present && has_entity,
                    "Expected both HasAll types and HasEntity, got: {:?}",
                    query.device_filter
                );

                if let Some(D::HasAll(set)) = filters.iter().find(|f| matches!(f, D::HasAll(_))) {
                    assert!(set.contains(&T::Light), "Should have Light in HasAll");
                    assert!(set.contains(&T::Dimmer), "Should have Dimmer in HasAll");
                }
            }
            Some(D::HasAll(set)) => {
                assert!(
                    set.contains(&T::Light) && set.contains(&T::Dimmer),
                    "Should have both types"
                );
            }
            _ => panic!(
                "Expected device filter to be And with extracted types, got: {:?}",
                query.device_filter
            ),
        }
    }

    #[test]
    fn benchmark_large_filter_tree() {
        println!("\n == benchmark_large_filter_tree ==");
        let mut query = Query {
            action: QueryAction::Get,
            target: QueryTarget::Components(T::Light),
            floe_filter: None,
            group_filter: None,
            device_filter: Some(D::All(vec![
                D::Any(vec![
                    D::NameMatches("bedroom_*".to_string()),
                    D::NameMatches("living_*".to_string()),
                ]),
                D::All(vec![
                    D::UpdatedWithinSeconds(3600),
                    D::EntityCount(ComparisonOp::Gte, 2),
                ]),
            ])),
            entity_filter: Some(E::All(vec![
                E::Any(vec![
                    E::NameEquals("light".to_string()),
                    E::NameMatches("*_light_*".to_string()),
                ]),
                E::All(vec![
                    E::Has(T::Dimmer),
                    E::Condition(ComparisonOp::Gte, Component::Dimmer(0.1)),
                ]),
            ])),
            limit: Some(100),
            tag: 0,
        };

        let start = Instant::now();
        query.optimize();
        let duration = start.elapsed();
        println!("TOTAL: {:?}", duration);

        assert!(matches!(query.device_filter, Some(D::All(_))));
        assert!(matches!(query.entity_filter, Some(E::All(_))));

        if let Some(D::All(filters)) = &query.device_filter {
            let has_present = filters.iter().any(|f| matches!(f, D::HasAll(_)));
            assert!(has_present, "Should have HasAll in device filter");
        }
    }
}
