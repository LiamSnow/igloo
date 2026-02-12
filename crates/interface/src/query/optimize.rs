use crate::query::{
    ComponentAction, EntityFilter, OneShotQuery, TypeFilter, ValueFilter, WatchQuery,
};

impl OneShotQuery {
    pub fn optimize(&mut self) {
        match self {
            OneShotQuery::Entity(q) => {
                q.entity_filter.optimize();
            }
            OneShotQuery::Component(q) => {
                // every action but Put requires .component to exist
                if !matches!(q.action, ComponentAction::Put(_)) {
                    TypeFilter::add_with(&mut q.entity_filter.type_filter, q.component);
                }

                q.entity_filter.optimize();
            }
            _ => {}
        }
    }
}

impl WatchQuery {
    pub fn optimize(&mut self) {
        match self {
            WatchQuery::Metadata => {}
            WatchQuery::Component(q) => {
                TypeFilter::add_with(&mut q.entity_filter.type_filter, q.component);

                if let Some(tf) = &mut q.entity_filter.type_filter {
                    tf.optimize();
                }
            }
        }
    }
}

impl EntityFilter {
    pub fn optimize(&mut self) {
        // For a value filter to succeed, we need those components
        // to exist, so we add a type_filter which allows us to
        // precheck devices before checking all its entities
        if let Some(vf) = &mut self.value_filter {
            vf.optimize();
            let extra = vf.to_type_filter();

            self.type_filter = Some(match self.type_filter.take() {
                None => extra,
                Some(existing) => TypeFilter::merge_and(existing, extra),
            });
        }

        if let Some(tf) = &mut self.type_filter {
            tf.optimize();

            if matches!(tf, TypeFilter::And(f) | TypeFilter::Or(f) if f.is_empty()) {
                self.type_filter = None;
            }
        }
    }
}

impl ValueFilter {
    pub fn optimize(&mut self) {
        match self {
            ValueFilter::And(filters) | ValueFilter::Or(filters) => {
                for filter in filters.iter_mut() {
                    filter.optimize();
                }

                self.flatten_local();
                self.normalize_local();
                self.reorder_local();
            }
            ValueFilter::Not(inner) => {
                inner.optimize();
            }
            ValueFilter::If(_, _) => {}
        }
    }

    /// And([And([a, b], And([c, d]))]) -> And([a, b, c, d])
    fn flatten_local(&mut self) {
        let is_and = matches!(self, ValueFilter::And(_));

        match self {
            ValueFilter::And(filters) | ValueFilter::Or(filters) => {
                let needs_flattening = filters.iter().any(|f| {
                    matches!(
                        (is_and, f),
                        (true, ValueFilter::And(_)) | (false, ValueFilter::Or(_))
                    )
                });

                if !needs_flattening {
                    return;
                }

                let mut flattened = Vec::with_capacity(filters.len() * 2);
                for filter in std::mem::take(filters) {
                    match filter {
                        ValueFilter::And(mut inner) if is_and => {
                            flattened.append(&mut inner);
                        }
                        ValueFilter::Or(mut inner) if !is_and => {
                            flattened.append(&mut inner);
                        }
                        other => flattened.push(other),
                    }
                }
                *filters = flattened;
            }
            _ => {}
        }
    }

    /// And([If(_)]) -> If(_)
    fn normalize_local(&mut self) {
        match self {
            ValueFilter::And(filters) | ValueFilter::Or(filters) => {
                if filters.len() == 1 {
                    let single = filters.pop().unwrap();
                    *self = single;
                }
            }
            _ => {}
        }
    }

    /// Evaluate cheap operations first
    fn reorder_local(&mut self) {
        match self {
            ValueFilter::And(filters) | ValueFilter::Or(filters) => {
                if filters.len() <= 1 {
                    return;
                }

                let needs_reorder = filters.windows(2).any(|w| w[0].cost() > w[1].cost());
                if !needs_reorder {
                    return;
                }

                filters.sort_by_cached_key(|f| f.cost());
            }
            _ => {}
        }
    }

    fn cost(&self) -> usize {
        match self {
            ValueFilter::If(_, _) => 1,
            ValueFilter::Not(inner) => inner.cost() + 1,
            ValueFilter::And(filters) => filters.iter().map(|f| f.cost()).sum::<usize>() + 1,
            ValueFilter::Or(filters) => filters.iter().map(|f| f.cost()).sum::<usize>() + 1,
        }
    }

