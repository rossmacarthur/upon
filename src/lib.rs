//! A simple, powerful template engine with minimal dependencies and
//! configurable delimiters.
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
//! ## Engine
//!
//! - Clear and well documented API
//! - Customizable value formatters: `{{ user.name | escape_html }}`
//! - Render to a [`String`] or any [`std::io::Write`] implementor
//! - Render using any [`serde`] serializable values
//! - Convenient macro for quick rendering:
//!   `upon::value!{ name: "John", age: 42 }`
//! - Pretty error messages when displayed using `{:#}`
//! - Format agnostic (does _not_ escape values for HTML by default)
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
//! values (hence filters). Performance was also a concern for me, template
//! engines like [Handlebars] and [Tera] have a lot of features but can be up to
//! five to seven times slower to render than engines like [TinyTemplate].
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
//! Currently the minimum supported version for `upon` is Rust 1.65. Disabling
//! the **`filters`** feature reduces it to Rust 1.60. The MSRV will only ever
//! be increased in a breaking release.
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
//! Next, [`add_template`][Engine::add_template] is used to compile and store a
//! template in the engine.
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Finally, the template is rendered by fetching it using
//! [`get_template`][Engine::get_template] and calling
//! [`render`][TemplateRef::render].
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! # engine.add_template("hello", "Hello {{ user.name }}!")?;
//! let template = engine.get_template("hello").unwrap();
//! let result = template.render(upon::value!{ user: { name: "John Smith" }}).to_string()?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! If the lifetime of the template source is shorter than the engine lifetime
//! or you don't need to store the compiled template then you can also use the
//! [`compile`][Engine::compile] function to return the template directly.
//!
//! ```
//! # let engine = upon::Engine::new();
//! let template = engine.compile("Hello {{ user.name }}!")?;
//! let result = template.render(upon::value!{ user: { name: "John Smith" }}).to_string()?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! # Further reading
//!
//! - The [`syntax`] module documentation outlines the template syntax.
//! - The [`filters`] module documentation describes filters and how they work.
//! - The [`fmt`] module documentation contains information on value formatters.
//! - The [`examples/`][examples] directory in the repository contains concrete
//!   code examples.
//!
//! [examples]: https://github.com/rossmacarthur/upon/tree/trunk/examples
//!
//! # Features
//!
//! The following crate features are available.
//!
//! - **`filters`** _(enabled by default)_ — Enables support for filters in
//!   templates (see [`Engine::add_filter`]). This does _not_ affect value
//!   formatters (see [`Engine::add_formatter`]). Disabling this will improve
//!   compile times.
//!
//! - **`serde`** _(enabled by default)_ — Enables all serde support and pulls
//!   in the [`serde`] crate as a dependency. If disabled then you can use
//!   [`render_from`][TemplateRef::render_from] to render templates and
//!   construct the context using [`Value`]'s `From` impls.
//!
//! - **`unicode`** _(enabled by default)_ — Enables unicode support and pulls
//!   in the [`unicode-ident`][unicode_ident] and
//!   [`unicode-width`][unicode_width] crates. If disabled then unicode
//!   identifiers will no longer be allowed in templates and `.chars().count()`
//!   will be used in error formatting.
//!
//! To disable all features or to use a subset you need to set `default-features
//! = false` in your Cargo manifest and then enable the features that you would
//! like. For example to use **`serde`** but disable **`filters`** and
//! **`unicode`** you would do the following.
//!
//! ```toml
//! [dependencies]
//! upon = { version = "...", default-features = false, features = ["serde"] }
//! ```

#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "filters")]
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub mod filters;
pub mod fmt;
#[cfg(doc)]
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

pub use crate::error::Error;
pub use crate::render::Renderer;
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
    functions: BTreeMap<Cow<'engine, str>, EngineBoxFn>,
    templates: BTreeMap<Cow<'engine, str>, program::Template<'engine>>,
    max_include_depth: usize,
}

/// A type of function stored in the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFn {
    /// A value formatter. See [`Engine::add_formatter`].
    Formatter,

    /// A filter. See [`Engine::add_filter`].
    #[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
    #[cfg(feature = "filters")]
    Filter,
}

enum EngineBoxFn {
    Formatter(Box<FormatFn>),
    #[cfg(feature = "filters")]
    Filter(Box<FilterFn>),
}

type ValueFn<'a> = dyn Fn(&[ValueMember]) -> std::result::Result<Value, String> + 'a;

/// A member in a value path.
///
/// Passed to custom value function when using
/// [`render_from_fn`][Template::render_from_fn].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueMember<'a> {
    pub op: ValueAccessOp,
    pub access: ValueAccess<'a>,
}

/// A key in a value path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueAccess<'a> {
    /// An index into an array like `2` in `user.names.2`.
    Index(usize),

    /// A key lookup from a map or member access like `name` in `user.name`.
    Key(&'a str),
}

/// The type of member access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueAccessOp {
    /// A member access like `.` in `user.name`.
    Direct,

    /// A member access like `?.` in `user?.name`.
    Optional,
}

