//! A tiny template engine.
//!
//! # Features
//!
//! - Rendering values: `{{ user.name }}`
//! - Conditionals: `{% if user.enabled %} ... {% endif %}`
//! - Loops: `{% for user in users %} ... {% endfor %}`
//! - Customizable filter functions: `{{ value | my_filter }}`
//! - Configurable template delimiters: `<? user.name ?>`, `(( if user.enabled ))`
//! - Render any [`serde`][serde] serializable values.
//! - Macro for quick rendering: `data!{ name: "John", age: 42 }`
//!
//! # Introduction
//!
//! Your entry point is the compilation and rendering [`Engine`], this stores
//! the delimiter settings, registered templates, and filter functions.
//! Generally, you only need to construct one engine.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! ```
//!
//! Add templates to the engine using `.add_template()` either using a string
//! literal or using [`include_str!`].
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! engine.add_template("hello", "Hello {{ user.name }}!")?;
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Now render the template using its name.
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! # engine.add_template("hello", "Hello {{ user.name }}!")?;
//! let result = engine.render("hello", upon::data!{ user: { name: "John" } })?;
//! assert_eq!(result, "Hello John!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! For convenience you can also compile and render templates without storing
//! the template in the engine.
//!
//! ```
//! # let mut engine = upon::Engine::new();
//! let template = engine.compile("Hello {{ user.name }}!")?;
//! template.render(upon::data!{ user: { name: "John" } })?;
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! # Examples
//!
//! ### Render using structured data
//!
//! ```
//! use upon::Engine;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Data {
//!     user: User,
//! }
//! #[derive(Serialize)]
//! struct User {
//!     name: String,
//! }
//!
//! let data = Data { user: User { name: "John Smith".into() } };
//!
//! let result = Engine::new().compile("Hello {{ user.name }}")?.render(data)?;
//!
//! assert_eq!(result, "Hello John Smith");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Transform data using filters
//!
//! ```
//! use upon::{data, Engine, Value};
//!
//! fn lower(mut v: Value) -> Value {
//!     if let Value::String(s) = &mut v {
//!         *s = s.to_lowercase();
//!     }
//!     v
//! }
//!
//! let mut engine = Engine::new();
//! engine.add_filter("lower", lower);
//!
//! let result = engine
//!     .compile("Hello {{ value | lower }}")?
//!     .render(data! { value: "WORLD!" })?;
//!
//! assert_eq!(result, "Hello world!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! ### Render a template using custom tags
//!
//! ```
//! let result = upon::Engine::with_delims("<?", "?>", "<%", "%>")
//!     .compile("Hello <? value ?>")?
//!     .render(upon::data! { value: "World!" })?;
//!
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```

mod ast;
mod compile;
mod error;
mod lex;
mod macros;
mod render;
mod span;
pub mod value;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

pub use crate::error::{Error, Result};
use crate::span::Span;
pub use crate::value::{to_value, Value};

/// The compilation and rendering engine.
#[derive(Clone)]
pub struct Engine<'e> {
    delims: Delimiters<'e>,
    templates: HashMap<String, ast::Template<'e>>,
    filters: HashMap<String, Arc<dyn Fn(Value) -> Value + Send + Sync + 'e>>,
}

/// Delimiter configuration.
#[derive(Debug, Clone)]
struct Delimiters<'e> {
    begin_expr: &'e str,
    end_expr: &'e str,
    begin_block: &'e str,
    end_block: &'e str,
}

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template<'e> {
    engine: &'e Engine<'e>,
    template: ast::Template<'e>,
}

impl<'e> Default for Engine<'e> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'e> Engine<'e> {
    pub fn new() -> Self {
        Self {
            delims: Delimiters::default(),
            templates: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    pub fn with_delims(
        begin_expr: &'e str,
        end_expr: &'e str,
        begin_block: &'e str,
        end_block: &'e str,
    ) -> Self {
        Self {
            delims: Delimiters::new(begin_expr, end_expr, begin_block, end_block),
            templates: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Value) -> Value + Send + Sync + 'e,
    {
        self.filters.insert(name.into(), Arc::new(f));
    }

    pub fn remove_filter<Q: ?Sized>(&mut self, name: &Q)
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.filters.remove(name);
    }

    /// Add a new named template to the engine.
    pub fn add_template(&mut self, name: impl Into<String>, source: &'e str) -> Result<()> {
        let t = compile::template(source, &self.delims)?;
        self.templates.insert(name.into(), t);
        Ok(())
    }

    /// Remove a named template from the engine.
    ///
    /// # Panics
    ///
    /// If the template does not exist.
    #[track_caller]
    pub fn remove_template(&mut self, name: &'e str) {
        self.templates.remove(name).unwrap();
    }

    /// Compile an unamed template and return it.
    pub fn compile(&'e self, source: &'e str) -> Result<Template<'e>> {
        Template::compile(self, source)
    }

    /// Render a named template to a string using the provided data.
    ///
    /// # Panics
    ///
    /// If the template does not exist.
    pub fn render<S>(&'e self, name: &str, data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        let t = self.templates.get(name).unwrap();
        render::template(self, t, to_value(data)?)
    }
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filters = f
            .debug_map()
            .entries(self.filters.keys().map(|k| (k, "<filter>")))
            .finish();
        f.debug_struct("Engine")
            .field("delims", &self.delims)
            .field("templates", &self.templates)
            .field("filters", &filters)
            .finish()
    }
}

impl Default for Delimiters<'_> {
    fn default() -> Self {
        Self::new("{{", "}}", "{%", "%}")
    }
}

impl<'e> Delimiters<'e> {
    /// Returns a new tag configuration.
    const fn new(
        begin_expr: &'e str,
        end_expr: &'e str,
        begin_block: &'e str,
        end_block: &'e str,
    ) -> Self {
        Self {
            begin_expr,
            end_expr,
            begin_block,
            end_block,
        }
    }
}

impl<'e> Template<'e> {
    #[inline]
    fn compile(engine: &'e Engine<'e>, source: &'e str) -> Result<Self> {
        let template = compile::template(source, &engine.delims)?;
        Ok(Self { engine, template })
    }

    #[inline]
    pub fn source(&self) -> &'e str {
        self.template.source
    }

    /// Render the template to a string using the provided data.
    #[inline]
    pub fn render<S>(&self, data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        render::template(self.engine, &self.template, to_value(data)?)
    }
}