    pub fn to_type_filter(&self) -> TypeFilter {
        match self {
            ValueFilter::If(_, comp) => TypeFilter::With(comp.get_type()),
            ValueFilter::And(filters) => {
                TypeFilter::And(filters.iter().map(|f| f.to_type_filter()).collect())
            }
            ValueFilter::Or(filters) => {
                TypeFilter::Or(filters.iter().map(|f| f.to_type_filter()).collect())
            }
            ValueFilter::Not(filter) => match &**filter {
                ValueFilter::If(_, comp) => TypeFilter::Without(comp.get_type()),
                ValueFilter::Not(filter) => filter.to_type_filter(),
                _ => TypeFilter::Not(Box::new(filter.to_type_filter())),
            },
        }
    }
}

impl TypeFilter {
    /// Merges With(comp) into an optional existing filter
    pub fn add_with(existing: &mut Option<Self>, comp: ComponentType) {
        let with_filter = TypeFilter::With(comp);

        *existing = Some(match existing.take() {
            None => with_filter,
            Some(TypeFilter::And(mut filters)) => {
                if !filters.contains(&with_filter) {
                    filters.push(with_filter);
                }

                if filters.len() == 1 {
                    filters.pop().unwrap()
                } else {
                    TypeFilter::And(filters)
                }
            }
            Some(existing) => TypeFilter::merge_and(existing, with_filter),
        });
    }

    pub fn optimize(&mut self) {
        match self {
            TypeFilter::And(filters) | TypeFilter::Or(filters) => {
                for filter in filters.iter_mut() {
                    filter.optimize();
                }

                self.flatten_local();
                self.deduplicate_local();
                self.normalize_local();
                self.reorder_local();
            }
            TypeFilter::Not(inner) => {
                inner.optimize();
                self.simplify_not();
            }
            TypeFilter::With(_) | TypeFilter::Without(_) => {}
        }
    }

    fn simplify_not(&mut self) {
        let placeholder = TypeFilter::With(ComponentType::Light);
        let TypeFilter::Not(inner_box) = std::mem::replace(self, placeholder) else {
            return;
        };

        let simplified = match *inner_box {
            TypeFilter::With(comp) => TypeFilter::Without(comp),
            TypeFilter::Without(comp) => TypeFilter::With(comp),
            TypeFilter::Not(double_inner) => *double_inner,
            other => TypeFilter::Not(Box::new(other)),
        };

        *self = simplified;
    }

    /// And([And([a, b], And([c, d]))]) -> And([a, b, c, d])
    fn flatten_local(&mut self) {
        let is_and = matches!(self, TypeFilter::And(_));

        match self {
            TypeFilter::And(filters) | TypeFilter::Or(filters) => {
                let needs_flattening = filters.iter().any(|f| {
                    matches!(
                        (is_and, f),
                        (true, TypeFilter::And(_)) | (false, TypeFilter::Or(_))
                    )
                });

                if !needs_flattening {
                    return;
                }

                let mut flattened = Vec::with_capacity(filters.len() * 2);
                for filter in std::mem::take(filters) {
                    match filter {
                        TypeFilter::And(mut inner) if is_and => {
                            flattened.append(&mut inner);
                        }
                        TypeFilter::Or(mut inner) if !is_and => {
                            flattened.append(&mut inner);
                        }
                        other => flattened.push(other),
                    }
                }
                *filters = flattened;
            }
            _ => {}
        }
    }

