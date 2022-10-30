//! A lightweight and powerful template engine for Rust.
//!
//! # Overview
//!
//! ## Syntax
//!
//! - Expressions: `{{ user.name }}`
//! - Conditionals: `{% if user.enabled %} ... {% endif %}`
//! - Loops: `{% for user in users %} ... {% endfor %}`
//! - Nested templates: `{% include "nested" %}`
//! - Configurable delimiters: `<? user.name ?>`, `(( if user.enabled ))`
//! - Arbitrary user defined filters: `{{ user.name | replace: "\t", " " }}`
//!
//! See the [`syntax`] module for the full syntax documentation.
//!
//! ## Engine
//!
//! - Clear and well documented API
//! - Customizable value formatters: `{{ user.name | escape_html }}`
//! - Render to a [`String`] or any [`std::io::Write`] implementor
//! - Render using any [`serde`] serializable values
//! - Convenient macro for quick rendering:
//!   `upon::value!{ name: "John", age: 42 }`
//! - Pretty error messages when displayed using `{:#}`
//! - Minimal dependencies and decent runtime performance
//!
//! ## Why another template engine?
//!
//! It's true there are already a lot of template engines for Rust!
//!
//! I created `upon` because I required a template engine that had runtime
//! compiled templates, configurable syntax delimiters and minimal dependencies.
//! I also didn't need support for arbitrary expressions in the template syntax
//! but occasionally I needed something more flexible than outputting simple
//! values. Performance was also a concern for me, template engines like
//! [Handlebars] and [Tera] have a lot of features but can be up to five to
//! seven times slower to render than engines like [TinyTemplate].
//!
//! Basically I wanted something like [TinyTemplate] with support for
//! configurable delimiters and user defined filter functions. The syntax is
//! inspired by template engines like [Liquid] and [Jinja].
//!
//! [Jinja]: https://jinja.palletsprojects.com
//! [Handlebars]: https://crates.io/crates/handlebars
//! [Liquid]: https://liquidjs.com
//! [Tera]: https://crates.io/crates/tera
//! [TinyTemplate]: https://crates.io/crates/tinytemplate
//!
//! ## MSRV
//!
//! Currently the minimum supported version for `upon` is Rust 1.60. The MSRV
//! policy is to support the last **five** minor versions of Rust (~30 weeks).
//!
//! # Getting started
//!
//! First, add the crate to your Cargo manifest.
//!
//! ```sh
//! cargo add upon
//! ```
//!
//! Now construct an [`Engine`]. The engine stores the syntax config, filter
//! functions, formatters, and compiled templates. Generally, you only need to
//! construct one engine during the lifetime of a program.
//!
//! ```
//! let engine = upon::Engine::new();
//! ```
//!
//! Next, [`.add_template`][Engine::add_template] is used to compile and store a
//! template in the engine.
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Finally, the template is rendered by fetching it using
//! [`.get_template`][Engine::get_template] and calling
//! [`.render`][TemplateRef::render].
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! # engine.add_template("hello", "Hello {{ user.name }}!")?;
//! let template = engine.get_template("hello").unwrap();
//! let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! If the lifetime of the template source is shorter than the engine lifetime
//! or you don't need to store the compiled template then you can also use the
//! [`.compile`][Engine::compile] function to return the template directly.
//!
//! ```
//! # let engine = upon::Engine::new();
//! let template = engine.compile("Hello {{ user.name }}!")?;
//! let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! # Features
//!
//! - **filters** _(enabled by default)_ — Enables support for filters in
//!   templates, see [`Engine::add_filter()`]. This does _not_ affect value
//!   formatters, see [`Engine::add_formatter()`]. Disabling this will improve
//!   compile times.
//!
//! - **serde** _(enabled by default)_ — Enables all serde support and pulls in
//!   the [`serde`] crate as a dependency. If disabled then you can use
//!   [`.render_from()`][TemplateRef::render_from] to render templates and
//!   construct the context using [`Value`]'s `From` impls.
//!
//! - **unicode** _(enabled by default)_ — Enables unicode support and pulls in
//!   the [`unicode-ident`][unicode_ident] and [`unicode-width`][unicode_width]
//!   crates. If disabled then unicode identifiers will no longer be allowed in
//!   templates and `.chars().count()` will be used in error formatting.
//!
//! # Examples
//!
//! The following section contains some simple examples. See the
//! [`examples/`][examples] directory in the repository for more.
//!
//! [examples]: https://github.com/rossmacarthur/upon/tree/trunk/examples
//!
//! ## Render using structured data
//!
//! You can render using any [`serde`] serializable data.
//!
//! ```
//! #[derive(serde::Serialize)]
//! struct Context { user: User }
//!
//! #[derive(serde::Serialize)]
//! struct User { name: String }
//!
//! let ctx = Context { user: User { name: "John Smith".into() } };
//!
//! let result = upon::Engine::new()
//!     .compile("Hello {{ user.name }}")?
//!     .render(&ctx)?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ## Transform data using filters
//!
//! Data can be transformed using registered filters.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_filter("lower", str::to_lowercase);
//!
//! let result = engine
//!     .compile("Hello {{ value | lower }}")?
//!     .render(upon::value! { value: "WORLD!" })?;
//!
//! assert_eq!(result, "Hello world!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! See the [`filters`] module documentation for more information on filters.
//!
//! ## Render a template using custom syntax
//!
//! The template syntax can be set by constructing an engine using
//! [`Engine::with_syntax`].
//!
//! ```
//! let syntax = upon::Syntax::builder().expr("<?", "?>").block("<%", "%>").build();
//!
//! let result = upon::Engine::with_syntax(syntax)
//!     .compile("Hello <? user.name ?>")?
//!     .render(upon::value!{ user: { name: "John Smith" }})?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ## Render a template to an `impl io::Write`
//!
//! You can render a template directly to a buffer implementing [`io::Write`]
//! by using [`.render_to_writer()`][TemplateRef::render_to_writer].
//!
//! ```
//! use std::io;
//!
//! let stdout = io::BufWriter::new(io::stdout());
//!
//! upon::Engine::new()
//!     .compile("Hello {{ user.name }}")?
//!     .render_to_writer(stdout, upon::value! { user: { name: "John Smith" }})?;
//! #
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ## Add and use a custom formatter
//!
//! You can add your own custom formatter's or even override the default
//! formatter using [`Engine::set_default_formatter`]. The following example
//! shows how you could add `debug` formatter to the engine.
//!
//! ```
//! use std::fmt::Write;
//! use upon::{Value, Result};
//!
//! let mut engine = upon::Engine::new();
//! engine.add_formatter("debug", |f, value| {
//!     write!(f, "Value::{:?}", value)?;
//!     Ok(())
//! });
//!
//!
//! let result = engine
//!     .compile("User age: {{ user.age | debug }}")?
//!     .render(upon::value! { user: { age: 23 } })?;
//!
//! assert_eq!(result, "User age: Value::Integer(23)");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! See the [`fmt`] module documentation for more information.

