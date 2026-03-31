//! Example demonstrating the Builder derive macro
//!
//! NOTE: This example is disabled until the Builder macro is implemented.
//! See crates/phenotype-macros/src/lib.rs

// use phenotype_macros::Builder;

// /// A simple Person struct with builder support
// #[derive(Builder, Clone)]
// struct Person {
//     name: String,
//     age: u32,
// }

// fn main() {
//     let person = PersonBuilder::new()
//         .name("Alice".to_string())
//         .age(30)
//         .build()
//         .expect("Failed to build person");

//     println!("Person: {} (age {})", person.name, person.age);
// }
