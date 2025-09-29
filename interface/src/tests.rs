use crate::{Bool, Component, ComponentUpdate, Float, FloeCommand, IglooCodable, Int, Text};
use smallvec::smallvec;

#[test]
fn test() {
    let cmd = FloeCommand::Updates(ComponentUpdate {
        device: 1,
        entity: 2,
        values: smallvec![
            Component::Bool(Bool(true)),
            Component::Text(Text("asdfasdf as fiojfqoi j".to_string())),
            Component::Float(Float(16.2)),
            Component::Int(Int(10)),
        ],
    });
}
