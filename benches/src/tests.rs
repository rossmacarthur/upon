use crate::context;
use crate::{Engine, Handlebars, Liquid, Minijinja, Tera, TinyTemplate, Upon};

macro_rules! t {
    ($E:ty, $source:literal) => {{
        let result = render::<$E>(include_str!(concat!("../benchdata/", $source)));
        goldie::assert!(result);
    }};
}

macro_rules! t_syntax {
    ($E:ty, $source:literal) => {{
        let result = render_syntax::<$E>(include_str!(concat!("../benchdata/", $source)));
        goldie::assert!(result);
    }};
}

macro_rules! t_recurse {
    ($E:ty, $source:literal) => {
        let result = render_recurse::<$E>(include_str!(concat!("../benchdata/", $source)));
        goldie::assert!(result);
    };
}

// /////////////////////////////////////////////////////////////////////////////
//  Basic
// /////////////////////////////////////////////////////////////////////////////

#[test]
fn basic_handlebars() {
    t!(Handlebars, "basic/handlebars.html");
}

#[test]
fn basic_liquid() {
    t!(Liquid, "basic/liquid.html");
}

#[test]
fn basic_minijinja() {
    t!(Minijinja, "basic/jinja.html");
}

#[test]
fn basic_tera() {
    t!(Tera, "basic/jinja.html");
}

#[test]
fn basic_tinytemplate() {
    t!(TinyTemplate, "basic/tinytemplate.html");
}

#[test]
fn basic_upon() {
    t!(Upon, "basic/jinja.html");
}

// /////////////////////////////////////////////////////////////////////////////
//  Functions
// /////////////////////////////////////////////////////////////////////////////

#[test]
fn functions_handlebars() {
    t!(Handlebars, "functions/handlebars.html");
}

#[test]
fn functions_minijinja() {
    t!(Minijinja, "functions/jinja.html");
}

#[test]
fn functions_tera() {
    t!(Tera, "functions/jinja.html");
}

#[test]
fn functions_upon() {
    t!(Upon, "functions/jinja.html");
}

// /////////////////////////////////////////////////////////////////////////////
//  Literals
// /////////////////////////////////////////////////////////////////////////////

#[test]
fn literals_minijinja() {
    t!(Minijinja, "literals/jinja.html");
}

#[test]
fn literals_upon() {
    t!(Upon, "literals/jinja.html");
}

#[test]
fn syntax_minijinja() {
    t_syntax!(Minijinja, "syntax/jinja.html");
}

#[test]
fn syntax_upon() {
    t_syntax!(Upon, "syntax/jinja.html");
}

// /////////////////////////////////////////////////////////////////////////////
//  Recurse
// /////////////////////////////////////////////////////////////////////////////

#[test]
fn recurse_handlebars() {
    t_recurse!(Handlebars, "recurse/handlebars.html");
}

#[test]
fn recurse_liquid() {
    t_recurse!(Liquid, "recurse/liquid.html");
}

#[test]
fn recurse_minijinja() {
    t_recurse!(Minijinja, "recurse/minijinja.html");
}

#[test]
fn recurse_upon() {
    t_recurse!(Upon, "recurse/upon.html");
}

fn render<'a, E: Engine<'a>>(source: &'a str) -> String {
    let ctx = context::plain();
    let mut engine = E::new();
    engine.add_template("bench", source);
    engine.render("bench", &ctx)
}

fn render_syntax<'a, E: Engine<'a>>(source: &'a str) -> String {
    let ctx = context::plain();
    let mut engine = E::with_syntax(("{", "}"), ("<%", "%>"), ("<#", "#>"));
    engine.add_template("bench", source);
    engine.render("bench", &ctx)
}

fn render_recurse<'a, E: Engine<'a>>(source: &'a str) -> String {
    let rec = context::recurse(20);
    let mut engine = E::new();
    engine.add_partial("bench", source);
    engine.add_template("bench", source);
    engine.render("bench", &rec)
}
