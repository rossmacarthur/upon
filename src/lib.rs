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
//! - Arbitrary user defined functions: `{{ user.name | replace: "\t", " " }}`
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
//! values (hence functions). Performance was also a concern for me, template
//! engines like [Handlebars] and [Tera] have a lot of features but can be up to
//! five to seven times slower to render than engines like [TinyTemplate].
//!
//! Basically I wanted something like [TinyTemplate] with support for
//! configurable delimiters and user defined functions. The syntax is inspired
//! by template engines like [Liquid] and [Jinja].
//!
//! [Jinja]: https://jinja.palletsprojects.com
//! [Handlebars]: https://crates.io/crates/handlebars
//! [Liquid]: https://liquidjs.com
//! [Tera]: https://crates.io/crates/tera
//! [TinyTemplate]: https://crates.io/crates/tinytemplate
//!
//! ## MSRV
//!
//! Currently the minimum supported version for `upon` is Rust 1.66. The MSRV
//! will only ever be increased in a breaking release.
//!
//! # Getting started
//!
//! First, add the crate to your Cargo manifest.
//!
//! ```sh
//! cargo add upon
//! ```
//!
//! Now construct an [`Engine`]. The engine stores the syntax config, functions,
//! formatters, and compiled templates. Generally, you only need to construct
//! one engine during the lifetime of a program.
//!
//! ```
//! let engine = upon::Engine::new();
//! ```
//!
//! Next, [`add_template(..)`][Engine::add_template] is used to compile and store a
//! template in the engine.
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Finally, the template is rendered by fetching it using
//! [`template(..)`][Engine::template], calling
//! [`render(..)`][TemplateRef::render] and rendering to a string.
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! # engine.add_template("hello", "Hello {{ user.name }}!")?;
//! let result = engine
//!     .template("hello")
//!     .render(upon::value!{ user: { name: "John Smith" }})
//!     .to_string()?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! # Further reading
//!
//! - The [`syntax`] module documentation outlines the template syntax.
//! - The [`functions`] module documentation describes functions and how they
//!   work.
//! - The [`fmt`] module documentation contains information on value formatters.
//! - In addition to the examples in the current document, the
//!   [`examples/`][examples] directory in the repository contains some more
//!   concrete code examples.
//!
//! [examples]: https://github.com/rossmacarthur/upon/tree/trunk/examples
//!
//! # Features
//!
//! The following crate features are available.
//!
//! - **`functions`** _(enabled by default)_ — Enables support for functions in
//!   templates (see [`Engine::add_function`]). This does _not_ affect value
//!   formatters (see [`Engine::add_formatter`]). Disabling this will improve
//!   compile times.
//!
//! - **`serde`** _(enabled by default)_ — Enables all serde support and pulls
//!   in the [`serde`] crate as a dependency. If disabled then you can use
//!   [`render_from(..)`][TemplateRef::render_from] to render templates and
//!   construct the context using [`Value`]'s `From` impls.
//!
//! - **`syntax`** _(disabled by default)_ — Enables support for configuring
//!   custom delimiters in templates (see [`Engine::with_syntax`]) and pulls in
//!   the [`aho-corasick`][aho_corasick] crate.
//!
//! - **`unicode`** _(enabled by default)_ — Enables unicode support and pulls
//!   in the [`unicode-ident`][unicode_ident] and
//!   [`unicode-width`][unicode_width] crates. If disabled then unicode
//!   identifiers will no longer be allowed in templates and `.chars().count()`
//!   will be used in error formatting.
//!
//! To disable all features or to use a subset you need to set `default-features
//! = false` in your Cargo manifest and then enable the features that you would
//! like. For example to use **`serde`** but disable **`functions`** and
//! **`unicode`** you would do the following.
//!
//! ```toml
//! [dependencies]
//! upon = { version = "...", default-features = false, features = ["serde"] }
//! ```
//!
//! # Examples
//!
//! ## Nested templates
//!
//! You can include other templates by name using `{% include .. %}`.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//! engine.add_template("goodbye", "Goodbye {{ user.name }}!")?;
//! engine.add_template("nested", "{% include \"hello\" %}\n{% include \"goodbye\" %}")?;
//!
//! let result = engine.template("nested")
//!     .render(upon::value!{ user: { name: "John Smith" }})
//!     .to_string()?;
//! assert_eq!(result, "Hello John Smith!\nGoodbye John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ## Render to writer
//!
//! Instead of rendering to a string it is possible to render the template to
//! any [`std::io::Write`] implementor using
//! [`to_writer(..)`][crate::Renderer::to_writer].
//!
//! ```
//! use std::io;
//!
//! let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//!
//! let mut stdout = io::BufWriter::new(io::stdout());
//! engine
//!     .template("hello")
//!     .render(upon::value!{ user: { name: "John Smith" }})
//!     .to_writer(&mut stdout)?;
//! // Prints: Hello John Smith!
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ## Borrowed templates with short lifetimes
//!
//! If the lifetime of the template source is shorter than the engine lifetime
//! or you don't need to store the compiled template then you can also use the
//! [`compile(..)`][Engine::compile] function to return the template directly.
//!
//! ```
//! # let engine = upon::Engine::new();
//! let template = engine.compile("Hello {{ user.name }}!")?;
//! let result = template
//!     .render(&engine, upon::value!{ user: { name: "John Smith" }})
//!     .to_string()?;
//! assert_eq!(result, "Hello John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ## Custom template store and function
//!
//! The [`compile(..)`][Engine::compile] function can also be used in
//! conjunction with a custom template store which can allow for more advanced
//! use cases. For example: relative template paths or controlling template
//! access.
//!
//! ```
//! # let engine = upon::Engine::new();
//! let mut store = std::collections::HashMap::<&str, upon::Template>::new();
//! store.insert("hello", engine.compile("Hello {{ user.name }}!")?);
//! store.insert("goodbye", engine.compile("Goodbye {{ user.name }}!")?);
//! store.insert("nested", engine.compile("{% include \"hello\" %}\n{% include \"goodbye\" %}")?);
//!
//! let result = store.get("nested")
//!     .unwrap()
//!     .render(&engine, upon::value!{ user: { name: "John Smith" }})
//!     .with_template_fn(|name| {
//!         store
//!             .get(name)
//!             .ok_or_else(|| String::from("template not found"))
//!     })
//!     .to_string()?;
//! assert_eq!(result, "Hello John Smith!\nGoodbye John Smith!");
//! # Ok::<(), upon::Error>(())
//! ```

