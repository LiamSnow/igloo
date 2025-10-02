use borsh::{BorshDeserialize, BorshSerialize};

use crate::Color;

#[test]
fn idk() {
    let color = Color {
        r: 10,
        g: 255,
        b: 50,
    };

    let encoded = borsh::to_vec(&color).unwrap();

    let decoded: Color = borsh::from_slice(&encoded[..]).unwrap();

    assert_eq!(color, decoded);
}

#[derive(BorshSerialize, BorshDeserialize)]
struct MsgPrefix {
    length: u32,
    code: u16,
}

#[test]
fn borsh_test() {
    let prefix = MsgPrefix {
        length: 10,
        code: 15,
    };
    println!("A: {:#?}", borsh::to_vec(&prefix).unwrap());

    let prefix = MsgPrefix {
        length: 131_072,
        code: 15,
    };
    println!("B: {:#?}", borsh::to_vec(&prefix).unwrap());
}
