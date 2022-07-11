pub mod context;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

/// Abstraction for a template engine.
pub trait Engine<'a> {
    fn name() -> &'static str;
    fn new() -> Self;
    fn add_template(&mut self, name: &'static str, source: &'a str);
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize;
}

////////////////////////////////////////////////////////////////////////////////
/// handlebars
////////////////////////////////////////////////////////////////////////////////

pub type Handlebars<'engine> = handlebars::Handlebars<'engine>;

impl<'engine> Engine<'engine> for Handlebars<'engine> {
    #[inline]
    fn name() -> &'static str {
        "handlebars"
    }

    #[inline]
    fn new() -> Self {
        let mut hbs = handlebars::Handlebars::new();
        // handlebars escapes HTML by default, so lets add a default formatter
        // to make the benchmark a bit fairer.
        hbs.register_escape_fn(handlebars::no_escape);
        hbs
    }

    #[inline]
    fn add_template(&mut self, name: &'static str, source: &'engine str) {
        self.register_template_string(name, source).unwrap();
    }

    #[inline]
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize,
    {
        self.render(name, ctx).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// liquid
////////////////////////////////////////////////////////////////////////////////

pub struct Liquid {
    parser: liquid::Parser,
    store: HashMap<&'static str, liquid::Template>,
}

impl<'engine> Engine<'engine> for Liquid {
    #[inline]
    fn name() -> &'static str {
        "liquid"
    }

    #[inline]
    fn new() -> Self {
        Self {
            parser: liquid::ParserBuilder::with_stdlib().build().unwrap(),
            store: HashMap::new(),
        }
    }

    #[inline]
    fn add_template(&mut self, name: &'static str, source: &'engine str) {
        let template = self.parser.parse(source).unwrap();
        self.store.insert(name, template);
    }

    #[inline]
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize,
    {
        self.store
            .get(name)
            .unwrap()
            .render(&liquid::to_object(ctx).unwrap())
            .unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// minijinja
////////////////////////////////////////////////////////////////////////////////

pub type Minijinja<'engine> = minijinja::Environment<'engine>;

impl<'engine> Engine<'engine> for Minijinja<'engine> {
    #[inline]
    fn name() -> &'static str {
        "minijinja"
    }

    #[inline]
    fn new() -> Self {
        minijinja::Environment::new()
    }

    #[inline]
    fn add_template(&mut self, name: &'static str, source: &'engine str) {
        self.add_template(name, source).unwrap();
    }

    #[inline]
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize,
    {
        self.get_template(name).unwrap().render(ctx).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// tera
////////////////////////////////////////////////////////////////////////////////

pub type Tera = tera::Tera;

impl<'engine> Engine<'engine> for Tera {
    #[inline]
    fn name() -> &'static str {
        "tera"
    }

    #[inline]
    fn new() -> Self {
        tera::Tera::default()
    }

    #[inline]
    fn add_template(&mut self, name: &'static str, source: &'engine str) {
        self.add_raw_template(name, source).unwrap();
    }

    #[inline]
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize,
    {
        let ctx = tera::Context::from_serialize(ctx).unwrap();
        self.render(name, &ctx).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// tinytemplate
////////////////////////////////////////////////////////////////////////////////

pub type TinyTemplate<'engine> = tinytemplate::TinyTemplate<'engine>;

impl<'engine> Engine<'engine> for TinyTemplate<'engine> {
    #[inline]
    fn name() -> &'static str {
        "tinytemplate"
    }

    #[inline]
    fn new() -> Self {
        let mut tt = tinytemplate::TinyTemplate::new();
        // tinytemplate escapes HTML by default, so lets add a default formatter
        // to make the benchmark a bit fairer.
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        tt
    }

    #[inline]
    fn add_template(&mut self, name: &'static str, source: &'engine str) {
        self.add_template(name, source).unwrap();
    }

    #[inline]
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize,
    {
        self.render(name, ctx).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// upon
////////////////////////////////////////////////////////////////////////////////

pub type Upon<'engine> = upon::Engine<'engine>;

impl<'engine> Engine<'engine> for upon::Engine<'engine> {
    #[inline]
    fn name() -> &'static str {
        "upon"
    }

    #[inline]
    fn new() -> Self {
        upon::Engine::new()
    }

    #[inline]
    fn add_template(&mut self, name: &'static str, source: &'engine str) {
        self.add_template(name, source).unwrap();
    }

    #[inline]
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize,
    {
        self.get_template(name).unwrap().render(ctx).unwrap()
    }
}
