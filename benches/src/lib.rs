pub mod context;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

/// Abstraction for a template engine.
pub trait Engine<'a>: Sized {
    fn name() -> &'static str;
    fn new() -> Self;
    fn with_syntax(
        _expr: (&'static str, &'static str),
        _block: (&'static str, &'static str),
        _comment: (&'static str, &'static str),
    ) -> Self {
        unimplemented!()
    }
    fn add_filters(&mut self) {
        unimplemented!()
    }
    fn add_template(&mut self, name: &'static str, source: &'a str);
    fn render<S>(&self, name: &'static str, ctx: &S) -> String
    where
        S: serde::Serialize;
}

// /////////////////////////////////////////////////////////////////////////////
//  handlebars
// /////////////////////////////////////////////////////////////////////////////

pub type Handlebars<'engine> = handlebars::Handlebars<'engine>;

use handlebars::handlebars_helper;

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
    fn add_filters(&mut self) {
        handlebars_helper!(lower: |s: String| s.to_lowercase());
        handlebars_helper!(reverse: |s: String| String::from_iter(s.chars().rev()));
        self.register_helper("lower", Box::new(lower));
        self.register_helper("reverse", Box::new(reverse));
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

// /////////////////////////////////////////////////////////////////////////////
//  liquid
// /////////////////////////////////////////////////////////////////////////////

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

// /////////////////////////////////////////////////////////////////////////////
//  minijinja
// /////////////////////////////////////////////////////////////////////////////

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
    fn with_syntax(
        (variable_start, variable_end): (&'static str, &'static str),
        (block_start, block_end): (&'static str, &'static str),
        (comment_start, comment_end): (&'static str, &'static str),
    ) -> Self {
        let mut env = minijinja::Environment::new();
        env.set_syntax(minijinja::Syntax {
            block_start: block_start.into(),
            block_end: block_end.into(),
            variable_start: variable_start.into(),
            variable_end: variable_end.into(),
            comment_start: comment_start.into(),
            comment_end: comment_end.into(),
        })
        .unwrap();
        env
    }

    fn add_filters(&mut self) {}

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

// /////////////////////////////////////////////////////////////////////////////
//  tera
// /////////////////////////////////////////////////////////////////////////////

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
    fn add_filters(&mut self) {}

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

// /////////////////////////////////////////////////////////////////////////////
//  tinytemplate
// /////////////////////////////////////////////////////////////////////////////

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

// /////////////////////////////////////////////////////////////////////////////
//  upon
// /////////////////////////////////////////////////////////////////////////////

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
    fn with_syntax(
        (begin_expr, end_expr): (&'static str, &'static str),
        (begin_block, end_block): (&'static str, &'static str),
        (begin_comment, end_comment): (&'static str, &'static str),
    ) -> Self {
        upon::Engine::with_syntax(
            upon::Syntax::builder()
                .expr(begin_expr, end_expr)
                .block(begin_block, end_block)
                .comment(begin_comment, end_comment)
                .build(),
        )
    }

    #[inline]
    fn add_filters(&mut self) {
        self.add_filter("lower", str::to_lowercase);
        self.add_filter("reverse", |s: &str| String::from_iter(s.chars().rev()));
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
        self.template(name).render(ctx).to_string().unwrap()
    }
}
