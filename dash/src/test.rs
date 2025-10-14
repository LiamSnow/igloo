use crate::parser::parse;

#[test]
fn test_master() {
    let source = r###"


fn add(a: i32, b: i32) -> i32 {
	a + b
}


    "###;
    let ast = parse(source).unwrap();
    println!("{ast:#?}");
}
