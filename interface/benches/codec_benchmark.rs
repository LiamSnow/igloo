use bytes::BytesMut;
use criterion::{Criterion, criterion_group, criterion_main};
use smallvec::smallvec;
use std::hint::black_box;

use igloo_interface::{Bool, Color, Component, ComponentUpdate, Float, IglooCodable, Int, Text};

fn create_test_update() -> ComponentUpdate {
    ComponentUpdate {
        device: 1,
        entity: 2,
        values: smallvec![
            Component::Bool(Bool(true)),
            // Component::Text(Text("asdfasdf as fiojfqoi j".to_string())),
            Component::Float(Float(16.2)),
            Component::Int(Int(10)),
            Component::Color(Color {
                r: 10,
                g: 20,
                b: 30
            })
        ],
    }
}

fn bench_igloo_codec(c: &mut Criterion) {
    let update = create_test_update();

    c.bench_function("igloo_encode", |b| {
        b.iter(|| {
            let mut buf = BytesMut::with_capacity(256);
            black_box(update.encode(&mut buf).unwrap());
        });
    });

    let mut buf = BytesMut::with_capacity(256);
    update.encode(&mut buf).unwrap();
    let encoded = buf.freeze();

    c.bench_function("igloo_decode", |b| {
        b.iter(|| {
            let mut bytes = encoded.clone();
            black_box(ComponentUpdate::decode(&mut bytes).unwrap());
        });
    });
}

fn bench_serde_json(c: &mut Criterion) {
    let update = create_test_update();

    c.bench_function("json_encode", |b| {
        b.iter(|| {
            black_box(serde_json::to_vec(&update).unwrap());
        });
    });

    let encoded = serde_json::to_vec(&update).unwrap();

    c.bench_function("json_decode", |b| {
        b.iter(|| {
            black_box(serde_json::from_slice::<ComponentUpdate>(&encoded).unwrap());
        });
    });
}

fn bench_bincode(c: &mut Criterion) {
    let update = create_test_update();
    let config = bincode::config::standard();

    c.bench_function("bincode_encode", |b| {
        b.iter(|| {
            black_box(bincode::encode_to_vec(&update, config).unwrap());
        });
    });

    let encoded = bincode::encode_to_vec(&update, config).unwrap();

    c.bench_function("bincode_decode", |b| {
        b.iter(|| {
            let (decoded, _): (ComponentUpdate, _) =
                bincode::decode_from_slice(&encoded, config).unwrap();
            black_box(decoded);
        });
    });
}

fn bench_roundtrip_comparison(c: &mut Criterion) {
    let update = create_test_update();
    let bincode_config = bincode::config::standard();

    c.bench_function("igloo_roundtrip", |b| {
        b.iter(|| {
            let mut buf = BytesMut::with_capacity(256);
            update.encode(&mut buf).unwrap();
            let mut bytes = buf.freeze();
            black_box(ComponentUpdate::decode(&mut bytes).unwrap());
        });
    });

    c.bench_function("json_roundtrip", |b| {
        b.iter(|| {
            let encoded = serde_json::to_vec(&update).unwrap();
            black_box(serde_json::from_slice::<ComponentUpdate>(&encoded).unwrap());
        });
    });

    c.bench_function("bincode_roundtrip", |b| {
        b.iter(|| {
            let encoded = bincode::encode_to_vec(&update, bincode_config).unwrap();
            let (decoded, _): (ComponentUpdate, _) =
                bincode::decode_from_slice(&encoded, bincode_config).unwrap();
            black_box(decoded);
        });
    });
}

criterion_group!(
    benches,
    bench_igloo_codec,
    bench_serde_json,
    bench_bincode,
    bench_roundtrip_comparison
);

criterion_main!(benches);
