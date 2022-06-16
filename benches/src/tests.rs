use crate::context::{Context, User};
use crate::{Engine, Handlebars, Minijinja, Tera, TinyTemplate, Upon};

macro_rules! t {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!($source));
        goldie::assert!(result);
    }};
}

#[test]
fn handlebars() {
    t!(Handlebars, "../benchdata/handlebars.html");
}

#[test]
fn minijinja() {
    t!(Minijinja, "../benchdata/liquid.html");
}

#[test]
fn tera() {
    t!(Tera, "../benchdata/liquid.html");
}

#[test]
fn tinytemplate() {
    t!(TinyTemplate, "../benchdata/tinytemplate.html");
}

#[test]
fn upon() {
    t!(Upon, "../benchdata/liquid.html");
}

fn render<'a, E: Engine<'a>>(source: &'a str) -> String {
    let ctx = Context {
        title: "My awesome webpage!".to_owned(),
        users: vec![
            User {
                name: "Nancy Wheeler".to_owned(),
                age: 17,
                is_disabled: false,
            },
            User {
                name: "Steve Harrington".to_owned(),
                age: 18,
                is_disabled: false,
            },
            User {
                name: "Billy Hargrove".to_owned(),
                age: 19,
                is_disabled: true,
            },
        ],
    };

    let mut engine = E::new();
    engine.add_template("bench", &source);
    engine.render("bench", &ctx)
}