#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod fmt;
#[cfg(feature = "functions")]
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub mod functions;
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
#[cfg(feature = "syntax")]
#[cfg_attr(docsrs, doc(cfg(feature = "syntax")))]
pub use crate::types::syntax::{Syntax, SyntaxBuilder};
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use crate::value::to_value;
pub use crate::value::Value;

use crate::compile::Searcher;
use crate::fmt::DynFormatter;
#[cfg(feature = "functions")]
use crate::functions::{DynFunction, Function, FunctionArgs, FunctionReturn};
use crate::types::program;

/// A type alias for results in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The compilation and rendering engine.
pub struct Engine<'engine> {
    searcher: Searcher,
    default_formatter: &'engine DynFormatter,
    callables: BTreeMap<Cow<'engine, str>, EngineBoxCallable>,
    templates: BTreeMap<Cow<'engine, str>, program::Template<'engine>>,
    max_include_depth: usize,
}

/// A type of callable stored in the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineCallable {
    /// A formatter. See [`Engine::add_formatter`].
    Formatter,

    /// A function. See [`Engine::add_function`].
    #[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
    #[cfg(feature = "functions")]
    Function,
}

enum EngineBoxCallable {
    Formatter(Box<DynFormatter>),
    #[cfg(feature = "functions")]
    Function(Box<DynFunction>),
}

type ValueFn<'a> = dyn Fn(&[ValueMember<'_>]) -> std::result::Result<Value, String> + 'a;

/// A member in a value path.
///
/// Passed to custom value function when using
/// [`render_from_fn`][Template::render_from_fn].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueMember<'a> {
    /// The type of member access (direct or optional).
    pub op: ValueAccessOp,
    /// The index or key being accessed.
    pub access: ValueAccess<'a>,
}