#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "filters")]
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub mod filters;
pub mod fmt;
pub mod syntax;

mod compile;
mod error;
#[cfg(feature = "serde")]
mod macros;
mod render;
mod types;
mod value;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io;

pub use crate::error::Error;
pub use crate::types::syntax::{Syntax, SyntaxBuilder};
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use crate::value::to_value;
pub use crate::value::Value;

use crate::compile::Searcher;
#[cfg(feature = "filters")]
use crate::filters::{Filter, FilterArgs, FilterFn, FilterReturn};
use crate::fmt::FormatFn;
use crate::types::program;

/// A type alias for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The compilation and rendering engine.
pub struct Engine<'engine> {
    searcher: Searcher,
    default_formatter: &'engine FormatFn,
    functions: BTreeMap<Cow<'engine, str>, EngineFn>,
    templates: BTreeMap<Cow<'engine, str>, program::Template<'engine>>,
    max_include_depth: usize,
}

enum EngineFn {
    #[cfg(feature = "filters")]
    Filter(Box<FilterFn>),
    Formatter(Box<FormatFn>),
}

/// A compiled template.
pub struct Template<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: program::Template<'source>,
}

/// A reference to a compiled template in an [`Engine`].
#[derive(Clone, Copy)]
pub struct TemplateRef<'engine> {
    engine: &'engine Engine<'engine>,
    name: &'engine str,
    template: &'engine program::Template<'engine>,
}

