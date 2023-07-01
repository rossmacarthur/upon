#![cfg(feature = "builtins")]

use upon::{value, Engine};

#[test]
fn lower() {
    let result = Engine::new()
        .compile(r#"{{ "TEST" | lower }}"#)
        .unwrap()
        .render(value! {})
        .unwrap();
    assert_eq!(result, "test");
}

#[test]
fn upper() {
    let result = Engine::new()
        .compile(r#"{{ "test" | upper }}"#)
        .unwrap()
        .render(value! {})
        .unwrap();
    assert_eq!(result, "TEST");
}

#[test]
fn replace() {
    let result = Engine::new()
        .compile(r#"{{ "tEst" | replace: "E", "e" }}"#)
        .unwrap()
        .render(value! {})
        .unwrap();
    assert_eq!(result, "test");
}

#[test]
fn first() {
    let result = Engine::new()
        .compile(r#"{{ names | first }}"#)
        .unwrap()
        .render(value! {
            names: ["John", "James"]
        })
        .unwrap();
    assert_eq!(result, "John");
}

#[test]
fn last() {
    let result = Engine::new()
        .compile(r#"{{ names | last }}"#)
        .unwrap()
        .render(value! {
            names: ["John", "James"]
        })
        .unwrap();
    assert_eq!(result, "James");
}

#[test]
fn keys() {
    let result = Engine::new()
        .compile(r#"{% for key in user | keys %}{{ key }} {% endfor %}"#)
        .unwrap()
        .render(value! {
            user: {
                name: "John",
                age: 42,
            },
        })
        .unwrap();
    assert_eq!(result, "age name ");
}

#[test]
fn values() {
    let result = Engine::new()
        .compile(r#"{% for value in user | values %}{{ value }} {% endfor %}"#)
        .unwrap()
        .render(value! {
            user: {
                name: "John",
                age: 42,
            },
        })
        .unwrap();
    assert_eq!(result, "42 John ");
}

#[test]
fn len_list() {
    let result = Engine::new()
        .compile(r#"{{ names | len }}"#)
        .unwrap()
        .render(value! {
            names: ["John", "James"]
        })
        .unwrap();
    assert_eq!(result, "2");
}

#[test]
fn len_map() {
    let result = Engine::new()
        .compile(r#"{{ user | len }}"#)
        .unwrap()
        .render(value! {
            user: {
                name: "John",
                age: 42,
                is_enabled: true,
            }
        })
        .unwrap();
    assert_eq!(result, "3");
}

#[test]
fn get_list() {
    let result = Engine::new()
        .compile(r#"{{ names | get: 1 }}"#)
        .unwrap()
        .render(value! {
            names: ["John", "James"]
        })
        .unwrap();
    assert_eq!(result, "James");
}

#[test]
fn get_map() {
    let result = Engine::new()
        .compile(r#"{{ user | get: "name" }}"#)
        .unwrap()
        .render(value! {
            user: {
                name: "John",
            }
        })
        .unwrap();
    assert_eq!(result, "John");
}

#[test]
fn get_list_err_cannot_index_into_list_with_string() {
    let result = Engine::new()
        .compile(r#"{{ names | get: "name" }}"#)
        .unwrap()
        .render(value! {
            names: ["John", "James"]
        })
        .unwrap();
    assert_eq!(result, "James");
}

#[test]
fn get_map_err_cannot_index_into_map_with_integer() {
    let result = Engine::new()
        .compile(r#"{{ user | get: 123 }}"#)
        .unwrap()
        .render(value! {
            user: {
                name: "John",
            }
        })
        .unwrap();
    assert_eq!(result, "John");
}
