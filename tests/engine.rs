#![cfg(feature = "serde")]

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
fn engine_compile_borrowed_source_non_static() -> upon::Result<()> {
    let engine = Engine::new();
    let source = String::from("{{ lorem }}");
    let result = engine.compile(&source)?.render(value! { lorem: "ipsum" })?;
    assert_eq!(result, "ipsum");
    Ok(())
}

#[test]
fn engine_add_template_borrowed_source_non_static() -> upon::Result<()> {
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

#[test]
fn engine_add_template_owned_source() -> upon::Result<()> {
    let mut engine = Engine::new();
    let source = String::from("{{ lorem }}");
    engine.add_template("test", source)?;
    let result = engine
        .get_template("test")
        .unwrap()
        .render(value! { lorem: "ipsum" })?;
    assert_eq!(result, "ipsum");
    Ok(())
}

#[cfg(feature = "filters")]
#[test]
fn engine_add_template_nested() {
    use upon::filters::*;

    #[derive(Default)]
    struct Wrapper<'engine> {
        engine: Engine<'engine>,
    }

    impl<'engine> Wrapper<'engine> {
        fn add_filter<F, R, A>(&mut self, name: &'engine str, f: F)
        where
            F: Filter<R, A> + Send + Sync + 'static,
            R: FilterReturn,
            A: for<'a> FilterArgs<'a>,
        {
            self.engine.add_filter(name, f);
        }
    }

    let mut engine = Wrapper::default();
    engine.add_filter("lower", str::to_lowercase);
}