impl<'engine> Default for Engine<'engine> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'engine> Engine<'engine> {
    /// Construct a new engine.
    #[inline]
    pub fn new() -> Self {
        Self::with_syntax(Syntax::default())
    }

    /// Construct a new engine with custom syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use upon::{Engine, Syntax};
    ///
    /// let syntax = Syntax::builder().expr("<{", "}>").block("<[", "]>").build();
    /// let engine = Engine::with_syntax(syntax);
    /// ```
    #[inline]
    pub fn with_syntax(syntax: Syntax<'engine>) -> Self {
        Self {
            searcher: Searcher::new(syntax),
            default_formatter: &fmt::default,
            functions: BTreeMap::new(),
            templates: BTreeMap::new(),
            max_include_depth: 64,
        }
    }

    /// Set the default formatter.
    #[inline]
    pub fn set_default_formatter<F>(&mut self, f: &'engine F)
    where
        F: Fn(&mut fmt::Formatter<'_>, &Value) -> fmt::Result + Sync + Send + 'static,
    {
        self.default_formatter = f;
    }

    /// Set the maximum length of the template render stack.
    ///
    /// This is the maximum number of nested `{% include ... %}` statements that
    /// are allowed during rendering, as counted from the root template.
    ///
    /// Defaults to `64`.
    #[inline]
    pub fn set_max_include_depth(&mut self, depth: usize) {
        self.max_include_depth = depth;
    }

    /// Add a new filter to the engine.
    ///
    /// **Note:** filters and formatters share the same namespace.
    ///
    /// See the [`filters`] module documentation for more information on
    /// filters.
    #[cfg(feature = "filters")]
    #[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
    #[inline]
    pub fn add_filter<N, F, R, A>(&mut self, name: N, f: F)
    where
        N: Into<Cow<'engine, str>>,
        F: Filter<R, A> + Send + Sync + 'static,
        R: FilterReturn,
        A: for<'a> FilterArgs<'a>,
    {
        self.functions
            .insert(name.into(), EngineFn::Filter(filters::new(f)));
    }

    /// Add a new value formatter to the engine.
    ///
    /// **Note:** filters and formatters share the same namespace.
    #[inline]
    pub fn add_formatter<N, F>(&mut self, name: N, f: F)
    where
        N: Into<Cow<'engine, str>>,
        F: Fn(&mut fmt::Formatter<'_>, &Value) -> fmt::Result + Sync + Send + 'static,
    {
        self.functions
            .insert(name.into(), EngineFn::Formatter(Box::new(f)));
    }

    /// Add a template to the engine.
    ///
    /// The template will be compiled and stored under the given name.
    ///
    /// You can either pass a borrowed ([`&str`]) or owned ([`String`]) template
    /// to this function. When passing a borrowed template, the lifetime needs
    /// to be at least as long as the engine lifetime. For shorter template
    /// lifetimes use [`.compile(..)`][Engine::compile].
    #[inline]
    pub fn add_template<N, S>(&mut self, name: N, source: S) -> Result<()>
    where
        N: Into<Cow<'engine, str>>,
        S: Into<Cow<'engine, str>>,
    {
        let name = name.into();
        let source = source.into();
        let template = compile::template(self, source).map_err(|e| e.with_template_name(&name))?;
        self.templates.insert(name, template);
        Ok(())
    }

    /// Lookup a template by name.
    #[inline]
    pub fn get_template(&self, name: &str) -> Option<TemplateRef<'_>> {
        self.templates
            .get_key_value(name)
            .map(|(name, template)| TemplateRef {
                engine: self,
                name,
                template,
            })
    }

    /// Compile a template.
    ///
    /// The template will not be stored in the engine. The advantage over using
    /// [`.add_template(..)`][Engine::add_template] here is that the template
    /// source does not need to outlive the engine.
    #[inline]
    pub fn compile<'source>(&self, source: &'source str) -> Result<Template<'_, 'source>> {
        let template = compile::template(self, Cow::Borrowed(source))?;
        Ok(Template {
            engine: self,
            template,
        })
    }
}

impl std::fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("searcher", &(..))
            .field("default_formatter", &(..))
            .field("functions", &self.functions)
            .field("templates", &self.templates)
            .field("max_include_depth", &self.max_include_depth)
            .finish()
    }
}