/// A key in a value path.
///
/// Passed to custom value function when using
/// [`render_from_fn`][Template::render_from_fn].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueAccess<'a> {
    /// An index into an array like `2` in `user.names.2`.
    Index(usize),

    /// A key lookup from a map or member access like `name` in `user.name`.
    Key(&'a str),
}

/// The type of member access.
///
/// Passed to custom value function when using
/// [`render_from_fn`][Template::render_from_fn].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueAccessOp {
    /// A member access like `.` in `user.name`.
    Direct,

    /// A member access like `?.` in `user?.name`.
    Optional,
}

/// A compiled template created using [`Engine::compile`].
///
/// For convenience this struct's lifetime is not tied to the lifetime of the
/// engine. However, it is considered a logic error to attempt to render this
/// template using a different engine than the one that created it. If that
/// happens the render call may panic or produce incorrect output.
#[cfg_attr(internal_debug, derive(Debug))]
pub struct Template<'source> {
    template: program::Template<'source>,
}

/// A reference to a compiled template in an [`Engine`].
#[derive(Clone, Copy)]
pub struct TemplateRef<'engine> {
    engine: &'engine Engine<'engine>,
    name: &'engine str,
    template: &'engine program::Template<'engine>,
}

impl Default for Engine<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'engine> Engine<'engine> {
    /// Construct a new engine.
    #[inline]
    pub fn new() -> Self {
        Self::with_searcher(Searcher::new())
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
    ///
    /// # Note
    ///
    /// Passing a custom syntax to this function always uses the `aho-corasick`
    /// implementation for searching. This means that even if you pass the
    /// default syntax to this function it is *not* equivalent to
    /// [`Engine::new()`][Engine::new].
    #[cfg_attr(docsrs, doc(cfg(feature = "syntax")))]
    #[cfg(feature = "syntax")]
    #[inline]
    pub fn with_syntax(syntax: Syntax<'engine>) -> Self {
        Self::with_searcher(Searcher::with_syntax(syntax))
    }

    #[inline]
    fn with_searcher(searcher: Searcher) -> Self {
        Self {
            searcher,
            default_formatter: &fmt::default,
            callables: BTreeMap::new(),
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
    /// Formatters and functions share the same namespace. If a formatter or
    /// function with the same name already exists in the engine, it is replaced
    /// and `Some(_)` with the type of callable that was replaced is returned,
    /// else `None` is returned.
    #[inline]
    pub fn add_formatter<N, F>(&mut self, name: N, f: F) -> Option<EngineCallable>
    where
        N: Into<Cow<'engine, str>>,
        F: Fn(&mut fmt::Formatter<'_>, &Value) -> fmt::Result + Sync + Send + 'static,
    {
        self.callables
            .insert(name.into(), EngineBoxCallable::Formatter(Box::new(f)))
            .map(|f| f.discriminant())
    }

    /// Add a new function to the engine.
    ///
    /// See the [`functions`] module documentation for more information on
    /// functions.
    ///
    /// # Note
    ///
    /// Formatters and functions share the same namespace. If a formatter or
    /// function with the same name already exists in the engine, it is replaced
    /// and `Some(_)` with the type of callable that was replaced is returned,
    /// else `None` is returned.
    #[cfg(feature = "functions")]
    #[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
    #[inline]
    pub fn add_function<N, F, R, A>(&mut self, name: N, f: F) -> Option<EngineCallable>
    where
        N: Into<Cow<'engine, str>>,
        F: Function<R, A> + Send + Sync + 'static,
        R: FunctionReturn,
        A: FunctionArgs,
    {
        self.callables
            .insert(name.into(), EngineBoxCallable::Function(functions::new(f)))
            .map(|f| f.discriminant())
    }
    #[deprecated(
        since = "0.10.0",
        note = "use `add_function` instead, all functions can be used as filters"
    )]
    #[cfg(feature = "functions")]
    #[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
    #[inline]
    pub fn add_filter<N, F, R, A>(&mut self, name: N, f: F) -> Option<EngineCallable>
    where
        N: Into<Cow<'engine, str>>,
        F: Function<R, A> + Send + Sync + 'static,
        R: FunctionReturn,
        A: FunctionArgs,
    {
        self.add_function(name, f)
    }

    /// Remove a formatter or function by name.
    ///
    /// # Note
    ///
    /// Formatters and functions share the same namespace. If a formatter or
    /// function with this name existed in the engine, it is replaced and
    /// `Some(_)` with the type of function that was replaced is returned, else
    /// `None` is returned.
    pub fn remove_callable(&mut self, name: &str) -> Option<EngineCallable> {
        self.callables.remove(name).map(|f| f.discriminant())
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
        match compile::template(self, source.into()) {
            Ok(template) => {
                self.templates.insert(name.into(), template);
                Ok(())
            }
            Err(err) => Err(err.with_template_name(name.into().into())),
        }
    }

    /// Lookup a template by name.
    ///
    /// # Panics
    ///
    /// If the template with the given name does not exist.
    ///
    #[inline]
    #[track_caller]
    pub fn template(&self, name: &str) -> TemplateRef<'_> {
        match self.get_template(name) {
            Some(template) => template,
            None => panic!("template with name '{name}' does not exist in engine"),
        }
    }

    /// Lookup a template by name, returning `None` if it does not exist.
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
    pub fn compile<'source, S>(&self, source: S) -> Result<Template<'source>>
    where
        S: Into<Cow<'source, str>>,
    {
        let template = compile::template(self, source.into())?;
        Ok(Template { template })
    }
}

impl std::fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("searcher", &self.searcher)
            .field("default_formatter", &format_args!("FormatterFn"))
            .field("callables", &self.callables)
            .field("templates", &self.templates)
            .field("max_include_depth", &self.max_include_depth)
            .finish()
    }
}

