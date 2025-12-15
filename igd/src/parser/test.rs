// use bumpalo::Bump;

// use super::parse;

// #[test]
// fn test() {
//     //     let source = r##"
//     // using

//     // use std::RGBCTLight;
//     // use std::ThermostatCard;
//     // use mushroom::Button;
//     // use mushroom::SensorCard;

//     // type ZombieID = int;

//     // struct Zombie {
//     //     id: ZombieID,
//     //     health: int,
//     //     walk_speed: float,
//     //     name: string,
//     // }

//     // fn usage() {
//     //     let my_zombie = Zombie {
//     //         id: 10,
//     //         health: 100,
//     //         walk_speed: 1.5,
//     //         name: "My Zombie",
//     //     };

//     //     my_zombie.health -= 10;
//     // }

//     // enum Animal {
//     //     Cat,
//     //     Dog,
//     //     Sheep
//     // }

//     // fn usage() {
//     //     let my_animal = Animal::Cat;
//     // }

//     // fn square(num: int) -> int {
//     //     return num * num;
//     // }

//     // fn vars() {
//     //     let i = 1;

//     //     i = 2;

//     //     i += 5;

//     //     i *= 2;

//     //     let q = [1, 2, 3];
//     // }

//     // fn tuples() {
//     //     let my_tuple = (10, "Hello");

//     //     let my_tuple: (int, string) = (10, "Hello");

//     //     my_tuple.0 = 50; // valid
//     //     my_tuple.1 = 1.2; // invalid type
//     //     my_tuple.1 = 1.2 as string; // valid because of cast
//     // }

//     // fn loops() {
//     //     for i in 0..2 {
//     //         if i == 1 {
//     //             break;
//     //         }
//     //     }

//     //     if 4 < 10 {
//     //         print("Hello World");
//     //     }

//     //     while i > 69 {
//     //         print("Hello World");
//     //     }
//     // }

//     // element TestEl {
//     //     let a = 2;
//     //     let b = 2;
//     //     let c = 2;
//     // }

//     // element TestEl2(group: Group, show_humidity: bool) {
//     // }

//     // dashboard "Example" {
//     // }

//     //     "##;
//     let source = r##"

// fn test() {
//     let x = $ + 1;
//     let y = 2 * @;
//     let z = 3;
// }

//     "##;

//     let arena = Bump::new();
//     let mut errors = Vec::new();
//     let ast = parse(source, &arena, &mut errors).unwrap();
//     println!("AST: {ast:#?}");
//     println!("ERRORS: {errors:?}");
// }