/// A compiled template created using [`Engine::compile`].
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

    /// Set the default formatter.
    ///
    /// The default formatter defines how values are formatted in the rendered
    /// template. If not configured this defaults to [`fmt::default`] which does
    /// not perform any escaping. See the [`fmt`] module documentation for more
    /// information on value formatters.
    #[inline]
    pub fn set_default_formatter<F>(&mut self, f: &'engine F)
    where
        F: Fn(&mut fmt::Formatter<'_>, &Value) -> fmt::Result + Sync + Send + 'static,
    {
        self.default_formatter = f;
    }

    /// Add a new value formatter to the engine.
    ///
    /// See the [`fmt`] module documentation for more information on formatters.
    ///
    /// # Note
    ///
    /// Formatters and filters share the same namespace. If a filter or
    /// formatter with the same name already exists in the engine, it is
    /// replaced and `Some(_)` with the type of function that was replaced is
    /// returned, else `None` is returned.
    #[inline]
    pub fn add_formatter<N, F>(&mut self, name: N, f: F) -> Option<EngineFn>
    where
        N: Into<Cow<'engine, str>>,
        F: Fn(&mut fmt::Formatter<'_>, &Value) -> fmt::Result + Sync + Send + 'static,
    {
        self.functions
            .insert(name.into(), EngineBoxFn::Formatter(Box::new(f)))
            .map(|f| f.discriminant())
    }

    /// Add a new filter to the engine.
    ///
    /// See the [`filters`] module documentation for more information on
    /// filters.
    ///
    /// # Note
    ///
    /// Formatters and filters share the same namespace. If a filter or
    /// formatter with the same name already exists in the engine, it is
    /// replaced and `Some(_)` with the type of function that was replaced is
    /// returned, else `None` is returned.
    #[cfg(feature = "filters")]
    #[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
    #[inline]
    pub fn add_filter<N, F, R, A>(&mut self, name: N, f: F) -> Option<EngineFn>
    where
        N: Into<Cow<'engine, str>>,
        F: Filter<R, A> + Send + Sync + 'static,
        R: FilterReturn,
        A: FilterArgs,
    {
        self.functions
            .insert(name.into(), EngineBoxFn::Filter(filters::new(f)))
            .map(|f| f.discriminant())
    }

    /// Remove a formatter or filter by name.
    ///
    /// # Note
    ///
    /// Formatters and filters share the same namespace. If a filter or
    /// formatter with name existed in the engine, it is replaced and `Some(_)`
    /// with the type of function that was replaced is returned, else `None` is
    /// returned.
    pub fn remove_function(&mut self, name: &str) -> Option<EngineFn> {
        self.functions.remove(name).map(|f| f.discriminant())
    }

    /// Add a template to the engine.
    ///
    /// The template will be compiled and stored under the given name.
    ///
    /// You can either pass a borrowed template ([`&str`]) or owned template
    /// ([`String`]) to this function. When passing a borrowed template, the
    /// lifetime needs to be at least as long as the engine lifetime. For
    /// shorter template lifetimes use [`.compile(..)`][Engine::compile].
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

    /// Remove a template by name.
    ///
    /// Returns `true` if a template was removed, `false` if there was no
    /// template of that name.
    #[inline]
    pub fn remove_template(&mut self, name: &str) -> bool {
        self.templates.remove(name).is_some()
    }

    /// Compile a template.
    ///
    /// The template will not be stored in the engine. The advantage over using
    /// [`.add_template(..)`][Engine::add_template] here is that if the template
    /// source is borrowed, it does not need to outlive the engine.
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

impl EngineBoxFn {
    fn discriminant(&self) -> EngineFn {
        match self {
            #[cfg(feature = "filters")]
            Self::Filter(_) => EngineFn::Filter,
            Self::Formatter(_) => EngineFn::Formatter,
        }
    }
}

impl std::fmt::Debug for EngineBoxFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            #[cfg(feature = "filters")]
            Self::Filter(_) => "Filter",
            Self::Formatter(_) => "Formatter",
        };
        f.debug_tuple(name).finish()
    }
}

impl<'render> Template<'render, 'render> {
    /// Render the template using the provided [`serde`] value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render<S>(&self, ctx: S) -> Renderer<'_, S>
    where
        S: serde::Serialize,
    {
        Renderer::with_serde(self.engine, &self.template, ctx)
    }

    /// Render the template using the provided value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from(&self, ctx: &'render Value) -> Renderer<'_> {
        Renderer::with_value(self.engine, &self.template, ctx)
    }

    /// Render the using the provided value function.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from_fn<F>(&self, value_fn: F) -> Renderer<'_>
    where
        F: Fn(&[ValueMember<'_>]) -> std::result::Result<Value, String> + 'render,
    {
        Renderer::with_value_fn(self.engine, &self.template, Box::new(value_fn))
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

impl<'render> TemplateRef<'render> {
    /// Render the template using the provided [`serde`] value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render<S>(&self, ctx: S) -> Renderer<'_, S>
    where
        S: serde::Serialize,
    {
        Renderer::with_serde(self.engine, self.template, ctx)
    }

    /// Render the template using the provided value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from(&self, ctx: &'render Value) -> Renderer<'render> {
        Renderer::with_value(self.engine, self.template, ctx)
    }

    /// Render the using the provided value function.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from_fn<F>(&self, value_fn: F) -> Renderer<'render>
    where
        F: Fn(&[ValueMember<'_>]) -> std::result::Result<Value, String> + 'render,
    {
        Renderer::with_value_fn(self.engine, self.template, Box::new(value_fn))
    }

    /// Returns the original template source.
    #[inline]
    pub fn source(&self) -> &'render str {
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