    /// And([With(Light), With(Dimmer), With(Light)]) -> And([With(Light), With(Dimmer)])
    fn deduplicate_local(&mut self) {
        match self {
            TypeFilter::And(filters) | TypeFilter::Or(filters) => {
                if filters.len() <= 1 {
                    return;
                }

                let mut i = 0;
                while i < filters.len() {
                    if filters[..i].contains(&filters[i]) {
                        filters.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
    }

    /// And([With(_)]) -> With(_)
    fn normalize_local(&mut self) {
        match self {
            TypeFilter::And(filters) | TypeFilter::Or(filters) => {
                if filters.len() == 1 {
                    let single = filters.pop().unwrap();
                    *self = single;
                }
            }
            _ => {}
        }
    }

    /// Evaluate cheap operations first
    fn reorder_local(&mut self) {
        match self {
            TypeFilter::And(filters) | TypeFilter::Or(filters) => {
                if filters.len() <= 1 {
                    return;
                }

                let needs_reorder = filters.windows(2).any(|w| w[0].cost() > w[1].cost());
                if !needs_reorder {
                    return;
                }

                filters.sort_by_cached_key(|f| f.cost());
            }
            _ => {}
        }
    }

    fn cost(&self) -> usize {
        match self {
            TypeFilter::With(_) => 1,
            TypeFilter::Without(_) => 1,
            TypeFilter::Not(inner) => inner.cost() + 1,
            TypeFilter::And(filters) => filters.iter().map(|f| f.cost()).sum::<usize>() + 1,
            TypeFilter::Or(filters) => filters.iter().map(|f| f.cost()).sum::<usize>() + 1,
        }
    }

    pub fn merge_and(a: TypeFilter, b: TypeFilter) -> TypeFilter {
        match (a, b) {
            (TypeFilter::And(mut a_filters), TypeFilter::And(b_filters)) => {
                for filter in b_filters {
                    if !a_filters.contains(&filter) {
                        a_filters.push(filter);
                    }
                }
                TypeFilter::And(a_filters)
            }
            (TypeFilter::And(mut filters), other) | (other, TypeFilter::And(mut filters)) => {
                if !filters.contains(&other) {
                    filters.push(other);
                }
                TypeFilter::And(filters)
            }
            (a, b) => {
                if a == b {
                    a
                } else {
                    TypeFilter::And(vec![a, b])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Component,
        query::{
            ComponentAction, ComponentQuery, DeviceFilter, EntityAction, EntityIDFilter,
            EntityQuery,
        },
        types::compare::ComparisonOp,
    };

    #[test]
    fn test_type_filter_flatten() {
        let mut filter = TypeFilter::And(vec![
            TypeFilter::With(ComponentType::Light),
            TypeFilter::And(vec![
                TypeFilter::With(ComponentType::Dimmer),
                TypeFilter::With(ComponentType::Switch),
            ]),
        ]);
        filter.optimize();

        match filter {
            TypeFilter::And(filters) => assert_eq!(filters.len(), 3),
            _ => panic!("Expected flattened And"),
        }
    }

    #[test]
    fn test_type_filter_deduplicate() {
        let mut filter = TypeFilter::And(vec![
            TypeFilter::With(ComponentType::Light),
            TypeFilter::With(ComponentType::Dimmer),
            TypeFilter::With(ComponentType::Light),
        ]);
        filter.optimize();

        match filter {
            TypeFilter::And(filters) => assert_eq!(filters.len(), 2),
            _ => panic!("Expected deduplicated And"),
        }
    }

    #[test]
    fn test_type_filter_normalize() {
        let mut single = TypeFilter::And(vec![TypeFilter::With(ComponentType::Light)]);
        single.optimize();
        assert_eq!(single, TypeFilter::With(ComponentType::Light));

        let mut empty = TypeFilter::And(vec![]);
        empty.optimize();
        assert!(matches!(empty, TypeFilter::And(f) if f.is_empty()));
    }

    #[test]
    fn test_type_filter_simplify_not() {
        let mut filter = TypeFilter::Not(Box::new(TypeFilter::With(ComponentType::Light)));
        filter.optimize();
        assert_eq!(filter, TypeFilter::Without(ComponentType::Light));

        let mut filter = TypeFilter::Not(Box::new(TypeFilter::Without(ComponentType::Light)));
        filter.optimize();
        assert_eq!(filter, TypeFilter::With(ComponentType::Light));

        let mut filter = TypeFilter::Not(Box::new(TypeFilter::Not(Box::new(TypeFilter::With(
            ComponentType::Light,
        )))));
        filter.optimize();
        assert_eq!(filter, TypeFilter::With(ComponentType::Light));
    }

    #[test]
    fn test_type_filter_reorder_by_cost() {
        let mut filter = TypeFilter::And(vec![
            TypeFilter::Or(vec![
                TypeFilter::With(ComponentType::Light),
                TypeFilter::With(ComponentType::Dimmer),
            ]),
            TypeFilter::With(ComponentType::Switch),
        ]);
        filter.optimize();

        match filter {
            TypeFilter::And(filters) => {
                let costs: Vec<usize> = filters.iter().map(|f| f.cost()).collect();
                for i in 1..costs.len() {
                    assert!(costs[i - 1] <= costs[i]);
                }
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_type_filter_merge_and() {
        let result = TypeFilter::merge_and(
            TypeFilter::And(vec![
                TypeFilter::With(ComponentType::Light),
                TypeFilter::With(ComponentType::Dimmer),
            ]),
            TypeFilter::And(vec![
                TypeFilter::With(ComponentType::Switch),
                TypeFilter::With(ComponentType::Light),
            ]),
        );
        match result {
            TypeFilter::And(filters) => assert_eq!(filters.len(), 3),
            _ => panic!("Expected merged And"),
        }

        let result = TypeFilter::merge_and(
            TypeFilter::With(ComponentType::Light),
            TypeFilter::With(ComponentType::Light),
        );
        assert_eq!(result, TypeFilter::With(ComponentType::Light));
    }

    #[test]
    fn test_type_filter_complex() {
        let mut filter = TypeFilter::And(vec![
            TypeFilter::And(vec![
                TypeFilter::With(ComponentType::Light),
                TypeFilter::With(ComponentType::Light),
            ]),
            TypeFilter::With(ComponentType::Dimmer),
            TypeFilter::And(vec![TypeFilter::With(ComponentType::Light)]),
        ]);
        filter.optimize();

        match filter {
            TypeFilter::And(filters) => {
                assert_eq!(filters.len(), 2);
                assert!(filters.contains(&TypeFilter::With(ComponentType::Light)));
                assert!(filters.contains(&TypeFilter::With(ComponentType::Dimmer)));
            }
            _ => panic!("Expected optimized And"),
        }
    }

    #[test]
    fn test_value_filter_flatten_and_normalize() {
        let mut filter = ValueFilter::And(vec![
            ValueFilter::If(ComparisonOp::Eq, Component::Dimmer(0.1)),
            ValueFilter::And(vec![ValueFilter::If(
                ComparisonOp::Eq,
                Component::Switch(false),
            )]),
        ]);
        filter.optimize();

        match filter {
            ValueFilter::And(filters) => assert_eq!(filters.len(), 2),
            _ => panic!("Expected flattened And"),
        }

        let mut single = ValueFilter::And(vec![ValueFilter::If(
            ComparisonOp::Eq,
            Component::Dimmer(0.1),
        )]);
        single.optimize();
        assert!(matches!(single, ValueFilter::If(_, _)));
    }

    #[test]
    fn test_value_filter_to_type_filter() {
        let filter = ValueFilter::If(ComparisonOp::Eq, Component::Dimmer(0.1));
        assert_eq!(
            filter.to_type_filter(),
            TypeFilter::With(ComponentType::Dimmer)
        );

        let filter = ValueFilter::Not(Box::new(ValueFilter::If(
            ComparisonOp::Eq,
            Component::Dimmer(0.1),
        )));
        assert_eq!(
            filter.to_type_filter(),
            TypeFilter::Without(ComponentType::Dimmer)
        );

        let filter = ValueFilter::Not(Box::new(ValueFilter::Not(Box::new(ValueFilter::If(
            ComparisonOp::Eq,
            Component::Dimmer(0.1),
        )))));
        assert_eq!(
            filter.to_type_filter(),
            TypeFilter::With(ComponentType::Dimmer)
        );

        let filter = ValueFilter::And(vec![
            ValueFilter::If(ComparisonOp::Eq, Component::Dimmer(0.1)),
            ValueFilter::If(ComparisonOp::Eq, Component::Switch(false)),
        ]);
        match filter.to_type_filter() {
            TypeFilter::And(filters) => {
                assert_eq!(filters.len(), 2);
                assert!(filters.contains(&TypeFilter::With(ComponentType::Dimmer)));
                assert!(filters.contains(&TypeFilter::With(ComponentType::Switch)));
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_entity_filter_add_components() {
        let mut filter = EntityFilter {
            id: EntityIDFilter::Any,
            type_filter: None,
            value_filter: None,
            last_update: None,
        };
        TypeFilter::add_with(&mut filter.type_filter, ComponentType::Light);
        assert_eq!(
            filter.type_filter,
            Some(TypeFilter::With(ComponentType::Light))
        );

        let mut filter = EntityFilter {
            id: EntityIDFilter::Any,
            type_filter: Some(TypeFilter::With(ComponentType::Light)),
            value_filter: None,
            last_update: None,
        };
        TypeFilter::add_with(&mut filter.type_filter, ComponentType::Dimmer);
        match filter.type_filter {
            Some(TypeFilter::And(filters)) => assert_eq!(filters.len(), 2),
            _ => panic!("Expected And with both components"),
        }

        let mut filter = EntityFilter {
            id: EntityIDFilter::Any,
            type_filter: Some(TypeFilter::And(vec![TypeFilter::With(
                ComponentType::Light,
            )])),
            value_filter: None,
            last_update: None,
        };
        TypeFilter::add_with(&mut filter.type_filter, ComponentType::Light);
        assert_eq!(
            filter.type_filter,
            Some(TypeFilter::With(ComponentType::Light))
        );
    }

    #[test]
    fn test_entity_filter_value_to_type() {
        let mut filter = EntityFilter {
            id: EntityIDFilter::Any,
            type_filter: None,
            value_filter: Some(ValueFilter::If(ComparisonOp::Eq, Component::Dimmer(0.1))),
            last_update: None,
        };
        filter.optimize();
        assert_eq!(
            filter.type_filter,
            Some(TypeFilter::With(ComponentType::Dimmer))
        );

        let mut filter = EntityFilter {
            id: EntityIDFilter::Any,
            type_filter: Some(TypeFilter::With(ComponentType::Light)),
            value_filter: Some(ValueFilter::If(ComparisonOp::Eq, Component::Dimmer(0.1))),
            last_update: None,
        };
        filter.optimize();
        match filter.type_filter {
            Some(TypeFilter::And(filters)) => assert_eq!(filters.len(), 2),
            _ => panic!("Expected merged And"),
        }
    }

    #[test]
    fn test_entity_filter_removes_empty() {
        let mut filter = EntityFilter {
            id: EntityIDFilter::Any,
            type_filter: Some(TypeFilter::And(vec![])),
            value_filter: None,
            last_update: None,
        };
        filter.optimize();
        assert_eq!(filter.type_filter, None);
    }

    #[test]
    fn test_query_component_optimize() {
        let mut query = OneShotQuery::Component(ComponentQuery {
            device_filter: DeviceFilter::default(),
            entity_filter: EntityFilter {
                id: EntityIDFilter::Any,
                type_filter: None,
                value_filter: Some(ValueFilter::If(ComparisonOp::Eq, Component::Switch(false))),
                last_update: None,
            },
            action: ComponentAction::GetValue,
            component: ComponentType::Light,
            post_op: None,
            include_parents: false,
            limit: None,
        });

        query.optimize();

        match query {
            OneShotQuery::Component(cq) => match cq.entity_filter.type_filter {
                Some(TypeFilter::And(filters)) => {
                    assert_eq!(filters.len(), 2);
                    assert!(filters.contains(&TypeFilter::With(ComponentType::Light)));
                    assert!(filters.contains(&TypeFilter::With(ComponentType::Switch)));
                }
                _ => panic!("Expected And with component and value filter type"),
            },
            _ => panic!("Expected Component query"),
        }
    }

    #[test]
    fn test_query_entity_optimize() {
        let mut query = OneShotQuery::Entity(EntityQuery {
            device_filter: DeviceFilter::default(),
            entity_filter: EntityFilter {
                id: EntityIDFilter::Any,
                type_filter: Some(TypeFilter::And(vec![
                    TypeFilter::With(ComponentType::Light),
                    TypeFilter::With(ComponentType::Light),
                ])),
                value_filter: None,
                last_update: None,
            },
            action: EntityAction::Snapshot,
            limit: None,
        });

        query.optimize();

        match query {
            OneShotQuery::Entity(eq) => {
                assert_eq!(
                    eq.entity_filter.type_filter,
                    Some(TypeFilter::With(ComponentType::Light))
                );
            }
            _ => panic!("Expected Entity query"),
        }
    }
}
