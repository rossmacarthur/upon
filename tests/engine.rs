use std::thread;

use upon::{value, Engine};

#[test]
fn engine_debug() {
    format!("{:?}", Engine::new());
}

#[test]
fn engine_send_and_sync() {
    let engine = Engine::new();
    thread::spawn(move || {
        let result = engine
            .compile("{{ lorem }}")
            .unwrap()
            .render(value! { lorem: "ipsum" })
            .unwrap();
        assert_eq!(result, "ipsum");
    });
}

#[test]
fn engine_compile_non_static_source() -> upon::Result<()> {
    let engine = Engine::new();
    let source = String::from("{{ lorem }}");
    let result = engine.compile(&source)?.render(value! { lorem: "ipsum" })?;
    assert_eq!(result, "ipsum");
    Ok(())
}

#[test]
fn engine_add_template_non_static_source() -> upon::Result<()> {
    let mut engine = Engine::new();
    let source = String::from("{{ lorem }}");
    engine.add_template("test", &source)?;
    let result = engine
        .get_template("test")
        .unwrap()
        .render(value! { lorem: "ipsum" })?;
    assert_eq!(result, "ipsum");
    Ok(())
}
