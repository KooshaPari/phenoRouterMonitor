//! Example demonstrating the Builder derive macro
//!
//! This example shows how to use the Builder derive macro to generate
//! a fluent builder interface for struct construction.

use phenotype_macros::Builder;

/// A simple Person struct with builder support
#[derive(Builder, Clone)]
struct Person {
    name: String,
    age: u32,
}

/// A more complex struct with multiple fields
#[derive(Builder, Clone)]
struct Address {
    street: String,
    city: String,
    zip_code: String,
    country: String,
}

fn main() {
    // Example 1: Building a Person using the builder
    let person = PersonBuilder::new()
        .name("Alice".to_string())
        .age(30)
        .build()
        .expect("Failed to build person");

    println!("Person: {} (age {})", person.name, person.age);

    // Example 2: Using the default builder
    let builder = PersonBuilder::default();
    let person2 = builder
        .name("Bob".to_string())
        .age(25)
        .build()
        .expect("Failed to build person");

    println!("Person: {} (age {})", person2.name, person2.age);

    // Example 3: Building an Address
    let address = AddressBuilder::new()
        .street("123 Main St".to_string())
        .city("San Francisco".to_string())
        .zip_code("94102".to_string())
        .country("USA".to_string())
        .build()
        .expect("Failed to build address");

    println!(
        "Address: {}, {}, {}, {}",
        address.street, address.city, address.zip_code, address.country
    );

    // Example 4: Demonstrating missing field error
    let result = PersonBuilder::new()
        .name("Charlie".to_string())
        .build();

    match result {
        Ok(_) => println!("Unexpectedly succeeded"),
        Err(e) => println!("Expected error for missing age: {}", e),
    }

    // Example 5: Using builder convenience method on struct
    let person3 = Person::builder()
        .name("Diana".to_string())
        .age(28)
        .build()
        .expect("Failed to build person");

    println!("Person: {} (age {})", person3.name, person3.age);
}
