use criterion::{Criterion, criterion_group, criterion_main};
use igloo_interface::{
    Component, ComponentType,
    query::{
        ComponentAction, ComponentQuery, DeviceAction, DeviceFilter, DeviceQuery, EntityAction,
        EntityFilter, EntityQuery, Query, TypeFilter, ValueFilter,
    },
    types::{agg::AggregationOp, compare::ComparisonOp},
};
use igloo_server::{query::QueryEngine, tree::sim::make_test_tree};
use std::hint::black_box;

fn tree_sizes() -> Vec<(&'static str, usize, usize, usize)> {
    vec![
        // # extensions, # groups, # devs
        ("small", 2, 4, 14),
        ("medium", 3, 8, 40),
        ("large", 5, 15, 100),
    ]
}

fn queries() -> Vec<(&'static str, Query)> {
    vec![
        (
            "device_get_all_ids",
            Query::Device(DeviceQuery {
                filter: DeviceFilter::default(),
                action: DeviceAction::GetID,
                limit: None,
            }),
        ),
        (
            "device_count",
            Query::Device(DeviceQuery {
                filter: DeviceFilter::default(),
                action: DeviceAction::Count,
                limit: None,
            }),
        ),
        (
            "component_dimmer_gt_half",
            Query::Component(ComponentQuery {
                device_filter: DeviceFilter::default(),
                entity_filter: EntityFilter {
                    value_filter: Some(ValueFilter::If(ComparisonOp::Gt, Component::Dimmer(0.5))),
                    ..Default::default()
                },
                action: ComponentAction::GetValue,
                component: ComponentType::Dimmer,
                post_op: None,
                include_parents: true,
                limit: None,
            }),
        ),
        (
            "entity_light_and_dimmer",
            Query::Entity(EntityQuery {
                device_filter: DeviceFilter::default(),
                entity_filter: EntityFilter {
                    type_filter: Some(TypeFilter::And(vec![
                        TypeFilter::With(ComponentType::Light),
                        TypeFilter::With(ComponentType::Dimmer),
                    ])),
                    ..Default::default()
                },
                action: EntityAction::Snapshot,
                limit: None,
            }),
        ),
        (
            "component_dimmer_mean",
            Query::Component(ComponentQuery {
                device_filter: DeviceFilter::default(),
                entity_filter: EntityFilter::default(),
                action: ComponentAction::GetValue,
                component: ComponentType::Dimmer,
                post_op: Some(AggregationOp::Mean),
                include_parents: false,
                limit: None,
            }),
        ),
        (
            "entity_count_with_switch",
            Query::Entity(EntityQuery {
                device_filter: DeviceFilter::default(),
                entity_filter: EntityFilter {
                    type_filter: Some(TypeFilter::With(ComponentType::Switch)),
                    ..Default::default()
                },
                action: EntityAction::Count,
                limit: None,
            }),
        ),
    ]
}

fn bench_queries(c: &mut Criterion) {
    for (size_name, extensions, groups, devices) in tree_sizes() {
        let mut tree = make_test_tree(extensions, groups, devices);
        let mut engine = QueryEngine::default();

        for (query_name, query) in queries() {
            c.bench_function(&format!("{size_name}/{query_name}"), |b| {
                b.iter(|| {
                    engine.eval(&mut tree, black_box(query.clone())).unwrap();
                });
            });
        }
    }
}

criterion_group!(benches, bench_queries);
criterion_main!(benches);