impl std::fmt::Debug for EngineFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            #[cfg(feature = "filters")]
            Self::Filter(_) => "Filter",
            Self::Formatter(_) => "Formatter",
        };
        f.debug_tuple(name).finish()
    }
}

impl<'engine, 'source> Template<'engine, 'source> {
    /// Render the template to a string using the provided value.
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render<S>(&self, ctx: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        render::template(self.engine, &self.template, to_value(ctx)?)
    }

    /// Render the template to a writer using the provided value.
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render_to_writer<W, S>(&self, writer: W, ctx: S) -> Result<()>
    where
        W: io::Write,
        S: serde::Serialize,
    {
        render::template_to(self.engine, &self.template, writer, to_value(ctx)?)
    }

    /// Render the template to a string using the provided value.
    #[inline]
    pub fn render_from<V>(&self, ctx: V) -> Result<String>
    where
        V: Into<Value>,
    {
        render::template(self.engine, &self.template, ctx.into())
    }

    /// Render the template to a writer using the provided value.
    #[inline]
    pub fn render_to_writer_from<W, V>(&self, writer: W, ctx: V) -> Result<()>
    where
        W: io::Write,
        V: Into<Value>,
    {
        render::template_to(self.engine, &self.template, writer, ctx.into())
    }

    /// Returns the original template source.
    #[inline]
    pub fn source(&self) -> &str {
        &self.template.source
    }
}

impl std::fmt::Debug for Template<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template")
            .field("engine", &(..))
            .field("template", &self.template)
            .finish()
    }
}

impl<'engine> TemplateRef<'engine> {
    /// Render the template to a string using the provided value.
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render<S>(&self, ctx: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        render::template(self.engine, self.template, to_value(ctx)?)
            .map_err(|e| e.with_template_name(self.name))
    }

    /// Render the template to a writer using the provided value.
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render_to_writer<W, S>(&self, writer: W, ctx: S) -> Result<()>
    where
        W: io::Write,
        S: serde::Serialize,
    {
        render::template_to(self.engine, self.template, writer, to_value(ctx)?)
            .map_err(|e| e.with_template_name(self.name))
    }

    /// Render the template to a string using the provided value.
    #[inline]
    pub fn render_from<V>(&self, ctx: V) -> Result<String>
    where
        V: Into<Value>,
    {
        render::template(self.engine, self.template, ctx.into())
            .map_err(|e| e.with_template_name(self.name))
    }

    /// Render the template to a writer using the provided value.
    #[inline]
    pub fn render_to_writer_from<W, V>(&self, writer: W, ctx: V) -> Result<()>
    where
        W: io::Write,
        V: Into<Value>,
    {
        render::template_to(self.engine, self.template, writer, ctx.into())
            .map_err(|e| e.with_template_name(self.name))
    }

    /// Returns the original template source.
    #[inline]
    pub fn source(&self) -> &'engine str {
        &self.template.source
    }
}

impl std::fmt::Debug for TemplateRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateRef")
            .field("engine", &(..))
            .field("name", &self.name)
            .field("template", &self.template)
            .finish()
    }
}
