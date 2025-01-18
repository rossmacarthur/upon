use crate::context::{Context, User};
use crate::{Engine, Handlebars, Liquid, Minijinja, Tera, TinyTemplate, Upon};

macro_rules! t {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!($source), false);
        goldie::assert!(result);
    }};
}

macro_rules! t_syntax {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!($source), true);
        goldie::assert!(result);
    }};
}

#[test]
fn basic_handlebars() {
    t!(Handlebars, "../benchdata/basic/handlebars.html");
}
#[test]
fn basic_liquid() {
    t!(Liquid, "../benchdata/basic/liquid.html");
}

#[test]
fn basic_minijinja() {
    t!(Minijinja, "../benchdata/basic/jinja.html");
}

#[test]
fn basic_tera() {
    t!(Tera, "../benchdata/basic/jinja.html");
}

#[test]
fn basic_tinytemplate() {
    t!(TinyTemplate, "../benchdata/basic/tinytemplate.html");
}

#[test]
fn basic_upon() {
    t!(Upon, "../benchdata/basic/jinja.html");
}

#[test]
fn filters_handlebars() {
    t!(Handlebars, "../benchdata/filters/handlebars.html");
}

#[test]
fn filters_minijinja() {
    t!(Minijinja, "../benchdata/filters/jinja.html");
}

#[test]
fn filters_tera() {
    t!(Tera, "../benchdata/filters/jinja.html");
}

#[test]
fn filters_upon() {
    t!(Upon, "../benchdata/filters/jinja.html");
}

#[test]
fn literals_minijinja() {
    t!(Minijinja, "../benchdata/literals/jinja.html");
}

#[test]
fn literals_upon() {
    t!(Upon, "../benchdata/literals/jinja.html");
}

#[test]
fn syntax_minijinja() {
    t_syntax!(Minijinja, "../benchdata/syntax/jinja.html");
}

#[test]
fn syntax_upon() {
    t_syntax!(Upon, "../benchdata/syntax/jinja.html");
}

fn render<'a, E: Engine<'a>>(source: &'a str, syntax: bool) -> String {
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

    let mut engine = if syntax {
        E::with_syntax(("{", "}"), ("<%", "%>"), ("<#", "#>"))
    } else {
        E::new()
    };
    engine.add_template("bench", source);
    engine.render("bench", &ctx)
}
