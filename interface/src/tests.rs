use crate::{Bool, Component, Float, FloeCommand, Int, Text, UpdatesPayload};

#[test]
fn test_to_from() {
    let cmd = FloeCommand::Updates(UpdatesPayload {
        device: 1,
        entity: 2,
        values: vec![
            Component::Bool(Bool(true)),
            Component::Text(Text("asdfasdf as fiojfqoi j".to_string())),
            Component::Float(Float(16.2)),
            Component::Int(Int(10)),
        ],
    });

    let encoded = borsh::to_vec(&cmd).unwrap();

    let decoded: FloeCommand = borsh::from_slice(&encoded[..]).unwrap();

    assert_eq!(cmd, decoded);
}

#[test]
fn test_encode() {
    let comp = Component::Int(Int(10));
    let encoded = borsh::to_vec(&comp).unwrap();
    println!("{encoded:02X?}");
}
