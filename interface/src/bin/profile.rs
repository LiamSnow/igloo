use bytes::BytesMut;
use igloo_interface::{Bool, Color, Component, ComponentUpdate, Float, IglooCodable, Int};
use smallvec::smallvec;

fn create_test_update() -> ComponentUpdate {
    ComponentUpdate {
        device: 42,
        entity: 123,
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

fn profile_igloo_encode() {
    let update = create_test_update();
    let mut buf = BytesMut::with_capacity(256);

    for _ in 0..1_000_000 {
        buf.clear();
        update.encode(&mut buf).unwrap();
        // Prevent optimization
        std::hint::black_box(&buf);
    }
}

fn profile_igloo_decode() {
    let update = create_test_update();
    let mut buf = BytesMut::with_capacity(256);
    update.encode(&mut buf).unwrap();
    let encoded = buf.freeze();

    for _ in 0..1_000_000 {
        let mut bytes = encoded.clone();
        let decoded = ComponentUpdate::decode(&mut bytes).unwrap();
        // Prevent optimization
        std::hint::black_box(decoded);
    }
}

fn profile_bincode_encode() {
    let update = create_test_update();

    for _ in 0..1_000_000 {
        let encoded = bincode::encode_to_vec(&update, bincode::config::standard()).unwrap();
        // Prevent optimization
        std::hint::black_box(encoded);
    }
}

fn profile_bincode_decode() {
    let update = create_test_update();
    let encoded = bincode::encode_to_vec(&update, bincode::config::standard()).unwrap();

    for _ in 0..1_000_000 {
        let decoded: ComponentUpdate =
            bincode::decode_from_slice(&encoded, bincode::config::standard())
                .unwrap()
                .0;
        // Prevent optimization
        std::hint::black_box(decoded);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!(
            "Usage: {} <igloo-encode|igloo-decode|bincode-encode|bincode-decode>",
            args[0]
        );
        std::process::exit(1);
    }

    match args[1].as_str() {
        "igloo-encode" => profile_igloo_encode(),
        "igloo-decode" => profile_igloo_decode(),
        "bincode-encode" => profile_bincode_encode(),
        "bincode-decode" => profile_bincode_decode(),
        _ => {
            eprintln!("Unknown profile target: {}", args[1]);
            std::process::exit(1);
        }
    }
}