impl EngineBoxCallable {
    fn discriminant(&self) -> EngineCallable {
        match self {
            #[cfg(feature = "functions")]
            Self::Function(_) => EngineCallable::Function,
            Self::Formatter(_) => EngineCallable::Formatter,
        }
    }
}

impl std::fmt::Debug for EngineBoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(match self {
            #[cfg(feature = "functions")]
            Self::Function(_) => "Function",
            Self::Formatter(_) => "Formatter",
        })
        .finish()
    }
}

impl<'render> Template<'render> {
    /// Render the template using the provided [`serde`] value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[inline]
    pub fn render<S>(&self, engine: &'render Engine<'render>, ctx: S) -> Renderer<'_>
    where
        S: serde::Serialize,
    {
        Renderer::with_serde(engine, &self.template, None, ctx)
    }

    /// Render the template using the provided value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from(
        &self,
        engine: &'render Engine<'render>,
        ctx: &'render Value,
    ) -> Renderer<'_> {
        Renderer::with_value(engine, &self.template, None, ctx)
    }

    /// Render the using the provided value function.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from_fn<F>(&self, engine: &'render Engine<'render>, value_fn: F) -> Renderer<'_>
    where
        F: Fn(&[ValueMember<'_>]) -> std::result::Result<Value, String> + 'render,
    {
        Renderer::with_value_fn(engine, &self.template, None, Box::new(value_fn))
    }

    /// Returns the original template source.
    #[inline]
    pub fn source(&self) -> &str {
        &self.template.source
    }
}

#[cfg(not(internal_debug))]
impl std::fmt::Debug for Template<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template").finish_non_exhaustive()
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
    pub fn render<S>(&self, ctx: S) -> Renderer<'_>
    where
        S: serde::Serialize,
    {
        Renderer::with_serde(self.engine, self.template, Some(self.name), ctx)
    }

    /// Render the template using the provided value.
    ///
    /// The returned struct must be consumed using
    /// [`.to_string()`][crate::Renderer::to_string] or
    /// [`.to_writer(..)`][crate::Renderer::to_writer].
    #[inline]
    pub fn render_from(&self, ctx: &'render Value) -> Renderer<'render> {
        Renderer::with_value(self.engine, self.template, Some(self.name), ctx)
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
        Renderer::with_value_fn(
            self.engine,
            self.template,
            Some(self.name),
            Box::new(value_fn),
        )
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
