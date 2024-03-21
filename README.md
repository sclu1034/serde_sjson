# serde_sjson

A **ser**ialization/**de**serialization library for Simplified JSON,
the Bitsquid/Stingray flavor of JSON.

## Usage

### Serializing

```rust
use serde::Serialize;
use serde_sjson::Result;

#[derive(Serialize)]
struct Person {
    name: String,
    age: u8,
    friends: Vec<String>,
}

fn main() -> Result<()> {
    let data = Person {
        name: String::from("Marc"),
        age: 21,
        friends: vec![String::from("Jessica"), String::from("Paul")],
    };

    let s = serde_sjson::to_string(&data)?;

    println!("{}", s);

    Ok(())
}
```

### Deserializing

```rust
use serde::Deserialize;
use serde_sjson::Result;

#[derive(Deserialize)]
struct Person {
    name: String,
    age: u8,
    friends: Vec<String>,
}

fn main() -> Result<()> {
    let sjson = r#"
    name = Marc
    age = 21
    friends = [
        Jessica
        Paul
    ]"#;

    let data: Person = serde_sjson::from_str(sjson)?;

    println!(
        "{} is {} years old and has {} friends.",
        data.name,
        data.age,
        data.friends.len()
    );

    Ok(())
}
```
