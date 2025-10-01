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
