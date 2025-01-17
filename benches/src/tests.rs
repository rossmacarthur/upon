use crate::context::{Context, User};
use crate::{Engine, Handlebars, Liquid, Minijinja, Tera, TinyTemplate, Upon};

macro_rules! t {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!($source), false, false);
        goldie::assert!(result);
    }};
}

macro_rules! t_filters {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!($source), false, true);
        goldie::assert!(result);
    }};
}

macro_rules! t_syntax {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!($source), true, false);
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
    t!(Minijinja, "../benchdata/basic/minijinja.html");
}

#[test]
fn basic_tera() {
    t!(Tera, "../benchdata/basic/tera.html");
}

#[test]
fn basic_tinytemplate() {
    t!(TinyTemplate, "../benchdata/basic/tinytemplate.html");
}

#[test]
fn basic_upon() {
    t!(Upon, "../benchdata/basic/upon.html");
}

#[test]
fn filters_handlebars() {
    t_filters!(Handlebars, "../benchdata/filters/handlebars.html");
}

#[test]
fn filters_minijinja() {
    t_filters!(Minijinja, "../benchdata/filters/minijinja.html");
}

#[test]
fn filters_tera() {
    t_filters!(Tera, "../benchdata/filters/tera.html");
}

#[test]
fn filters_upon() {
    t_filters!(Upon, "../benchdata/filters/upon.html");
}

#[test]
fn syntax_minijinja() {
    t_syntax!(Minijinja, "../benchdata/syntax/minijinja.html");
}

#[test]
fn syntax_upon() {
    t_syntax!(Upon, "../benchdata/syntax/upon.html");
}

fn render<'a, E: Engine<'a>>(source: &'a str, syntax: bool, filters: bool) -> String {
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
    if filters {
        engine.add_filters();
    }
    engine.add_template("bench", source);
    engine.render("bench", &ctx)
}
