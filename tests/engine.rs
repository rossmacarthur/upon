#![cfg(feature = "serde")]

use std::thread;

use upon::{value, Engine};

#[test]
fn engine_debug() {
    let _ = format!("{:?}", Engine::new());
}

#[test]
fn engine_send_and_sync() {
    let engine = Engine::new();
    let handle = thread::spawn(move || {
        let result = engine
            .compile("{{ lorem }}")
            .unwrap()
            .render(&engine, value! { lorem: "ipsum" })
            .to_string()
            .unwrap();
        assert_eq!(result, "ipsum");
    });
    handle.join().unwrap();
}

#[test]
fn engine_compile_borrowed_source_non_static() -> upon::Result<()> {
    let engine = Engine::new();
    let source = String::from("{{ lorem }}");
    let result = engine
        .compile(&source)?
        .render(&engine, value! { lorem: "ipsum" })
        .to_string()?;
    assert_eq!(result, "ipsum");
    Ok(())
}

#[test]
fn engine_add_template_borrowed_source_non_static() -> upon::Result<()> {
    let mut engine = Engine::new();
    let source = String::from("{{ lorem }}");
    engine.add_template("test", &source)?;
    let result = engine
        .template("test")
        .render(value! { lorem: "ipsum" })
        .to_string()?;
    assert_eq!(result, "ipsum");
    Ok(())
}

#[test]
fn engine_add_template_owned_source() -> upon::Result<()> {
    let mut engine = Engine::new();
    let source = String::from("{{ lorem }}");
    engine.add_template("test", source)?;
    let result = engine
        .template("test")
        .render(value! { lorem: "ipsum" })
        .to_string()?;
    assert_eq!(result, "ipsum");
    Ok(())
}

#[cfg(feature = "functions")]
#[test]
fn engine_add_function_nested() {
    use upon::functions::*;

    #[derive(Default)]
    struct Wrapper<'engine> {
        engine: Engine<'engine>,
    }

    impl<'engine> Wrapper<'engine> {
        fn add_function<F, R, A>(&mut self, name: &'engine str, f: F)
        where
            F: Function<R, A> + Send + Sync + 'static,
            R: FunctionReturn,
            A: FunctionArgs,
        {
            self.engine.add_function(name, f);
        }
    }

    let mut engine = Wrapper::default();
    engine.add_function("lower", str::to_lowercase);
}
